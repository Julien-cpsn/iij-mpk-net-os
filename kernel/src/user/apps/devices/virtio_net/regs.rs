//! virtio-pci register layouts and constants (virtio spec v1.2 §4.1.4
//! "Virtio Structure PCI Capabilities" and §5.1 "Network Device").

use core::ptr::{addr_of, addr_of_mut};

// ---- device status bits (§2.1) ----
pub const STATUS_ACKNOWLEDGE: u8 = 1;
pub const STATUS_DRIVER: u8 = 2;
pub const STATUS_DRIVER_OK: u8 = 4;
pub const STATUS_FEATURES_OK: u8 = 8;
#[allow(dead_code)]
pub const STATUS_DEVICE_NEEDS_RESET: u8 = 64;
#[allow(dead_code)]
pub const STATUS_FAILED: u8 = 128;

// ---- feature bits ----
#[allow(dead_code)]
pub const VIRTIO_NET_F_CSUM: u64 = 1 << 0;
#[allow(dead_code)]
pub const VIRTIO_NET_F_GUEST_CSUM: u64 = 1 << 1;
pub const VIRTIO_NET_F_MAC: u64 = 1 << 5;
#[allow(dead_code)]
pub const VIRTIO_NET_F_HOST_TSO4: u64 = 1 << 11;
#[allow(dead_code)]
pub const VIRTIO_NET_F_HOST_TSO6: u64 = 1 << 12;
pub const VIRTIO_NET_F_MRG_RXBUF: u64 = 1 << 15;
pub const VIRTIO_NET_F_STATUS: u64 = 1 << 16;
pub const VIRTIO_NET_F_CTRL_VQ: u64 = 1 << 17;
pub const VIRTIO_NET_F_MQ: u64 = 1 << 22;
pub const VIRTIO_F_EVENT_IDX: u64 = 1 << 29;
pub const VIRTIO_F_VERSION_1: u64 = 1 << 32;
#[allow(dead_code)]
pub const VIRTIO_F_RING_PACKED: u64 = 1 << 34; // not implemented by this split-ring driver, see README

/// The feature set this driver negotiates. Deliberately excludes
/// VIRTIO_F_INDIRECT_DESC (an extra pointer chase per descriptor, no
/// benefit for the single-descriptor-per-packet layout used here) and
/// offload features this driver doesn't implement yet (TSO/UFO/checksum
/// offload). Turning on VIRTIO_NET_F_CSUM / GUEST_CSUM / HOST_TSO4 and
/// filling in NetHdr's csum/gso fields in net.rs is the single biggest
/// lever left for bulk (non-pps-bound) throughput — see README.
pub const DRIVER_FEATURES: u64 = VIRTIO_F_VERSION_1
    | VIRTIO_NET_F_MAC
    | VIRTIO_NET_F_STATUS
    | VIRTIO_NET_F_MRG_RXBUF
    | VIRTIO_NET_F_CTRL_VQ
    | VIRTIO_NET_F_MQ
    | VIRTIO_F_EVENT_IDX;

pub const NO_VECTOR: u16 = 0xFFFF;

// ---- common cfg struct (§4.1.4.3) ----
#[repr(C)]
pub struct CommonCfg {
    pub device_feature_select: u32,
    pub device_feature: u32,
    pub driver_feature_select: u32,
    pub driver_feature: u32,
    pub msix_config: u16,
    pub num_queues: u16,
    pub device_status: u8,
    pub config_generation: u8,
    pub queue_select: u16,
    pub queue_size: u16,
    pub queue_msix_vector: u16,
    pub queue_enable: u16,
    pub queue_notify_off: u16,
    pub queue_desc: u64,
    pub queue_avail: u64,
    pub queue_used: u64,
}

/// Volatile-access wrapper. Every field of `CommonCfg` is a live device
/// register, so plain reads/writes through a `&mut CommonCfg` reference
/// are not sound (the compiler may reorder or elide them, and a `&mut`
/// over the whole struct claims exclusive access to bytes we don't
/// actually want to touch). We compute a field pointer with
/// `addr_of!`/`addr_of_mut!` and go through `read_volatile`/
/// `write_volatile` instead.
#[derive(Clone, Copy)]
pub struct Common(pub *mut CommonCfg);

macro_rules! field_rw {
    ($get:ident, $set:ident, $field:ident, $t:ty) => {
        #[inline]
        pub fn $get(&self) -> $t {
            unsafe { addr_of!((*self.0).$field).read_volatile() }
        }
        #[inline]
        pub fn $set(&self, v: $t) {
            unsafe { addr_of_mut!((*self.0).$field).write_volatile(v) }
        }
    };
}

impl Common {
    field_rw!(get_device_feature_select, set_device_feature_select, device_feature_select, u32);
    field_rw!(get_device_feature, set_device_feature_unused, device_feature, u32);
    field_rw!(get_driver_feature_select, set_driver_feature_select, driver_feature_select, u32);
    field_rw!(get_driver_feature, set_driver_feature, driver_feature, u32);
    field_rw!(get_msix_config, set_msix_config, msix_config, u16);

    field_rw!(get_num_queues, set_num_queues_unused, num_queues, u16);
    field_rw!(get_status, set_status, device_status, u8);
    field_rw!(get_queue_select, set_queue_select, queue_select, u16);
    field_rw!(get_queue_size, set_queue_size_unused, queue_size, u16);
    field_rw!(get_queue_msix_vector, set_queue_msix_vector, queue_msix_vector, u16);

    field_rw!(get_queue_enable, set_queue_enable, queue_enable, u16);
    field_rw!(get_queue_notify_off, set_queue_notify_off_unused, queue_notify_off, u16);
    field_rw!(get_queue_desc, set_queue_desc, queue_desc, u64);
    field_rw!(get_queue_avail, set_queue_avail, queue_avail, u64);
    field_rw!(get_queue_used, set_queue_used, queue_used, u64);

    pub fn device_features(&self) -> u64 {
        self.set_device_feature_select(0);
        let lo = self.get_device_feature() as u64;
        self.set_device_feature_select(1);
        let hi = self.get_device_feature() as u64;
        lo | (hi << 32)
    }

    pub fn set_driver_features(&self, features: u64) {
        self.set_driver_feature_select(0);
        self.set_driver_feature(features as u32);
        self.set_driver_feature_select(1);
        self.set_driver_feature((features >> 32) as u32);
    }
}

// ---- virtio_net_config (§5.1.4) ----
#[repr(C)]
pub struct NetConfig {
    pub mac: [u8; 6],
    pub status: u16,
    pub max_virtqueue_pairs: u16,
    #[allow(dead_code)]
    pub mtu: u16,
    // Further fields require features this driver doesn't negotiate.
}

// ---- packet header, virtio-1.0+ layout (§5.1.6.1) ----
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetHdr {
    pub flags: u8,
    pub gso_type: u8,
    pub hdr_len: u16,
    pub gso_size: u16,
    pub csum_start: u16,
    pub csum_offset: u16,
    pub num_buffers: u16,
}
pub const NET_HDR_LEN: usize = size_of::<NetHdr>(); // 12 bytes

pub const NET_HDR: NetHdr = NetHdr {
    flags: 0,
    gso_type: 0,
    hdr_len: 0,
    gso_size: 0,
    csum_start: 0,
    csum_offset: 0,
    num_buffers: 1,
};

// ---- control vq (§5.1.6.5) ----
pub const VIRTIO_NET_CTRL_MQ: u8 = 4;
pub const VIRTIO_NET_CTRL_MQ_VQ_PAIRS_SET: u8 = 0;

#[repr(C)]
pub struct CtrlHdr {
    pub class: u8,
    pub cmd: u8,
}

#[repr(C)]
pub struct CtrlMq {
    pub virtqueue_pairs: u16,
}