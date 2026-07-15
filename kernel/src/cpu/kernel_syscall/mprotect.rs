use crate::cpu::protection::pk::set_frame_protection_key;
use crate::memory::tables::find_page_table_entry;
use x86_64::VirtAddr;
use crate::cpu::kernel_syscall::syscall::SyscallRegs;
use crate::kdebug;

const TARGET: &str = "MPROTECT";

pub fn pkey_mprotect(regs: &SyscallRegs) {
    let virt_addr = VirtAddr::new(regs.rdi);
    let length = regs.rsi;
    let pkey = regs.rdx as u8;

    assert!(pkey < 16);

    let mut addr_offset = 0;

    while addr_offset < length {
        let iter_addr = virt_addr + addr_offset;
        let (entry, entry_size) = find_page_table_entry(iter_addr).unwrap();

        set_frame_protection_key(entry, pkey);

        addr_offset += entry_size;
    }
    
    kdebug!("Virt address {:#X} with {} bytes, key set to {}", virt_addr, length, pkey);
}
