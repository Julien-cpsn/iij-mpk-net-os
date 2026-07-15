use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use crate::cpu::kernel_syscall::syscall::SyscallRegs;
use crate::memory::tables::NEW_T4;


pub fn read_cr3(regs: &mut SyscallRegs) {
    let (level_4_table_frame, _) = Cr3::read();
    
    regs.rax = level_4_table_frame.start_address().as_u64();
}
pub fn update_cr3() {
    unsafe {
        Cr3::update(|old_t4_frame, _| *old_t4_frame = *NEW_T4.get().unwrap());
        tlb::flush_all();
    }
}