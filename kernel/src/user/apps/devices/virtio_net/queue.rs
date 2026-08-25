//! Split virtqueue implementation (virtio spec §2.7).
//!
//! Key design choice: **one descriptor per packet**. RX buffers are laid
//! out as `[NetHdr][payload]` in a single contiguous allocation and handed
//! to the device write-only; TX buffers use the same layout, filled by us
//! and handed to the device read-only. VIRTIO_F_VERSION_1 allows "any
//! layout", so nothing requires splitting the header and payload across
//! two chained descriptors — and not chaining is a real win: half the
//! descriptor-ring pressure per packet, one avail-ring publish instead of
//! two, and one fewer pointer for vhost to chase per packet on the host
//! side. The only place this driver uses a multi-descriptor chain is the
//! control queue at setup time (`chain_add`), which is off the hot path.
//!
//! Notification is batched: `rx_add`/`tx_add` only stage descriptors
//! locally, `flush()` publishes them to the device and rings the doorbell
//! *once*, using VIRTIO_F_EVENT_IDX to skip the doorbell entirely when the
//! device (vhost) hasn't asked for one. Call `flush()` once per polling
//! iteration (e.g. once after each `Interface::poll()`), not once per
//! packet — that's the difference between one vmexit per batch and one
//! vmexit per packet.

use alloc::vec;
use alloc::vec::Vec;
use core::ptr::{addr_of, addr_of_mut};
use core::sync::atomic::{fence, Ordering};
use x86_64::VirtAddr;
use crate::user::api::user_syscalls::translate_addr;
use super::hal;

const DESC_F_NEXT: u16 = 1;
const DESC_F_WRITE: u16 = 2;

#[repr(C, align(16))]
struct Desc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C, align(2))]
struct AvailHdr {
    #[allow(dead_code)]
    flags: u16,
    idx: u16,
}

#[repr(C, align(4))]
struct UsedElem {
    id: u32,
    len: u32,
}

#[repr(C, align(4))]
struct UsedHdr {
    #[allow(dead_code)]
    flags: u16,
    idx: u16,
}

pub struct VirtQueue {
    size: u16,
    mask: u16,

    desc: *mut Desc,
    avail_hdr: *mut AvailHdr,
    avail_ring: *mut u16,
    #[allow(dead_code)]
    avail_used_event: *mut u16, // trailing field after avail ring; unused (we don't rely on device-side interrupt suppression, we disable interrupts outright)
    used_hdr: *mut UsedHdr,
    used_ring: *mut UsedElem,
    used_avail_event: *mut u16, // trailing field after used ring; device writes here to tell us when it wants a doorbell

    free_head: u16,
    free_count: u16,
    avail_idx_shadow: u16, // next slot we'll publish into avail.ring
    last_used_idx: u16,    // last used.idx we've consumed

    /// Per-descriptor buffer bookkeeping: (virtual pointer, length) of the
    /// DMA buffer currently backing each descriptor id.
    bufs: Vec<(*mut u8, u32)>,

    desc_phys: u64,
    avail_phys: u64,
    used_phys: u64,

    pub notify_addr: *mut u16,
    pub queue_index: u16,
}

unsafe impl Send for VirtQueue {}

impl VirtQueue {
    pub fn new(queue_index: u16, size: u16, notify_addr: *mut u16) -> Self {
        assert!(size.is_power_of_two(), "queue size must be a power of two");

        let desc_bytes = size as usize * size_of::<Desc>();
        let avail_bytes = 4 + size as usize * 2 + 2;
        let used_bytes = 4 + size as usize * 8 + 2;

        // Each ring gets its own page(s): avail (driver-written,
        // device-read) and used (device-written, driver-read) must never
        // share a cache line, or the driver core and the vhost thread end
        // up bouncing it back and forth on every packet.
        let desc_region = hal::alloc_dma(round_up(desc_bytes, 4096));
        let avail_region = hal::alloc_dma(round_up(avail_bytes, 4096));
        let used_region = hal::alloc_dma(round_up(used_bytes, 4096));

        unsafe {
            core::ptr::write_bytes(desc_region.virt, 0, desc_region.len);
            core::ptr::write_bytes(avail_region.virt, 0, avail_region.len);
            core::ptr::write_bytes(used_region.virt, 0, used_region.len);
        }

        let desc = desc_region.virt as *mut Desc;
        for i in 0..size {
            unsafe { (*desc.add(i as usize)).next = i.wrapping_add(1) };
        }

        let avail_hdr = avail_region.virt as *mut AvailHdr;
        let avail_ring = unsafe { avail_region.virt.add(4) as *mut u16 };
        let avail_used_event = unsafe { avail_ring.add(size as usize) };

        let used_hdr = used_region.virt as *mut UsedHdr;
        let used_ring = unsafe { used_region.virt.add(4) as *mut UsedElem };
        let used_avail_event = unsafe { used_ring.add(size as usize) as *mut u16 };

        VirtQueue {
            size,
            mask: size - 1,
            desc,
            avail_hdr,
            avail_ring,
            avail_used_event,
            used_hdr,
            used_ring,
            used_avail_event,
            free_head: 0,
            free_count: size,
            avail_idx_shadow: 0,
            last_used_idx: 0,
            bufs: vec![(core::ptr::null_mut::<u8>(), 0u32); size as usize],
            desc_phys: desc_region.phys,
            avail_phys: avail_region.phys,
            used_phys: used_region.phys,
            notify_addr,
            queue_index,
        }
    }

    pub fn size(&self) -> u16 {
        self.size
    }
    pub fn desc_phys(&self) -> u64 {
        self.desc_phys
    }
    pub fn avail_phys(&self) -> u64 {
        self.avail_phys
    }
    pub fn used_phys(&self) -> u64 {
        self.used_phys
    }

    fn alloc_desc(&mut self) -> u16 {
        let id = self.free_head;
        self.free_head = unsafe { (*self.desc.add(id as usize)).next };
        self.free_count -= 1;
        id
    }

    fn free_desc(&mut self, id: u16) {
        unsafe { (*self.desc.add(id as usize)).next = self.free_head };
        self.free_head = id;
        self.free_count += 1;
    }

    /// Post an RX buffer (device-writable), staged for the next `flush()`.
    pub fn rx_add(&mut self, buf: *mut u8, len: u32) {
        let id = self.alloc_desc();
        unsafe {
            let d = self.desc.add(id as usize);
            (*d).addr = translate_addr(VirtAddr::from_ptr(buf)).unwrap().as_u64();
            (*d).len = len;
            (*d).flags = DESC_F_WRITE;
            (*d).next = 0;
        }
        self.bufs[id as usize] = (buf, len);
        self.push_avail(id);
    }

    /// Post a TX buffer (device-readable, already filled with hdr+payload),
    /// staged for the next `flush()`.
    pub fn tx_add(&mut self, buf: *mut u8, len: u32) {
        let id = self.alloc_desc();
        unsafe {
            let d = self.desc.add(id as usize);
            (*d).addr = translate_addr(VirtAddr::from_ptr(buf)).unwrap().as_u64();
            (*d).len = len;
            (*d).flags = 0;
            (*d).next = 0;
        }
        self.bufs[id as usize] = (buf, len);
        self.push_avail(id);
    }

    /// Post a chain of buffers as one descriptor chain. Only used for the
    /// control queue (setup-time MQ negotiation) — the RX/TX hot path is
    /// always single-descriptor, see module docs.
    pub fn chain_add(&mut self, parts: &[(*mut u8, u32, bool /* device-writable */)]) -> u16 {
        let ids: Vec<u16> = parts.iter().map(|_| self.alloc_desc()).collect();
        for (i, &(buf, len, write)) in parts.iter().enumerate() {
            let id = ids[i];
            let has_next = i + 1 < ids.len();
            unsafe {
                let d = self.desc.add(id as usize);
                (*d).addr = translate_addr(VirtAddr::from_ptr(buf)).unwrap().as_u64();
                (*d).len = len;
                (*d).flags = (if write { DESC_F_WRITE } else { 0 }) | (if has_next { DESC_F_NEXT } else { 0 });
                (*d).next = if has_next { ids[i + 1] } else { 0 };
            }
            self.bufs[id as usize] = (buf, len);
        }
        self.push_avail(ids[0]);
        ids[0]
    }

    #[inline(always)]
    fn push_avail(&mut self, desc_id: u16) {
        let slot = self.avail_idx_shadow & self.mask;

        unsafe {
            self.avail_ring
                .add(slot as usize)
                .write(desc_id);
        }

        self.avail_idx_shadow = self.avail_idx_shadow.wrapping_add(1);
    }

    /// Make staged descriptors visible to the device and ring the doorbell
    /// if the device actually needs one (VIRTIO_F_EVENT_IDX suppression).
    /// Call once per poll cycle after adding buffers — this is what turns
    /// "one vmexit per packet" into "one vmexit per batch".
    pub fn flush(&mut self) {
        let published_before = unsafe { addr_of!((*self.avail_hdr).idx).read_volatile() };
        if self.avail_idx_shadow == published_before {
            return; // nothing staged
        }

        fence(Ordering::Release); // descriptor writes must be visible before idx is
        unsafe { addr_of_mut!((*self.avail_hdr).idx).write_volatile(self.avail_idx_shadow) };
        fence(Ordering::SeqCst); // idx write must be visible before we read used_avail_event

        if self.needs_notify(published_before) {
            unsafe { self.notify_addr.write_volatile(self.queue_index) };
        }
    }

    fn needs_notify(&self, old_idx: u16) -> bool {
        // Standard vring_need_event(event, new, old) suppression check
        // (virtio spec §2.7.10): notify iff the newly-published range
        // crosses the event index the device asked for.
        let event = unsafe { self.used_avail_event.read_volatile() };
        let new_idx = self.avail_idx_shadow;
        new_idx.wrapping_sub(event).wrapping_sub(1) < new_idx.wrapping_sub(old_idx)
    }

    /// Pop exactly one completed descriptor from the used ring, if any.
    /// Used for RX, where each call corresponds 1:1 to one packet handed
    /// up to the network stack.
    pub fn pop_used(&mut self) -> Option<(*mut u8, u32)> {
        let used_idx = unsafe { addr_of!((*self.used_hdr).idx).read_volatile() };
        if self.last_used_idx == used_idx {
            return None;
        }
        fence(Ordering::Acquire);
        let slot = self.last_used_idx & self.mask;
        let elem = unsafe { self.used_ring.add(slot as usize).read_volatile() };
        let id = elem.id as u16;
        let (buf, _cap) = self.bufs[id as usize];
        self.free_desc(id);
        self.last_used_idx = self.last_used_idx.wrapping_add(1);
        Some((buf, elem.len))
    }
}

fn round_up(x: usize, align: usize) -> usize {
    (x + align - 1) & !(align - 1)
}