use crate::memory::tables::{MEMORY_REGIONS, PHYSICAL_MEMORY_OFFSET};
use crate::println;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use humansize::{format_size, DECIMAL};
use linked_list_allocator::LockedHeap;
use spin::Mutex;


#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(memory_regions: &MemoryRegions) {
    let mut start = 0;
    let mut size = 0;

    println!("\tMemory regions:");

    for memory_region in memory_regions.iter() {
        if matches!(memory_region.kind, MemoryRegionKind::Usable) {
            let region_size = memory_region.end - memory_region.start;
            if region_size > size {
                start = memory_region.start;
                size = region_size;
            }
        }
    }

    let virt_addr = PHYSICAL_MEMORY_OFFSET.get().unwrap().as_u64() + start;

    unsafe {
        ALLOCATOR.lock().init(virt_addr as *mut u8, size as usize);
    }

    MEMORY_REGIONS.call_once(|| Mutex::new(memory_regions.to_vec()));

    for memory_region in memory_regions.iter() {
        println!(
            "\t\t{:?}: {:#X}..{:#X} ({})",
            memory_region.kind,
            memory_region.start,
            memory_region.end,
            format_size(memory_region.end - memory_region.start, DECIMAL)
        );
    }

    println!("\tHeap size: {}", format_size(size, DECIMAL));
}