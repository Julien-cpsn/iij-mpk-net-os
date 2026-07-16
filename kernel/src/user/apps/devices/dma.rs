use crate::memory::tables::PHYSICAL_MEMORY_OFFSET;
use crate::user::api::user_syscalls::{add_flags_to_frame, add_page_table_entry, allocate, translate_addr};
use crate::trace;
use alloc::alloc::dealloc;
use core::alloc::Layout;
use core::ptr::NonNull;
use virtio_drivers::{BufferDirection, Hal, PhysAddr as VirtioPhysAddr};
use x86_64::structures::paging::{PageSize, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

const TARGET: &str = "DMA";

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (VirtioPhysAddr, NonNull<u8>) {
        trace!("DMA ALLOC");

        let allocation_size = pages * Size4KiB::SIZE as usize;

        let virt_addr = allocate(None, allocation_size);
        add_flags_to_frame(virt_addr, PageTableFlags::USER_ACCESSIBLE, true);
        let phys_addr = translate_addr(virt_addr).unwrap();

        trace!("Allocation: phys {:#X}, virt {:#X}, {} bytes", phys_addr, virt_addr.as_u64(), allocation_size);

        (phys_addr.as_u64(), NonNull::new(virt_addr.as_mut_ptr()).unwrap())
    }

    unsafe fn dma_dealloc(phys_addr: VirtioPhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        trace!("DMA DEALLOC");

        let deallocation_size = pages * Size4KiB::SIZE as usize;

        trace!("Deallocation: phys {:#X}, virt {:#X}, {} bytes", phys_addr, vaddr.addr(), deallocation_size);

        unsafe { dealloc(vaddr.as_ptr(), Layout::array::<u8>(deallocation_size).unwrap()); }

        0
    }

    unsafe fn mmio_phys_to_virt(paddr: VirtioPhysAddr, size: usize) -> NonNull<u8> {
        trace!("MMIO PHYS TO VIRT");

        let addr = PHYSICAL_MEMORY_OFFSET.get().unwrap().as_u64() + paddr;

        let virt_addr = VirtAddr::new(addr);

        if translate_addr(virt_addr).is_none() {
            add_page_table_entry(virt_addr);
            add_flags_to_frame(virt_addr, PageTableFlags::USER_ACCESSIBLE, true);
        }

        trace!("Phys to virt: {paddr:#X} -> {virt_addr:#X}, {size} bytes");

        NonNull::new(addr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> VirtioPhysAddr {
        trace!("SHARE");

        let phys_addr = translate_addr(VirtAddr::from_ptr(buffer.as_ptr())).unwrap();

        trace!("Share: virt {:#X} -> phys {:#X}", buffer.addr().get(), phys_addr);

        phys_addr.as_u64()
    }

    unsafe fn unshare(phys_addr: VirtioPhysAddr, buffer: NonNull<[u8]>, _direction: BufferDirection) {
        trace!("UNSHARE");

        trace!("Unshare: virt {:#X} -> phys {:#X}", buffer.addr().get(), phys_addr);
    }
}