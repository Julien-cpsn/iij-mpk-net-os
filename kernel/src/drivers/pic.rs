use pic8259::ChainedPics;
use spin::{Mutex, Once};

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PIC: Once<Mutex<ChainedPics>> = Once::new();

pub fn init_pic() {
    PIC.call_once(|| Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }));
    let mut pic = PIC.get().unwrap().lock();
    unsafe { pic.initialize() };
}