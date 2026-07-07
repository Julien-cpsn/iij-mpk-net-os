use crate::println;
use core::arch::x86_64::__cpuid_count;
use strum_macros::FromRepr;
use x86_64::registers::control::{Cr4, Cr4Flags};
use x86_64::structures::paging::page_table::PageTableEntry;

const PKRU_MASK: u32 = (1 << 2) - 1;

#[repr(u8)]
#[derive(Debug, PartialEq, FromRepr)]
enum PkruPermission {
    ReadWrite = 0b00,
    ReadOnly = 0b10,
    NoAccess = 0b01,
    NoAccessBis = 0b11,
}

pub fn init_pku() {
    unsafe {
        Cr4::update(|cr4| *cr4 |= Cr4Flags::PROTECTION_KEY_USER);
    }

    if !has_pku() {
        panic!("PKU initialization failed");
    }

    println!("PKRU: {}", read_pkru());
    println!("Key 3: {:?}", read_pkru_key(3));
    println!("Setting PKRU key 3 to no access");
    write_pkru_key(3, PkruPermission::NoAccess);
    println!("Key 3: {:?}", read_pkru_key(3));
}

pub fn get_frame_protection_key(entry: &PageTableEntry) -> u32 {
    let flags = entry.flags().bits();
    let mask = (1 << 3) - 1;
    let protection_key = (flags >> 59) & mask;

    protection_key as u32
}

fn has_pku() -> bool {
    let r = __cpuid_count(7, 0);
    (r.ecx & (1 << 3)) != 0
}

fn read_pkru_key(key: u32) -> PkruPermission {
    assert!(key < 16);

    let pkru = read_pkru();

    let result = ((pkru >> (key * 2)) & PKRU_MASK) as u8;

    PkruPermission::from_repr(result).unwrap()
}

fn write_pkru_key(key: u32, new_permission: PkruPermission) {
    assert!(key < 16);

    let shift = key * 2;
    let mask = PKRU_MASK << shift;

    let mut pkru = read_pkru();
    pkru &= !mask;
    pkru |= (new_permission as u32) << shift;

    write_pkru(pkru);
}

#[inline]
fn read_pkru() -> u32 {
    unsafe { rdpkru() }
}

#[inline]
fn write_pkru(pkru: u32) {
    unsafe { wrpkru(pkru) }
}

#[allow(improper_ctypes)]
unsafe extern "unadjusted" {
    #[link_name = "llvm.x86.rdpkru"]
    fn rdpkru() -> u32;
    #[link_name = "llvm.x86.wrpkru"]
    fn wrpkru(pkru: u32);
}