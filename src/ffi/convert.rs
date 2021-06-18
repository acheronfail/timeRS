use std::time::Duration;

pub fn timeval_to_duration(t: libc::timeval) -> Duration {
    Duration::new(t.tv_sec as u64, (t.tv_usec as u32) * 1_000)
}

pub fn timespec_to_duration(t: libc::timespec) -> Duration {
    Duration::new(t.tv_sec as u64, t.tv_nsec as u32)
}