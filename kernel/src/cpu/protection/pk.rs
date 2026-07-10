use core::arch::x86_64::__cpuid_count;
use goolog::trace;
use strum_macros::{Display, FromRepr};
use x86_64::registers::control::{Cr4, Cr4Flags};
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::PageTableFlags;


const GOOLOG_TARGET: &str = "PK";

const PAGE_TABLE_ENTRY_PROTECTION_KEY_MASK: u64 = (1 << (62 + 1 - PKEY_SHIFT)) - 1;
const PKEY_SHIFT: u64 = 59;
const PKEY_MASK: u64 = PAGE_TABLE_ENTRY_PROTECTION_KEY_MASK << PKEY_SHIFT;

#[repr(u8)]
#[derive(Debug, PartialEq, FromRepr, Display)]
pub enum PkPermission {
    ReadWrite = 0b00,
    ReadOnly = 0b10,
    NoAccess = 0b01,
    NoAccessBis = 0b11,
}

pub const PK_MASK: u32 = (1 << 2) - 1;

pub fn init_pk() {
    unsafe {
        Cr4::update(|cr4| *cr4 |= Cr4Flags::PROTECTION_KEY_USER /*| Cr4Flags::PROTECTION_KEY_SUPERVISOR*/);
    }

    if !has_pku() {
        panic!("PK initialization failed");
    }
}

fn has_pku() -> bool {
    let r = __cpuid_count(7, 0);
    (r.ecx & (1 << 3)) != 0
}

pub fn get_frame_protection_key(entry: &PageTableEntry) -> u8 {
    let flags = entry.flags().bits();
    let protection_key = (flags >> PKEY_SHIFT) & PAGE_TABLE_ENTRY_PROTECTION_KEY_MASK;

    protection_key as u8
}

pub fn set_frame_protection_key(entry: &mut PageTableEntry, pkey: u8) {
    assert!(pkey < 16);

    trace!("Entry at physical address {:#X}, key set to {}", entry.addr().as_u64(), pkey);

    let mut flags = entry.flags().bits();
    flags &= !PKEY_MASK;
    flags |= (pkey as u64) << PKEY_SHIFT;

    entry.set_flags(PageTableFlags::from_bits_truncate(flags));
}