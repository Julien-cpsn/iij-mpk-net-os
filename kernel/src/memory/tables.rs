use crate::println;
use alloc::vec::Vec;
use bootloader_api::info::MemoryRegion;
use spin::{Mutex, Once, RwLock};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::FrameError;
use x86_64::structures::paging::{OffsetPageTable, PageTable, PhysFrame, Size1GiB, Size2MiB};
use x86_64::{PhysAddr, VirtAddr};

pub static PHYSICAL_MEMORY_OFFSET: Once<VirtAddr> = Once::new();
pub static MAPPER: Once<RwLock<OffsetPageTable<'static>>> = Once::new();
pub static MEMORY_REGIONS: Once<Mutex<Vec<MemoryRegion>>> = Once::new();

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub fn init_memory_mapping(physical_memory_offset: VirtAddr) {
    println!("\tPhysical memory offset: {:#X}", physical_memory_offset);

    let level_4_table = active_level_4_table(physical_memory_offset);
    let offset_page_table = unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) };

    MAPPER.call_once(|| RwLock::new(offset_page_table));
    PHYSICAL_MEMORY_OFFSET.call_once(|| physical_memory_offset);
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

/// Translates the given virtual address to the mapped physical address, or
/// `None` if the address is not mapped.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`.
pub fn translate_addr(addr: VirtAddr) -> Option<PhysAddr> {
    let mapper = MAPPER.get().unwrap().read();
    translate_addr_inner(addr, mapper.phys_offset())
}

/// Private function that is called by `translate_addr`.
///
/// This function is safe to limit the scope of `unsafe` because Rust treats
/// the whole body of unsafe functions as an unsafe block. This function must
/// only be reachable through `unsafe fn` from outside of this module.
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // traverse the multi-level page table
    for (t_index, &index) in table_indexes.iter().enumerate() {
        // convert the frame into a page table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};

        // read the page table entry and update `frame`
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => {
                let phys = match t_index {
                    0 => panic!("Too huge pages not supported"),
                    1 => {
                        let new_frame: PhysFrame<Size1GiB> = PhysFrame::containing_address(entry.addr());
                        new_frame.start_address() + (addr.as_u64()  % 0x40000000)
                    },
                    2 => {
                        let new_frame: PhysFrame<Size2MiB> = PhysFrame::containing_address(entry.addr());
                        new_frame.start_address() + (addr.as_u64() % 0x200000)

                    }
                    _ => unreachable!()
                };

                //crate::println!("trans: {:#X} -> {:#X}", addr, phys);

                return Some(phys)
            },
        };
    }

    // calculate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}