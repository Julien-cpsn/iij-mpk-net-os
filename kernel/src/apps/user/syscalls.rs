use core::arch::asm;
use x86_64::VirtAddr;
use crate::cpu::protection::pk::PkPermission;

#[inline(always)]
pub fn print(ptr: *const u8, len: usize) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0usize,
            in("rdi") ptr,
            in("rsi") len,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn pkey_mprotect(virt_addr: VirtAddr, length: usize, pkey: u8) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 10usize,
            in("rdi") virt_addr.as_u64(),
            in("rsi") length as u64,
            in("rdx") pkey as u64,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn update_cr3() {
    unsafe {
        asm!(
        "int 0x80",
        in("rax") 11usize,
        options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn read_pkru(pkey: u8) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 20usize,
            in("rdx") pkey as u64,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn write_pkru(pkey: u8, permission: PkPermission) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 21usize,
            in("rdx") pkey as u64,
            in("rdi") permission as u8 as u64,
            options(nostack, preserves_flags),
        );
    }
}