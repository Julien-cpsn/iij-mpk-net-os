use crate::kprintln;
use core::arch::x86_64::{__cpuid, _rdtsc};
use core::sync::atomic::{AtomicU64, Ordering};
use smoltcp::time::Instant;


static TSC_HZ: AtomicU64 = AtomicU64::new(0);

pub fn now() -> Instant {
    Instant::from_micros_const((now_ns() / 1_000) as i64)
}

pub fn now_ns() -> u64 {
    let tsc = unsafe { _rdtsc() } as u128;
    let hz = TSC_HZ.load(Ordering::Relaxed) as u128;

    ((tsc * 1_000_000_000) / hz) as u64
}

pub fn kvm_tsc_frequency() -> Option<u64> {
    let max = __cpuid(0x40000000);

    if max.eax < 0x40000010 {
        return None;
    }

    let leaf = __cpuid(0x40000010);

    if leaf.eax == 0 {
        return None;
    }

    // EAX = TSC frequency in kHz
    Some((leaf.eax as u64) * 1000)
}

pub fn init_time() {
    let freq = kvm_tsc_frequency().expect("KVM TSC frequency unavailable");

    kprintln!("\tTSC frequency = {} Hz", freq);

    TSC_HZ.store(freq, Ordering::Relaxed);
}