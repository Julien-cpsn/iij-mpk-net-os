use crate::cpu::protection::pk::PkPermission;
use crate::utils::qemu::QemuExitCode;
use core::arch::asm;
use x86_64::structures::paging::{PageTableFlags, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

#[inline(always)]
pub fn exit(exit_code: QemuExitCode) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0usize,
            in("rdi") exit_code as u64,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn print(ptr: *const u8, len: usize) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 10usize,
            in("rdi") ptr,
            in("rsi") len,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn log(target: &str, level: usize, ptr: *const u8, len: usize) {
    unsafe {
        asm!(
        "int 0x80",
        in("rax") 11usize,
        in("r8") target.as_ptr(),
        in("r9") target.len(),
        in("rdi") ptr,
        in("rsi") len,
        in("rdx") level,
        options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn read_cr3() -> PhysFrame {
    let mut ret: u64 = 20;

    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") ret,
        options(nostack, preserves_flags),
        );
    }

    PhysFrame::containing_address(PhysAddr::new(ret))
}

#[inline(always)]
pub fn update_cr3() {
    unsafe {
        asm!(
        "int 0x80",
        in("rax") 21usize,
        options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn read_pkru(pkey: u8) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 22usize,
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
            in("rax") 23usize,
            in("rdx") pkey as u64,
            in("rdi") permission as u8 as u64,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn pkey_mprotect(virt_addr: VirtAddr, length: usize, pkey: u8) {
    unsafe {
        asm!(
        "int 0x80",
        in("rax") 24usize,
        in("rdi") virt_addr.as_u64(),
        in("rsi") length as u64,
        in("rdx") pkey as u64,
        options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn allocate(address: Option<PhysAddr>, size: usize) -> VirtAddr {
    let mut ret: u64 = 30;

    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") ret,
        in("rdi") address.unwrap_or(PhysAddr::zero()).as_u64(),
        in("rsi") size as u64,
        options(nostack, preserves_flags),
        );
    }

    VirtAddr::new(ret)
}

#[inline(always)]
pub fn translate_addr(virt_addr: VirtAddr) -> Option<PhysAddr> {
    let mut ret: u64 = 40;

    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") ret,
        in("rdi") virt_addr.as_u64(),
        options(nostack, preserves_flags),
        );
    }

    if ret == 0 {
        None
    }
    else {
        Some(PhysAddr::new(ret))
    }
}

#[inline(always)]
pub fn add_page_table_entry(virt_addr: VirtAddr) {
    unsafe {
        asm!(
        "int 0x80",
        in("rax") 41usize,
        in("rdi") virt_addr.as_u64(),
        options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub fn add_flags_to_frame(virt_addr: VirtAddr, flags: PageTableFlags, recursive: bool) {
    unsafe {
        asm!(
        "int 0x80",
        in("rax") 42usize,
        in("rdi") virt_addr.as_u64(),
        in("rsi") flags.bits(),
        in("r8") recursive as u64,
        options(nostack, preserves_flags),
        );
    }
}