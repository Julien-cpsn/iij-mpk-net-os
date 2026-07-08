use crate::println;
use alloc::vec::Vec;
use bootloader_api::info::MemoryRegion;
use spin::{Mutex, Once, RwLock};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::{FrameError, PageTableEntry};
use x86_64::structures::paging::{OffsetPageTable, PageSize, PageTable, PageTableFlags, PhysFrame, Size1GiB, Size2MiB, Size4KiB};
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
    let (entry, entry_size) = find_page_table_entry(addr)?;

    let phys_addr = match entry.frame() {
        Ok(frame) => frame.start_address() + u64::from(addr.page_offset()),
        Err(FrameError::FrameNotPresent) => return None,
        Err(FrameError::HugeFrame) => {
            let new_frame = match entry_size {
                Size1GiB::SIZE => PhysFrame::<Size1GiB>::containing_address(PhysAddr::new(addr.as_u64())).start_address(),
                Size2MiB::SIZE => PhysFrame::<Size2MiB>::containing_address(PhysAddr::new(addr.as_u64())).start_address(),
                _ => unreachable!()
            };

            // Adjust alignment
            PhysAddr::new(new_frame.as_u64() + (addr.as_u64() % entry_size))
        }
    };

    Some(phys_addr)
}


pub fn find_page_table_entry(addr: VirtAddr) -> Option<(&'static mut PageTableEntry, u64)> {
    let physical_memory_offset = PHYSICAL_MEMORY_OFFSET.get().unwrap();

    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    
    let mut frame = level_4_table_frame;
    let mut entry = None;
    let mut page_size = None;

    // traverse the multi-level page table
    for (t_index, &index) in table_indexes.iter().enumerate() {
        // convert the frame into a page table reference
        let virt = physical_memory_offset.as_u64() + frame.start_address().as_u64();
        let table_ptr = virt as *mut PageTable;
        let table = unsafe {&mut *table_ptr};

        // read the page table entry and update `frame`
        entry = Some(&mut table[index]);

        frame = match entry.as_ref().unwrap().frame() {
            Ok(frame) => {
                page_size = Some(Size4KiB::SIZE);
                frame
            },
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => {
                match t_index {
                    0 => panic!("Too huge pages not supported"),
                    1 => {
                        page_size = Some(Size1GiB::SIZE);
                        break
                    },
                    2 => {
                        page_size = Some(Size2MiB::SIZE);
                        break
                    },
                    _ => unreachable!()
                };
            },
        };
    }

    Some((entry.unwrap(), page_size.unwrap()))
}

#[allow(unused)]
pub fn addr_frame_set_or_flags(addr: VirtAddr, flags: PageTableFlags, recursive: bool) -> Option<()> {
    let physical_memory_offset = PHYSICAL_MEMORY_OFFSET.get().unwrap();

    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];

    let mut frame = level_4_table_frame;

    // traverse the multi-level page table
    for (t_index, &index) in table_indexes.iter().enumerate() {
        // convert the frame into a page table reference
        let virt = physical_memory_offset.as_u64() + frame.start_address().as_u64();
        let table_ptr = virt as *mut PageTable;
        let table = unsafe { &mut *table_ptr };

        // read the page table entry and update `frame`
        let entry = &mut table[index];

        if recursive || t_index == 3 {
            entry.set_flags(entry.flags() | flags);
        }

        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => break,
        };
    }

    Some(())
}