//! Hardware/OS abstraction glue.
//!
//! Everything here is a thin wrapper around facilities your kernel already
//! has (physical memory allocation, virt<->phys translation, MMIO mapping).
//! Wire these up to your real syscalls/MM code — the rest of the driver
//! only depends on these functions, not on any OS-specific API, so this
//! should be the only file you need to edit to port the driver.

use crate::memory::tables::PHYSICAL_MEMORY_OFFSET;
use crate::user::api::user_syscalls::{add_flags_to_frame, add_page_table_entry, allocate, translate_addr};
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

/// A region of memory mapped into our address space, with its physical
/// base address recorded alongside. Used both for MMIO windows (PCI BAR
/// capability structures) and DMA memory (virtqueues, packet buffers).
#[derive(Clone, Copy)]
pub struct MappedRegion {
    pub virt: *mut u8,
    pub phys: u64,
    pub len: usize,
}

/// Map `len` bytes of physical MMIO space starting at `phys` into our
/// address space. Used for the virtio-pci capability structures (common
/// cfg, notify, isr, device cfg) that live inside PCI BARs.
///
/// IMPORTANT: this must be an *uncached* / device-memory mapping. Do not
/// reuse it for DMA buffers, which want normal cacheable memory.
pub fn map_mmio(phys: u64, len: usize) -> MappedRegion {
    let bar_virt = VirtAddr::new(PHYSICAL_MEMORY_OFFSET.get().unwrap().as_u64() + phys);

    if translate_addr(bar_virt).is_none() {
        add_page_table_entry(bar_virt);
        add_flags_to_frame(bar_virt, PageTableFlags::USER_ACCESSIBLE, true);
    }

    MappedRegion {
        virt: bar_virt.as_mut_ptr(),
        phys,
        len
    }
}

/// Allocate `len` bytes of physically-contiguous, cacheable, DMA-capable
/// memory (descriptor rings, packet buffers).
///
/// For this driver to behave correctly, that memory must:
///   1. be physically contiguous for its whole length (virtio descriptors
///      carry a single base physical address + length, no scatter list
///      per allocation), and
///   2. ideally be backed by huge pages, both so (1) is actually true
///      without extra bookkeeping, and so the RX/TX hot path doesn't take
///      TLB misses walking a large buffer pool under sustained line-rate
///      traffic.
///
/// This assumes `allocate(None, len)` gives you exactly that. If your
/// allocator can hand back scattered pages, either add an explicit
/// hugepage flag to this call, or change every call site in this driver
/// to request memory in page-sized chunks and manage physical
/// discontiguity per-descriptor.
pub fn alloc_dma(len: usize) -> MappedRegion {
    let virt_addr = allocate(None, len);
    let phys_addr = translate_addr(virt_addr).unwrap();
    add_flags_to_frame(virt_addr, PageTableFlags::USER_ACCESSIBLE, true);
    MappedRegion { virt: virt_addr.as_mut_ptr(), phys: phys_addr.as_u64(), len }
}