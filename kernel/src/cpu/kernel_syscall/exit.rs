use crate::cpu::kernel_syscall::syscall::SyscallRegs;
use crate::utils::qemu::{exit_qemu, QemuExitCode};

pub fn exit(regs: &SyscallRegs) {
    let exit_code = QemuExitCode::from_repr(regs.rdi as u32).unwrap();
    exit_qemu(exit_code);
}