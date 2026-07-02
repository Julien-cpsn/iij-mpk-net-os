use core::num::NonZeroUsize;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Once;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use crate::memory::allocator::BOOT_INFO_FRAME_ALLOCATOR;
use crate::memory::tables::MAPPER;
use crate::println;

pub const MMCONFIG_SIZE: usize = 2 * 1024 * 1024; // 2MiB

pub static MMCONFIG_BASE: Once<PhysAddr> = Once::new();
pub const MMIO_VIRT_BASE: VirtAddr = VirtAddr::new(0xFFFF_9200_0000_0000);
static MMIO_CURRENT_OFFSET: AtomicUsize = AtomicUsize::new(0);
pub const DMA_VIRT_BASE: VirtAddr = VirtAddr::new(0xFFFF_9100_0000_0000);
static DMA_VIRT_OFFSET: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct MemoryMapper;

impl accessor::Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        // Round size up to page size
        let page_size = Size4KiB::SIZE as usize;
        let size_rounded = (bytes + page_size - 1) & !(page_size - 1);

        // Allocate a page-aligned region in our MMIO virtual area
        // We must allocate in page multiples so use size_rounded for the offset step.
        let offset = MMIO_CURRENT_OFFSET.fetch_add(size_rounded, Ordering::SeqCst);

        // Align the physical start to the page boundary as well
        let page_offset = phys_base & (page_size - 1);
        let phys_start = phys_base - page_offset;

        let virt_addr = MMIO_VIRT_BASE.as_u64() as usize + offset + page_offset;

        // Page-align the virtual address (it should already be page aligned if START and offsets are)
        let virt_page = Page::containing_address(VirtAddr::new(virt_addr as u64));

        let phys_addr = PhysAddr::new(phys_start as u64);

        // Compute page frames to map: from phys_start .. phys_start + size_rounded - 1
        let start_frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys_addr);
        let end_phys = phys_start + size_rounded - 1;
        let end_frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(PhysAddr::new(end_phys as u64));

        let mut current_virt_page = virt_page;

        // Acquire mapper and frame allocator (your code)
        let mut mapper = MAPPER.get().unwrap().write();
        let mut frame_allocator = BOOT_INFO_FRAME_ALLOCATOR.lock();

        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            // Map each physical frame to the corresponding virtual page
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE | PageTableFlags::WRITE_THROUGH;

            unsafe {
                let map_result = mapper
                    .map_to(current_virt_page, frame, flags, frame_allocator.deref_mut())
                    .expect("Memory mapping failed");

                map_result.flush();
            };

            current_virt_page += 1u64; // next page
        }

        NonZeroUsize::new(virt_addr).expect("virtual addr cannot be zero")
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {
        let virt_addr = VirtAddr::new(virt_base as u64);

        let start_page: Page<Size4KiB> = Page::containing_address(virt_addr);
        let end_page = Page::containing_address(virt_addr + bytes as u64 - 1);

        let mut mapper = MAPPER.get().unwrap().write();

        for page in Page::range(start_page, end_page) {
            let res = mapper.unmap(page);
            res.expect("Unmapping failed").1.flush();
        }
    }
}

impl MemoryMapper {
    pub fn map_dma(&mut self, start: PhysFrame<Size4KiB>, pages: usize) -> VirtAddr {
        let offset = DMA_VIRT_OFFSET.fetch_add(pages * Size4KiB::SIZE as usize, Ordering::Relaxed);

        let virt = DMA_VIRT_BASE + offset as u64;

        let mut mapper = MAPPER.get().unwrap().write();
        let mut allocator = BOOT_INFO_FRAME_ALLOCATOR.lock();

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        for i in 0..pages {
            let page = Page::containing_address(
                virt + (i * Size4KiB::SIZE as usize) as u64,
            );

            let frame = start + i as u64;

            println!(
                "map page {:?} -> {:?}",
                page.start_address(),
                frame.start_address()
            );

            unsafe {
                mapper
                    .map_to(page, frame, flags, allocator.deref_mut())
                    .unwrap()
                    .flush();
            }
        }

        virt
    }
}