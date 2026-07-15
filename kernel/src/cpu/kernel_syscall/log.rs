use core::slice::from_raw_parts;
use log::{debug, error, info, trace, warn};
use crate::cpu::kernel_syscall::syscall::SyscallRegs;


pub fn log(regs: &mut SyscallRegs) {
    let level = regs.rdx as usize;
    let target = unsafe {
        from_raw_parts(
            regs.r8 as *const u8,
            regs.r9 as usize,
        )
    };

    let slice = unsafe {
        from_raw_parts(
            regs.rdi as *const u8,
            regs.rsi as usize,
        )
    };

    let target = core::str::from_utf8(target).unwrap();

    if let Ok(s) = core::str::from_utf8(slice) {
        match level {
            0 => error!(target: target, "{s}"),
            1 => warn!(target: target, "{s}"),
            2 => info!(target: target, "{s}"),
            3 => debug!(target: target, "{s}"),
            4 => trace!(target: target, "{s}"),
            _ => {}
        }
    }
}