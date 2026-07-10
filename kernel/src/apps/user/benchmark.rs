use crate::apps::user::syscalls::{update_cr3, write_pkru};
use crate::cpu::protection::pk::PkPermission;
use crate::println;
use crate::utils::time::now_ns;
use spin::Once;
use x86_64::structures::paging::PhysFrame;

const GOOLOG_TARGET: &str = "BENCHMARK";

pub static NEW_T4: Once<PhysFrame> = Once::new();

pub fn benchmark() {
    const DURATION: u64 = 1_000_000_000;

    println!("-- Time based benchmark ({DURATION} ns)");
    println!("Cr3 update + TLB flush");
    let count = time_based_bench(|| { update_cr3() }, DURATION);
    println!("Result: {count} calls");
    println!("1 call = {} ns", DURATION / count);
    println!();
    println!("WRPKRU update");
    let count = time_based_bench(|| { write_pkru(8, PkPermission::NoAccess) }, DURATION);
    println!("Result: {count} calls");
    println!("1 call = {} ns", DURATION / count);

    println!();
    println!();

    const COUNT: u64 = 100_000;
    println!("-- Counter based benchmark ({COUNT} iterations)");
    println!("Cr3 update + TLB flush");
    let duration = counter_based_bench(|| { update_cr3() }, COUNT);
    println!("Result: {duration} ns");
    println!("1 call = {} ns", duration / COUNT);
    println!();
    println!("WRPKRU update");
    let duration = counter_based_bench(|| { write_pkru(8, PkPermission::NoAccess) }, COUNT);
    println!("Result: {duration} ns");
    println!("1 call = {} ns", duration / COUNT);
}

fn time_based_bench(f: impl Fn(), duration: u64) -> u64 {
    let mut counter = 0;
    let start = now_ns();
    let mut elapsed = 0;

    while elapsed < duration {
        elapsed = now_ns() - start;
        f();
        counter += 1;
    }

    counter
}

fn counter_based_bench(f: impl Fn(), count: u64) -> u64 {
    let start = now_ns();

    for _ in 0..count {
        f();
    }

    let end = now_ns();

    end - start
}