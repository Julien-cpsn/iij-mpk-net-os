use crate::drivers::mmio::MemoryMapper;
use crate::memory::allocator::BOOT_INFO_FRAME_ALLOCATOR;
use crate::memory::tables::translate_addr;
use accessor::Mapper;
use core::ptr::NonNull;
use virtio_drivers::{BufferDirection, Hal, PhysAddr as VirtioPhysAddr};
use x86_64::{PhysAddr, VirtAddr};


pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection, _access_platform: bool) -> (VirtioPhysAddr, NonNull<u8>) {
        let start = BOOT_INFO_FRAME_ALLOCATOR
            .lock()
            .allocate_contiguous(pages)
            .expect("Out of DMA memory");

        let phys = start.start_address();

        let virt = MemoryMapper.map_dma(start, pages);
        let translated = translate_addr(virt).unwrap();

        assert_eq!(
            translated.as_u64(),
            phys.as_u64(),
            "DMA mapping mismatch!"
        );

        (phys.as_u64(), NonNull::new(virt.as_mut_ptr()).unwrap())
    }

    unsafe fn dma_dealloc(_paddr: VirtioPhysAddr, _vaddr: NonNull<u8>, _pages: usize, _access_platform: bool) -> i32 {
        // no-op for now
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: VirtioPhysAddr, size: usize) -> NonNull<u8> {
        let mut mapper = MemoryMapper;

        let virt = unsafe {
            mapper.map(
                PhysAddr::new(paddr).as_u64() as usize,
                size,
            )
        };

        NonNull::new(virt.get() as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection, _access_platform: bool) -> VirtioPhysAddr {
        translate_addr(VirtAddr::from_ptr(buffer.as_ptr())).unwrap().as_u64()
    }

    unsafe fn unshare(_paddr: VirtioPhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection, _access_platform: bool) {
    }
}