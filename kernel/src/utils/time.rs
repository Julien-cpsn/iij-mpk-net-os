use core::arch::x86_64::_rdtsc;
use smoltcp::time::Instant;

// TODO: use real frequency
const CPU_FREQ_HZ: u64 = 2_500_000_000; // 2.5 GHz

pub fn now() -> Instant {
    unsafe { Instant::from_micros_const(_rdtsc() as i64 / 2_500) }
}

pub fn now_ns() -> u64 {
    let tsc = unsafe { _rdtsc() };

    // cycles -> nanoseconds
    (tsc * 1_000_000_000) / CPU_FREQ_HZ
}