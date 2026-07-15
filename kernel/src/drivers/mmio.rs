use crate::memory::tables::{add_flags_to_frame, PHYSICAL_MEMORY_OFFSET};
use alloc::alloc::dealloc;
use core::alloc::Layout;
use core::fmt::Debug;
use core::num::NonZeroUsize;
use spin::Once;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::PageTableFlags;
use crate::ktrace;

const TARGET: &str = "MMIO";

pub static MMCONFIG_PHYS_BASE: Once<PhysAddr> = Once::new();
pub const MMCONFIG_SIZE: usize = 256 * 32 * 8 * 4096;


#[derive(Clone, Debug)]
pub struct MemoryMapper;

impl accessor::Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        let virt_addr = PHYSICAL_MEMORY_OFFSET.get().unwrap().as_u64() as usize + phys_base;
        
        ktrace!("MAP phys to virt: {phys_base:#X} -> {virt_addr:#X}, {bytes} bytes");

        add_flags_to_frame(VirtAddr::new(virt_addr as u64), PageTableFlags::USER_ACCESSIBLE, true);

        NonZeroUsize::new(virt_addr).unwrap()
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {
        ktrace!("UNMAP: {virt_base:#X}, {bytes} bytes");
        
        unsafe { dealloc(virt_base as *mut u8, Layout::array::<u8>(bytes).unwrap()); }
    }
}
