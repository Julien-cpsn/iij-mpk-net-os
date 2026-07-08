use core::arch::asm;

pub fn sys_print(ptr: *const u8, len: usize) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 1usize,
            in("rdi") ptr,
            in("rsi") len,
            options(nostack),
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn user_entry() -> ! {
    // Page fault here (PROTECTION_VIOLATION | USER_MODE | INSTRUCTION_FETCH)
    static MSG: [u8; 19] = *b"Hello from Ring 3!\n";

    sys_print(MSG.as_ptr(), MSG.len());

    loop { core::hint::spin_loop(); }
}