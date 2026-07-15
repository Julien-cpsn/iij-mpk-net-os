use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::PageTableFlags;
use crate::cpu::kernel_syscall::syscall::SyscallRegs;

pub fn translate_addr(regs: &mut SyscallRegs) {
    let virt_addr = VirtAddr::new(regs.rdi);

    let result = crate::memory::tables::translate_addr(virt_addr).unwrap_or(PhysAddr::zero());

    regs.rax = result.as_u64();
}

pub fn add_page_table_entry(regs: &mut SyscallRegs) {
    let virt_addr = VirtAddr::new(regs.rdi);

    crate::memory::tables::add_page_table_entry(virt_addr);
}

pub fn add_flags_to_frame(regs: &mut SyscallRegs) {
    let virt_addr = VirtAddr::new(regs.rdi);
    let flags = PageTableFlags::from_bits(regs.rsi).unwrap();
    let recursive = regs.r8 != 0;

    crate::memory::tables::add_flags_to_frame(virt_addr, flags, recursive);
}