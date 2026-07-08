use core::arch::naked_asm;
use crate::println;

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
pub extern "C" fn syscall_entry() {
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

pub fn syscall_dispatch(regs: &mut SyscallRegs) {
    match regs.rax {
        1 => {
            let slice = unsafe {
                core::slice::from_raw_parts(
                    regs.rdi as *const u8,
                    regs.rsi as usize,
                )
            };

            if let Ok(s) = core::str::from_utf8(slice) {
                println!("{}", s);
            }
        }

        _ => println!("Unknown syscall {}", regs.rax),
    }
}