//! High-level virtio-net-pci driver: feature negotiation, per-core
//! multiqueue setup, and the zero-copy smoltcp `Device` implementation.

use crate::user::apps::devices::virtio_net::hal::{MappedRegion, alloc_dma};
use crate::user::apps::devices::virtio_net::pci::{Bdf, Ecam, VirtioCapabilities};
use crate::user::apps::devices::virtio_net::queue::VirtQueue;
use crate::user::apps::devices::virtio_net::regs::{Common, CommonCfg, CtrlHdr, CtrlMq, NetConfig, NetHdr, DRIVER_FEATURES, NO_VECTOR, STATUS_ACKNOWLEDGE, STATUS_DRIVER, STATUS_DRIVER_OK, STATUS_FEATURES_OK, VIRTIO_F_VERSION_1, VIRTIO_NET_CTRL_MQ, VIRTIO_NET_CTRL_MQ_VQ_PAIRS_SET, VIRTIO_NET_F_CTRL_VQ, VIRTIO_NET_F_MQ, NET_HDR, NET_HDR_LEN};
use alloc::vec::Vec;
use crate::info;

/// Buffer size for both RX and TX: header + a full non-jumbo Ethernet
/// frame, rounded up. Bump this (and negotiate VIRTIO_NET_F_MTU) for
/// jumbo frames — keep single-descriptor-per-packet either way; prefer a
/// bigger flat buffer over chaining descriptors, see queue.rs.
pub(crate) const BUF_LEN: usize = 256;

struct Queues {
    rx: VirtQueue,
    tx: VirtQueue,
}

/// Owns device-wide state: feature negotiation, all negotiated queue
/// pairs, and the control queue. Hand individual queue pairs off to
/// per-core workers with `take_queue()` — each `VirtioNetQueue` then owns
/// its RX/TX rings and buffer pool outright with no locking, which is
/// what makes multiqueue actually scale instead of bouncing cache lines
/// between cores.
pub struct VirtioNetPci {
    common: Common,
    #[allow(dead_code)]
    device_cfg: *mut NetConfig,
    #[allow(dead_code)]
    isr: *mut u8,
    ctrl: Option<VirtQueue>,
    queues: Vec<Option<Queues>>,
    rx_bufs: Vec<MappedRegion>,
    tx_bufs: Vec<MappedRegion>,
    pub mac: [u8; 6],
    #[allow(dead_code)]
    features: u64,
}

impl VirtioNetPci {
    /// Run the full virtio-pci + virtio-net init sequence (spec §3.1.1)
    /// and bring up `num_queue_pairs` RX/TX queue pairs (clamped to what
    /// the device actually advertises). No MSI-X vectors are configured
    /// anywhere — this driver is pure-polling by design, see README.
    pub fn init(ecam: &Ecam, bdf: Bdf, num_queue_pairs: u16) -> Self {
        let caps = VirtioCapabilities::discover(ecam, bdf);
        let common = Common(caps.common.virt as *mut CommonCfg);

        common.set_status(0); // reset
        while common.get_status() != 0 {
            core::hint::spin_loop();
        }
        common.set_status(STATUS_ACKNOWLEDGE);
        common.set_status(common.get_status() | STATUS_DRIVER);

        let device_features = common.device_features();
        let wanted = DRIVER_FEATURES & device_features;
        assert_ne!(wanted & VIRTIO_F_VERSION_1, 0, "device doesn't support modern virtio (VERSION_1)");
        let mq = wanted & VIRTIO_NET_F_MQ != 0;

        common.set_driver_features(wanted);
        common.set_status(common.get_status() | STATUS_FEATURES_OK);
        assert_ne!(common.get_status() & STATUS_FEATURES_OK, 0, "device rejected our feature set");

        common.set_msix_config(NO_VECTOR); // polling driver: no interrupts

        let device_cfg = caps.device_cfg.virt as *mut NetConfig;
        let mac = unsafe { core::ptr::addr_of!((*device_cfg).mac).read_volatile() };
        let max_pairs = if mq {
            unsafe { core::ptr::addr_of!((*device_cfg).max_virtqueue_pairs).read_volatile() }
        } else {
            1
        };

        const TARGET: &str = "virtio_net";
        info!("Max pair: {max_pairs}");
        let pairs = num_queue_pairs.min(max_pairs).max(1);

        let notify_off_mult = caps.notify_off_multiplier;
        let notify_base = caps.notify_base.virt;

        let setup_queue = |index: u16| -> VirtQueue {
            common.set_queue_select(index);
            let size = common.get_queue_size();
            info!("Queue {index} size: {size}");
            assert!(size > 0, "device reports queue {index} as unavailable");
            common.set_queue_msix_vector(NO_VECTOR);
            let notify_off = common.get_queue_notify_off();
            let notify_addr = unsafe { notify_base.add(notify_off as usize * notify_off_mult as usize) as *mut u16 };
            let q = VirtQueue::new(index, size, notify_addr);
            common.set_queue_desc(q.desc_phys());
            common.set_queue_avail(q.avail_phys());
            common.set_queue_used(q.used_phys());
            common.set_queue_enable(1);
            q
        };

        let mut queues = Vec::with_capacity(pairs as usize);
        let mut rx_bufs = Vec::with_capacity(pairs as usize);
        let mut tx_bufs = Vec::with_capacity(pairs as usize);

        for p in 0..pairs {
            let mut rx = setup_queue(2 * p);
            let tx = setup_queue(2 * p + 1);

            let rx_pool = alloc_dma(rx.size() as usize * BUF_LEN);
            for i in 0..rx.size() as usize {
                let buf = unsafe { rx_pool.virt.add(i * BUF_LEN) };
                rx.rx_add(buf, BUF_LEN as u32);
            }
            rx.flush();
            let tx_pool = alloc_dma(tx.size() as usize * BUF_LEN);

            queues.push(Some(Queues { rx, tx }));
            rx_bufs.push(rx_pool);
            tx_bufs.push(tx_pool);
        }

        let ctrl = if wanted & VIRTIO_NET_F_CTRL_VQ != 0 {
            Some(setup_queue(2 * max_pairs))
        } else {
            None
        };

        common.set_status(common.get_status() | STATUS_DRIVER_OK);

        let mut dev = VirtioNetPci {
            common,
            device_cfg,
            isr: caps.isr.virt,
            ctrl,
            queues,
            rx_bufs,
            tx_bufs,
            mac,
            features: wanted,
        };

        if mq && pairs > 1 {
            dev.set_queue_pairs(pairs);
        }

        dev
    }

    /// Tell the device (and vhost) how many queue pairs to actually use,
    /// via the control queue (spec §5.1.6.5.5). Setup-time only, so a
    /// short busy-wait for the ack is fine.
    fn set_queue_pairs(&mut self, pairs: u16) {
        let ctrl = self.ctrl.as_mut().expect("MQ negotiated without CTRL_VQ");

        #[repr(C)]
        struct Req {
            hdr: CtrlHdr,
            mq: CtrlMq,
        }
        let scratch = alloc_dma(4096);
        let req_ptr = scratch.virt as *mut Req;
        unsafe {
            req_ptr.write(Req {
                hdr: CtrlHdr { class: VIRTIO_NET_CTRL_MQ, cmd: VIRTIO_NET_CTRL_MQ_VQ_PAIRS_SET },
                mq: CtrlMq { virtqueue_pairs: pairs },
            });
        }
        let ack_ptr = unsafe { scratch.virt.add(64) };
        unsafe { ack_ptr.write_volatile(0xFF) };

        ctrl.chain_add(&[
            (req_ptr as *mut u8, size_of::<Req>() as u32, false),
            (ack_ptr, 1, true),
        ]);
        ctrl.flush();

        loop {
            if let Some(_) = ctrl.pop_used() {
                break;
            }
            core::hint::spin_loop();
        }
        let status = unsafe { ack_ptr.read_volatile() };
        assert_eq!(status, 0, "VIRTIO_NET_CTRL_MQ_VQ_PAIRS_SET failed (status={status})");
    }

    /// Hand ownership of queue pair `index` to the caller — typically a
    /// per-core worker that will build its own `smoltcp::Interface`
    /// around it. Panics if already taken.
    ///
    /// NOTE: `self` (and therefore the mapped common-cfg/device-cfg/ISR
    /// registers) must stay alive for as long as any taken queue is in
    /// use — the device as a whole is still "owned" by this struct even
    /// once individual queues are handed out. Keep it in a `static`, a
    /// `Box::leak`, or whatever fits your kernel's driver lifetime model.
    pub fn take_queue(&mut self, index: usize) -> VirtioNetQueue {
        let Queues { rx, tx } = self.queues[index].take().expect("queue already taken");
        let tx_size = tx.size();
        VirtioNetQueue {
            rx,
            tx,
            rx_pool: self.rx_bufs[index],
            tx_pool: self.tx_bufs[index],
            tx_free: (0..tx_size).collect(),
        }
    }
}

/// A single RX/TX queue pair, fully owned by one core. Implements
/// `smoltcp::phy::Device` with zero-copy RX (the RxToken borrows straight
/// into the DMA buffer the device wrote) and TX buffers drawn from a
/// preallocated pool, so there is no allocation on the packet path.
pub struct VirtioNetQueue {
    rx: VirtQueue,
    pub(crate) tx: VirtQueue,
    rx_pool: MappedRegion,
    pub(crate) tx_pool: MappedRegion,
    pub(crate) tx_free: Vec<u16>,
}

impl VirtioNetQueue {
    #[inline]
    pub fn reclaim_tx(&mut self) {
        let pool = self.tx_pool.virt as usize;

        while let Some((buf, _)) = self.tx.pop_used() {
            let slot = (buf as usize - pool) / BUF_LEN;
            self.tx_free.push(slot as u16);
        }
    }

    /// Submit up to `count` packets without notifying the device between them.
    ///
    /// The callback receives `(buffer, slot)` for each packet. It must return
    /// the number of bytes to transmit.
    pub fn transmit_batch(&mut self, count: usize, packet_size: usize) -> usize {
        self.reclaim_tx();
        let count = count.min(self.tx_free.len());

        if count == 0 {
            return 0;
        }

        for _ in 0..count {
            let slot = unsafe {
                self.tx_free.pop().unwrap_unchecked()
            };

            let buf = unsafe {
                self.tx_pool
                    .virt
                    .add(slot as usize * BUF_LEN)
            };

            unsafe {
                core::ptr::copy_nonoverlapping(
                    &NET_HDR as *const NetHdr as *const u8,
                    buf,
                    NET_HDR_LEN,
                );
            }

            self.tx.tx_add(
                buf,
                (NET_HDR_LEN + packet_size) as u32,
            );
        }


        // One avail-ring publication / notification for the entire batch.
        self.flush_tx();

        count
    }

    /// Number of TX buffers currently available.
    #[inline]
    pub fn tx_available(&self) -> usize {
        self.tx_free.len()
    }

    /// Start a TX batch.
    ///
    /// Packets added through the returned batch are published together when
    /// `finish()` is called.
    #[inline]
    pub fn tx_batch(&mut self) -> TxBatch<'_> {
        self.reclaim_tx();
        TxBatch { queue: self }
    }

    #[inline]
    pub fn flush_rx(&mut self) {
        self.rx.flush();
    }

    #[inline]
    pub fn flush_tx(&mut self) {
        self.tx.flush();
    }

    #[inline]
    pub fn flush(&mut self) {
        self.flush_rx();
        self.flush_tx();
    }
}

pub struct TxBatch<'a> {
    queue: &'a mut VirtioNetQueue,
}

impl<'a> TxBatch<'a> {
    /// Allocate one TX buffer from the queue.
    #[inline]
    pub fn alloc(&'a mut self) -> Option<TxBuffer<'a>> {
        let slot = self.queue.tx_free.pop()?;

        let buf = unsafe {
            self.queue
                .tx_pool
                .virt
                .add(slot as usize * BUF_LEN)
        };

        Some(TxBuffer {
            batch: self,
            slot,
            buf,
        })
    }

    /// Number of TX buffers that can still be allocated.
    #[inline]
    pub fn available(&self) -> usize {
        self.queue.tx_free.len()
    }

    /// Publish all packets added to this batch.
    #[inline]
    pub fn finish(self) {
        self.queue.flush();
    }
}

pub struct TxBuffer<'a> {
    batch: &'a mut TxBatch<'a>,
    slot: u16,
    buf: *mut u8,
}

impl<'a> TxBuffer<'a> {
    #[inline]
    pub fn as_mut_slice(&mut self, offset: usize, len: usize) -> &mut [u8] {
        assert!(offset + len <= BUF_LEN);

        unsafe {
            core::slice::from_raw_parts_mut(self.buf.add(offset), len)
        }
    }

    #[inline]
    pub fn submit(self, len: usize) {
        assert!(len <= BUF_LEN);

        self.batch.queue.tx.tx_add(
            self.buf,
            len as u32,
        );
    }
}