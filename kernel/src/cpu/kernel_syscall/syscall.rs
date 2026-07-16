use core::arch::naked_asm;
use x86_64::structures::idt::InterruptStackFrame;
use crate::cpu::kernel_syscall::allocate::allocate;
use crate::cpu::kernel_syscall::cr3::{read_cr3, update_cr3};
use crate::cpu::kernel_syscall::exit::exit;
use crate::cpu::kernel_syscall::log::log;
use crate::cpu::kernel_syscall::mprotect::pkey_mprotect;
use crate::cpu::kernel_syscall::pkru::{read_pkru_key, write_pkru_key};
use crate::cpu::kernel_syscall::print::print;
use crate::cpu::kernel_syscall::tables::{add_flags_to_frame, add_page_table_entry, translate_addr};
use crate::kwarn;

const TARGET: &str = "SYSCALL";

#[repr(C)]
pub struct SyscallRegs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

#[unsafe(naked)]
pub extern "C" fn syscall_entry(_interrupt_stack_frame: InterruptStackFrame) {
    naked_asm!(
        "
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push rbp
        push rdi
        push rsi
        push rdx
        push rcx
        push rbx
        push rax

        mov rdi, rsp
        call {dispatch}

        pop rax
        pop rbx
        pop rcx
        pop rdx
        pop rsi
        pop rdi
        pop rbp
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15

        iretq
        ",
        dispatch = sym syscall_dispatch,
    );
}

extern "C" fn syscall_dispatch(regs: &mut SyscallRegs) {
    let syscall_id = regs.rax;

    if syscall_id != 11 && syscall_id != 10 {
        crate::ktrace!("Syscall: {syscall_id}");
    }

    match syscall_id {
        // General purpose
        0 => exit(regs),

        // Printing
        10 => print(regs),
        11 => log(regs),

        // Cr3 / protection
        20 => read_cr3(regs),
        21 => update_cr3(),
        22 => read_pkru_key(regs),
        23 => write_pkru_key(regs),
        24 => pkey_mprotect(regs),

        // Allocation
        30 => allocate(regs),

        // Tables
        40 => translate_addr(regs),
        41 => add_page_table_entry(regs),
        42 => add_flags_to_frame(regs),

        _ => kwarn!("Unknown syscall {}", regs.rax),
    }
}