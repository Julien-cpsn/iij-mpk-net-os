use bootloader_api::info::MemoryRegionKind;
use linked_list_allocator::LockedHeap;
use spin::Mutex;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB};
use crate::memory::tables::MEMORY_REGIONS;
use crate::println;

pub static BOOT_INFO_FRAME_ALLOCATOR: Mutex<BootInfoFrameAllocator> = Mutex::new(BootInfoFrameAllocator(0));

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 0x10000; // 64K

static mut HEAP: [u64; HEAP_SIZE / 8] = [0; HEAP_SIZE / 8];

pub struct BootInfoFrameAllocator(usize);

unsafe impl<T: PageSize> FrameAllocator<T> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<T>> {
        let memory_regions = MEMORY_REGIONS.get().unwrap().lock();

        let usable_regions = memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable && r.start >= 0x0100_000);

        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.start..r.end);

        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(T::SIZE as usize));

        // create `PhysFrame` types from the start addresses
        let frame = frame_addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
            .nth(self.0);

        self.0 += 1;

        frame
    }
}

impl FrameDeallocator<Size4KiB> for BootInfoFrameAllocator {
    unsafe fn deallocate_frame(&mut self, _frame: PhysFrame<Size4KiB>) {

    }
}

impl BootInfoFrameAllocator {
    pub fn allocate_contiguous(&mut self, pages: usize) -> Option<PhysFrame<Size4KiB>> {
        let memory_regions = MEMORY_REGIONS.get().unwrap().lock();

        let mut skipped = self.0;

        for region in memory_regions.iter().filter(|r| r.kind == MemoryRegionKind::Usable && r.start >= 0x0100_000) {
            let region_pages = ((region.end - region.start) / Size4KiB::SIZE) as usize;

            if skipped >= region_pages {
                skipped -= region_pages;
                continue;
            }

            if skipped + pages > region_pages {
                skipped = 0;
                continue;
            }

            let start = region.start + skipped as u64 * Size4KiB::SIZE;

            self.0 += pages;

            println!(
                "DMA {} pages phys={:#x}",
                pages,
                start
            );

            return Some(PhysFrame::containing_address(PhysAddr::new(start)));
        }

        None
    }
}

pub fn init_heap() {
    unsafe {
        let heap_start = &raw mut HEAP as *mut u8;
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }
}