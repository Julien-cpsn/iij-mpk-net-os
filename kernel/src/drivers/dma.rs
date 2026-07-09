use crate::memory::tables::{translate_addr, PHYSICAL_MEMORY_OFFSET};
use alloc::alloc::{alloc_zeroed, dealloc};
use core::alloc::Layout;
use core::ptr::NonNull;
use virtio_drivers::{BufferDirection, Hal, PhysAddr as VirtioPhysAddr};
use x86_64::structures::paging::{PageSize, Size4KiB};
use x86_64::VirtAddr;
use crate::println;
use crate::memory::tables::add_page_table_entry;

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (VirtioPhysAddr, NonNull<u8>) {
        let allocation_size = pages * Size4KiB::SIZE as usize;

        let allocated_space = unsafe { alloc_zeroed(Layout::array::<u8>(allocation_size).unwrap()) };
        let phys_addr = translate_addr(VirtAddr::from_ptr(allocated_space)).unwrap();

        crate::println!("DMA allocation: {phys_addr:#X}, {allocation_size}");

        (phys_addr.as_u64(), NonNull::new(allocated_space).unwrap())
    }

    unsafe fn dma_dealloc(_paddr: VirtioPhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        let deallocation_size = pages * Size4KiB::SIZE as usize;

        crate::println!("DMA deallocation: {_paddr:#X}, {:#X}, {deallocation_size}", vaddr.addr());

        unsafe { dealloc(vaddr.as_ptr(), Layout::array::<u8>(deallocation_size).unwrap()); }

        0
    }

    unsafe fn mmio_phys_to_virt(paddr: VirtioPhysAddr, _size: usize) -> NonNull<u8> {
        let virt_addr = PHYSICAL_MEMORY_OFFSET.get().unwrap().as_u64() + paddr;
        add_page_table_entry(VirtAddr::new(virt_addr));
        let phys_addr = translate_addr(VirtAddr::new(virt_addr)).unwrap();
        crate::println!("DMA MMIO phys to virt: {paddr:#X} -> {virt_addr:#X}");

        NonNull::new(virt_addr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> VirtioPhysAddr {
        let phys_addr = translate_addr(VirtAddr::from_ptr(buffer.as_ptr())).unwrap();

        println!("DMA share: {:#X} -> {:#X}", buffer.addr().get(), phys_addr);

        phys_addr.as_u64()
    }

    unsafe fn unshare(_paddr: VirtioPhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        crate::println!("DMA unshare: {_paddr:#X}");
    }
}
