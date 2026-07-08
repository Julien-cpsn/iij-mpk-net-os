use crate::cpu::interrupt::{breakpoint_handler, double_fault_handler, general_protection_fault_handler, page_fault_handler, syscall_handler, DOUBLE_FAULT_IST_INDEX};
use spin::{LazyLock, Mutex};
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::PrivilegeLevel;

const SYSCALL_VECTOR: u8 = 0x80;

pub static IDT: LazyLock<Mutex<InterruptDescriptorTable>> = LazyLock::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);

    unsafe {
        idt
            .double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    idt[SYSCALL_VECTOR]
        .set_handler_fn(syscall_handler)
        .set_privilege_level(PrivilegeLevel::Ring3);

    Mutex::new(idt)
});

pub fn init_idt() {
    let idt = IDT.lock();
    unsafe { idt.load_unsafe(); }
}
