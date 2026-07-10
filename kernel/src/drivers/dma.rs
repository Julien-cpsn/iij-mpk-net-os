use crate::memory::tables::{add_page_table_entry, translate_addr, PHYSICAL_MEMORY_OFFSET};
use alloc::alloc::{alloc_zeroed, dealloc};
use core::alloc::Layout;
use core::ptr::NonNull;
use goolog::trace;
use virtio_drivers::{BufferDirection, Hal, PhysAddr as VirtioPhysAddr};
use x86_64::structures::paging::{PageSize, Size4KiB};
use x86_64::VirtAddr;


const GOOLOG_TARGET: &str = "DMA";

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (VirtioPhysAddr, NonNull<u8>) {
        let allocation_size = pages * Size4KiB::SIZE as usize;

        let allocated_space = unsafe { alloc_zeroed(Layout::array::<u8>(allocation_size).unwrap()) };
        let phys_addr = translate_addr(VirtAddr::from_ptr(allocated_space)).unwrap();

        trace!("Allocation: phys {:#X}, virt {:#X}, {} bytes", phys_addr, allocated_space.addr(), allocation_size);

        (phys_addr.as_u64(), NonNull::new(allocated_space).unwrap())
    }

    unsafe fn dma_dealloc(phys_addr: VirtioPhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        let deallocation_size = pages * Size4KiB::SIZE as usize;

        trace!("Deallocation: phys {:#X}, virt {:#X}, {} bytes", phys_addr, vaddr.addr(), deallocation_size);

        unsafe { dealloc(vaddr.as_ptr(), Layout::array::<u8>(deallocation_size).unwrap()); }

        0
    }

    unsafe fn mmio_phys_to_virt(paddr: VirtioPhysAddr, size: usize) -> NonNull<u8> {
        let virt_addr = PHYSICAL_MEMORY_OFFSET.get().unwrap().as_u64() + paddr;

        if translate_addr(VirtAddr::new(virt_addr)).is_none() {
            add_page_table_entry(VirtAddr::new(virt_addr));
        }

        trace!("Phys to virt: {paddr:#X} -> {virt_addr:#X}, {size} bytes");

        NonNull::new(virt_addr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> VirtioPhysAddr {
        let phys_addr = translate_addr(VirtAddr::from_ptr(buffer.as_ptr())).unwrap();

        trace!("Share: virt {:#X} -> phys {:#X}", buffer.addr().get(), phys_addr);

        phys_addr.as_u64()
    }

    unsafe fn unshare(phys_addr: VirtioPhysAddr, buffer: NonNull<[u8]>, _direction: BufferDirection) {
        trace!("Unshare: virt {:#X} -> phys {:#X}", buffer.addr().get(), phys_addr);
    }
}