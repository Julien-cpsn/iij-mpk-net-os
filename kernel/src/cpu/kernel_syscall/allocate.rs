use crate::cpu::kernel_syscall::syscall::SyscallRegs;
use crate::drivers::mmio::MemoryMapper;
use accessor::Mapper;
use alloc::alloc::alloc_zeroed;
use core::alloc::Layout;
use core::mem::Alignment;
use x86_64::VirtAddr;
use x86_64::structures::paging::{PageSize, Size4KiB};

pub fn allocate(regs: &mut SyscallRegs) {
    let size = regs.rsi as usize;
    let virt_addr = match regs.rdi {
        0 => {
            let layout = Layout::from_size_alignment(
                size,
                Alignment::new(Size4KiB::SIZE as usize).unwrap()
            )
                .unwrap();
            
            let virt_addr = unsafe { alloc_zeroed(layout) };

            VirtAddr::from_ptr(virt_addr)
        }
        address => {
            let mut mapper = MemoryMapper;
            let virt_addr = unsafe { mapper.map(address as usize, size) };

            VirtAddr::new(virt_addr.get() as u64)
        }
    };

    regs.rax = virt_addr.as_u64();
}