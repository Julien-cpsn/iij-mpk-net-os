use crate::memory::tables::PHYSICAL_MEMORY_OFFSET;
use alloc::alloc::dealloc;
use core::alloc::Layout;
use core::fmt::Debug;
use core::num::NonZeroUsize;
use spin::Once;
use x86_64::PhysAddr;
use crate::println;

pub static MMCONFIG_PHYS_BASE: Once<PhysAddr> = Once::new();
pub const MMCONFIG_SIZE: usize = 256 * 32 * 8 * 4096;


#[derive(Clone, Debug)]
pub struct MemoryMapper;

impl accessor::Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, _bytes: usize) -> NonZeroUsize {
        let virt_addr = PHYSICAL_MEMORY_OFFSET.get().unwrap().as_u64() as usize + phys_base;
        
        println!("MMIO phys to virt: {phys_base:#X} -> {virt_addr:#X}");
        
        NonZeroUsize::new(virt_addr).unwrap()
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {
        unsafe { dealloc(virt_base as *mut u8, Layout::array::<u8>(bytes).unwrap()); }
    }
}
