use core::slice::from_raw_parts;
use crate::cpu::kernel_syscall::syscall::SyscallRegs;
use crate::kprint;

pub fn print(regs: &mut SyscallRegs) {
    let slice = unsafe {
        from_raw_parts(
            regs.rdi as *const u8,
            regs.rsi as usize,
        )
    };

    if let Ok(s) = core::str::from_utf8(slice) {
        kprint!("{s}")
    }
}