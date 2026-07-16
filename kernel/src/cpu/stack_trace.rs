use alloc::format;
use alloc::string::String;
use core::arch::asm;

#[repr(C)]
struct StackFrame {
    previous: *const StackFrame,
    return_address: usize,
}

pub fn get_stacktrace() -> String {
    let mut output = String::new();
    let rbp: usize;

    unsafe {
        asm!("mov {}, rbp", out(reg) rbp);
    }

    let mut frame = rbp as *const StackFrame;

    for i in 0..64 {
        if frame.is_null() {
            break;
        }

        unsafe {
            let ret = (*frame).return_address;
            let prev = (*frame).previous;

            output += &format!("#{:<2} {:#018x}\n", i, ret);

            // Sanity checks to avoid walking into garbage.
            if prev.is_null() || (prev as usize) <= (frame as usize) || (prev as usize & 0xf) != 0
            {
                break;
            }

            frame = prev;
        }
    }

    output
}