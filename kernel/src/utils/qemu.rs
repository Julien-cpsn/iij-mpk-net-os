use strum_macros::FromRepr;
use x86_64::instructions::nop;
use x86_64::instructions::port::Port;
use crate::{kerror, kinfo};

const TARGET: &str = "QEMU";

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    match exit_code {
        QemuExitCode::Success => kinfo!("Success, exiting..."),
        QemuExitCode::Failed => kerror!("Failed, exiting..."),
    }

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}