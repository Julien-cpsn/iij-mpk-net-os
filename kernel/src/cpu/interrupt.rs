use crate::kprintln;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use crate::memory::tables::find_page_table_entry;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    kprintln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    kprintln!("EXCEPTION: PAGE FAULT");
    kprintln!("Accessed Address: {:?}", Cr2::read());
    kprintln!("Error Code: {:?}", error_code);
    kprintln!("{:?}", stack_frame);

    let rip = stack_frame.instruction_pointer;

    if let Some((entry, size)) = find_page_table_entry(rip) {
        kprintln!("RIP entry size = {}", size);
        kprintln!("RIP entry flags = {:?}", entry.flags());
    }
    
    hlt_loop();
}

pub extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    kprintln!("EXCEPTION: GENERAL PROTECTION");
    kprintln!("Error code: {}", error_code);
    kprintln!("{:#?}", stack_frame);
    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}