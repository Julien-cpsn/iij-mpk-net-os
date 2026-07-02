use spin::{LazyLock, Mutex};
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::cpu::interrupt::{breakpoint_handler, double_fault_handler, page_fault_handler, DOUBLE_FAULT_IST_INDEX};

pub static IDT: LazyLock<Mutex<InterruptDescriptorTable>> = LazyLock::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);

    unsafe {
        idt
            .double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    Mutex::new(idt)
});

pub fn init_idt() {
    let idt = IDT.lock();
    unsafe { idt.load_unsafe(); }
}
