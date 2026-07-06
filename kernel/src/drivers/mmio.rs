use crate::memory::allocator::BOOT_INFO_FRAME_ALLOCATOR;
use crate::memory::tables::MAPPER;
use crate::{print, println};
use core::fmt::Debug;
use core::num::NonZeroUsize;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Once;
use x86_64::structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub static MMCONFIG_PHYS_BASE: Once<PhysAddr> = Once::new();
pub const MMIO_REGION_START: VirtAddr = VirtAddr::new(0xFFFF_9200_0000_0000);
static MMIO_CURRENT_OFFSET: AtomicUsize = AtomicUsize::new(0);
pub const MMCONFIG_SIZE: usize = 256 * 32 * 8 * 4096;


#[derive(Clone, Debug)]
pub struct MemoryMapper;

impl accessor::Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        print!("Mapping physical address {:#X} to {:#X}...", phys_base, phys_base + bytes);

        let offset = MMIO_CURRENT_OFFSET.fetch_add(bytes, Ordering::Relaxed);
        let virt_addr = MMIO_REGION_START + offset as u64;

        let phys_addr = PhysAddr::new(phys_base as u64);

        let start_frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys_addr);
        let end_frame = PhysFrame::containing_address(phys_addr + bytes as u64 - 1);

        let mut current_virt = virt_addr;

        let mut mapper = MAPPER.get().unwrap().write();
        let mut frame_allocator = BOOT_INFO_FRAME_ALLOCATOR.lock();

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE | PageTableFlags::WRITE_THROUGH;

        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            let page = Page::containing_address(current_virt);

            //println!("Mapping {:#?} {:#?} {:?}", page, frame, flags);

            unsafe {
                let map_result = mapper
                    .map_to(page, frame, flags, frame_allocator.deref_mut())
                    .expect("Memory mapping failed");

                map_result.flush();
            };

            current_virt += Size4KiB::SIZE; // next page
        }

        println!("done");

        NonZeroUsize::new(virt_addr.as_u64() as usize).expect("virtual addr cannot be zero")
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
