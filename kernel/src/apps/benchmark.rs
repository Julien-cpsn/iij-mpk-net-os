use crate::cpu::protection::pk::PkPermission;
use crate::cpu::protection::pkru::write_pkru_key;
use crate::println;
use crate::utils::time::now;
use smoltcp::time::Duration;
use spin::Once;
use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PhysFrame;

static NEW_T4: Once<PhysFrame> = Once::new();

pub fn benchmark() {
    let (t4_frame, _) = Cr3::read();
    NEW_T4.call_once(|| t4_frame.clone());

    println!("Cr3 update + TLB flush");
    let count = bench(|| { update_cr3() });
    println!("Result: {count} calls");
    println!("1 call = {} ns", 1_000_000_000 / count);

    println!();

    println!("WRPKRU update");
    let count = bench(|| { write_pkru_key(8, PkPermission::NoAccess) });
    println!("Result: {count} calls");
    println!("1 call = {} ns", 1_000_000_000 / count);
}

fn bench(f: impl Fn()) -> u32 {
    let mut counter = 0;
    let bench_duration = now();
    let mut elapsed = Duration::from_millis(0);

    while elapsed < Duration::from_secs(1) {
        elapsed = bench_duration - now();
        f();
        counter += 1;
    }

    counter
}

fn update_cr3() {
    unsafe {
        Cr3::update(|old_t4_frame, _| *old_t4_frame = *NEW_T4.get().unwrap());
        tlb::flush_all();
    }
}