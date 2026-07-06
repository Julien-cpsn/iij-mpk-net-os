use accessor::Mapper;
use crate::memory::tables::{translate_addr, PHYSICAL_MEMORY_OFFSET};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};
use virtio_drivers::{BufferDirection, Hal, PhysAddr as VirtioPhysAddr};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{PageSize, Size4KiB};
use crate::drivers::mmio::MemoryMapper;

const DMA_REGION_START: PhysAddr = PhysAddr::new(0x100_000_000 + 0x8_000);
static DMA_CURRENT_OFFSET: AtomicUsize = AtomicUsize::new(0);

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(_pages: usize, _direction: BufferDirection) -> (VirtioPhysAddr, NonNull<u8>) {
        let offset = DMA_CURRENT_OFFSET.fetch_add(Size4KiB::SIZE as usize, Ordering::Relaxed);
        let phys_addr = DMA_REGION_START.as_u64() + offset as u64;

        let virt_addr = PHYSICAL_MEMORY_OFFSET.get().unwrap().as_u64() + phys_addr;

        (phys_addr, NonNull::new(virt_addr as *mut u8).unwrap())
    }

    unsafe fn dma_dealloc(_paddr: VirtioPhysAddr, _vaddr: NonNull<u8>, _pages: usize) -> i32 {
        // no-op for now
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: VirtioPhysAddr, size: usize) -> NonNull<u8> {
        let mut mapper = MemoryMapper;

        let virt = unsafe {
            mapper.map(PhysAddr::new(paddr).as_u64() as usize, size)
        };

        NonNull::new(virt.get() as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> VirtioPhysAddr {
        translate_addr(VirtAddr::from_ptr(buffer.as_ptr())).unwrap().as_u64()
    }

    unsafe fn unshare(_paddr: VirtioPhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
    }
}