use core::arch::x86_64::_rdtsc;
use smoltcp::time::Instant;

// TODO: use real frequency
const CPU_FREQ_HZ: u128 = 2_500_000_000; // 2.5 GHz

pub fn now() -> Instant {
    Instant::from_micros_const((now_ns() / 100_000) as i64)
}

pub fn now_ns() -> u64 {
    let tsc = unsafe { _rdtsc() } as u128;

    // cycles -> nanoseconds
    ((tsc * 1_000_000_000) / CPU_FREQ_HZ) as u64
}