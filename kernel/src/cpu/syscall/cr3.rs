use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use crate::apps::user::benchmark::NEW_T4;

pub fn update_cr3() {
    unsafe {
        Cr3::update(|old_t4_frame, _| *old_t4_frame = *NEW_T4.get().unwrap());
        tlb::flush_all();
    }
}