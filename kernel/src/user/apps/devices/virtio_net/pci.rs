//! Minimal PCI (ECAM) access + virtio-pci capability discovery.
//!
//! Replaces `virtio_drivers::transport::pci::bus`. This is a small,
//! self-contained ECAM reader plus the vendor-capability walk defined by
//! the virtio-pci spec (§4.1.4), which is all we need to find the MMIO
//! structures a modern virtio-pci device exposes.

use super::hal::{MappedRegion, map_mmio};
use crate::drivers::mmio::{MMCONFIG_PHYS_BASE, MMCONFIG_SIZE};
use crate::user::api::user_syscalls::allocate;

pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;
/// 0x1000 = transitional virtio-net-pci (legacy + modern), 0x1041 =
/// modern-only (e.g. QEMU `disable-legacy=on,disable-modern=off`).
pub const VIRTIO_NET_DEVICE_ID_TRANSITIONAL: u16 = 0x1000;
pub const VIRTIO_NET_DEVICE_ID_MODERN: u16 = 0x1041;

const PCI_CAP_ID_VNDR: u8 = 0x09;
const PCI_STATUS_CAP_LIST: u16 = 1 << 4;

const VIRTIO_PCI_CAP_COMMON_CFG: u8 = 1;
const VIRTIO_PCI_CAP_NOTIFY_CFG: u8 = 2;
const VIRTIO_PCI_CAP_ISR_CFG: u8 = 3;
const VIRTIO_PCI_CAP_DEVICE_CFG: u8 = 4;

#[derive(Clone, Copy, Debug)]
pub struct Bdf {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

/// Raw ECAM config-space accessor for bus 0 (extend `addr()` if you need
/// to walk bridges onto other buses).
pub struct Ecam {
    base: *mut u8,
}

impl Ecam {
    pub fn new() -> Self {
        let phys = *MMCONFIG_PHYS_BASE.get().unwrap();
        let mapped = allocate(Some(phys), MMCONFIG_SIZE);
        Ecam { base: mapped.as_mut_ptr() }
    }

    fn addr(&self, bdf: Bdf, offset: u16) -> *mut u8 {
        let idx = ((bdf.bus as usize) << 20)
            | ((bdf.device as usize) << 15)
            | ((bdf.function as usize) << 12)
            | offset as usize;
        unsafe { self.base.add(idx) }
    }

    pub unsafe fn read8(&self, bdf: Bdf, off: u16) -> u8 {
        unsafe {
            self.addr(bdf, off).read_volatile()
        }
    }
    pub unsafe fn read16(&self, bdf: Bdf, off: u16) -> u16 {
        unsafe {
            (self.addr(bdf, off) as *mut u16).read_volatile()
        }
    }
    pub unsafe fn read32(&self, bdf: Bdf, off: u16) -> u32 {
        unsafe {
            (self.addr(bdf, off) as *mut u32).read_volatile()
        }
    }
    pub unsafe fn write32(&self, bdf: Bdf, off: u16, v: u32) {
        unsafe {
            (self.addr(bdf, off) as *mut u32).write_volatile(v)
        }
    }

    /// Enumerate bus 0, mirroring the original `pci_root.enumerate_bus(0)`.
    pub fn enumerate_bus0(&self) -> impl Iterator<Item = (Bdf, u16, u16)> + '_ {
        (0u8..32).flat_map(move |device| {
            let bdf0 = Bdf { bus: 0, device, function: 0 };
            let vendor0 = unsafe { self.read16(bdf0, 0x00) };
            let multi = vendor0 != 0xFFFF && unsafe { self.read8(bdf0, 0x0E) } & 0x80 != 0;
            let nfuncs: u8 = if multi { 8 } else { 1 };
            (0u8..nfuncs).filter_map(move |function| {
                let bdf = Bdf { bus: 0, device, function };
                let vendor = unsafe { self.read16(bdf, 0x00) };
                if vendor == 0xFFFF {
                    return None;
                }
                let dev_id = unsafe { self.read16(bdf, 0x02) };
                Some((bdf, vendor, dev_id))
            })
        })
    }

    /// Physical base address of a BAR (handles 32- and 64-bit BARs).
    fn bar_base(&self, bdf: Bdf, bar_index: u8) -> u64 {
        let off = 0x10 + (bar_index as u16) * 4;
        let low = unsafe { self.read32(bdf, off) };
        assert_eq!(low & 0x1, 0, "I/O-space BARs are not supported");
        let is_64 = (low >> 1) & 0x3 == 0x2;
        let base_low = (low & !0xF) as u64;
        if is_64 {
            let high = unsafe { self.read32(bdf, off + 4) };
            base_low | ((high as u64) << 32)
        } else {
            base_low
        }
    }

    /// Full BAR size via the standard write-all-ones probe. Only needed
    /// for the notify BAR: the notify address for queue N can legally
    /// land outside the notify capability's advertised `length`
    /// (spec §4.1.4.4), so we must map the whole BAR, not just `length`
    /// bytes at `offset`.
    fn bar_size(&self, bdf: Bdf, bar_index: u8) -> u64 {
        let off = 0x10 + (bar_index as u16) * 4;
        let orig_low = unsafe { self.read32(bdf, off) };
        let is_64 = (orig_low >> 1) & 0x3 == 0x2;

        unsafe { self.write32(bdf, off, 0xFFFF_FFFF) };
        let mask_low = unsafe { self.read32(bdf, off) } & !0xF;

        let size = if is_64 {
            let orig_high = unsafe { self.read32(bdf, off + 4) };
            unsafe { self.write32(bdf, off + 4, 0xFFFF_FFFF) };
            let mask_high = unsafe { self.read32(bdf, off + 4) };
            unsafe { self.write32(bdf, off + 4, orig_high) };
            let mask = (mask_low as u64) | ((mask_high as u64) << 32);
            (!mask).wrapping_add(1)
        } else {
            ((!mask_low) as u64).wrapping_add(1)
        };
        unsafe { self.write32(bdf, off, orig_low) };
        size
    }
}

#[derive(Clone, Copy, Default)]
struct RawCap {
    bar: u8,
    offset: u32,
    length: u32,
    notify_off_multiplier: u32,
}

pub struct VirtioCapabilities {
    pub common: MappedRegion,
    pub notify_base: MappedRegion,
    pub notify_off_multiplier: u32,
    pub isr: MappedRegion,
    pub device_cfg: MappedRegion,
}

impl VirtioCapabilities {
    pub fn discover(ecam: &Ecam, bdf: Bdf) -> Self {
        let status = unsafe { ecam.read16(bdf, 0x06) };
        assert_ne!(status & PCI_STATUS_CAP_LIST, 0, "device has no capability list");

        let mut common = None;
        let mut notify = None;
        let mut isr = None;
        let mut device_cfg = None;

        let mut ptr = unsafe { ecam.read8(bdf, 0x34) };

        while ptr != 0 {
            let cap_id = unsafe { ecam.read8(bdf, ptr as u16) };
            let cap_next = unsafe { ecam.read8(bdf, ptr as u16 + 1) };

            if cap_id == PCI_CAP_ID_VNDR {
                let cfg_type = unsafe { ecam.read8(bdf, ptr as u16 + 3) };
                let bar = unsafe { ecam.read8(bdf, ptr as u16 + 4) };
                let offset = unsafe { ecam.read32(bdf, ptr as u16 + 8) };
                let length = unsafe { ecam.read32(bdf, ptr as u16 + 12) };
                let notify_off_multiplier = if cfg_type == VIRTIO_PCI_CAP_NOTIFY_CFG {
                    unsafe { ecam.read32(bdf, ptr as u16 + 16) }
                } else {
                    0
                };

                let raw = RawCap { bar, offset, length, notify_off_multiplier };
                match cfg_type {
                    VIRTIO_PCI_CAP_COMMON_CFG => common = Some(raw),
                    VIRTIO_PCI_CAP_NOTIFY_CFG => notify = Some(raw),
                    VIRTIO_PCI_CAP_ISR_CFG => isr = Some(raw),
                    VIRTIO_PCI_CAP_DEVICE_CFG => device_cfg = Some(raw),
                    _ => {}
                }
            }
            ptr = cap_next;
        }

        let common = common.expect("no common cfg capability");
        let notify = notify.expect("no notify cfg capability");
        let isr = isr.expect("no isr cfg capability");
        let device_cfg = device_cfg.expect("no device cfg capability");

        let map_cap = |c: RawCap| {
            let bar_base = ecam.bar_base(bdf, c.bar) +  c.offset as u64;

            assert_ne!(c.length, 0, "virtio capability has zero length");

            map_mmio(bar_base, c.length as usize)
        };

        let bar_base = ecam.bar_base(bdf, notify.bar);
        let bar_size = ecam.bar_size(bdf, notify.bar);

        let notify_base = map_mmio(bar_base, bar_size as usize);

        VirtioCapabilities {
            common: map_cap(common),
            notify_base,
            notify_off_multiplier: notify.notify_off_multiplier,
            isr: map_cap(isr),
            device_cfg: map_cap(device_cfg),
        }
    }
}