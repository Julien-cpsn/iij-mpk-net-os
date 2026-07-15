use crate::cpu::kernel_syscall::syscall::SyscallRegs;
use crate::drivers::mmio::MemoryMapper;
use accessor::Mapper;

pub fn allocate(regs: &mut SyscallRegs) {
    let address = regs.rdi as usize;
    let size = regs.rsi as usize;

    let mut mapper = MemoryMapper;
    let virt_addr = unsafe { mapper.map(address, size) };

    regs.rax = virt_addr.get() as u64;
}