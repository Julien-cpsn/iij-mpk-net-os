use core::arch::x86_64::_rdtsc;
use smoltcp::time::Instant;

pub fn now() -> Instant {
    unsafe { Instant::from_micros_const(_rdtsc() as i64 / 2_500) }
}