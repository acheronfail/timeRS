pub mod mem;

use std::mem::MaybeUninit;
use std::time::Duration;

pub fn cpu_count() -> u32 {
    // SAFETY: TODO
    unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) as u32 }
}

pub fn timeval_to_duration(t: libc::timeval) -> Duration {
    Duration::new(t.tv_sec as u64, (t.tv_usec as u32) * 1_000)
}

pub fn wait_for_pid(pid: libc::pid_t) -> (i32, libc::rusage) {
    let mut usage: MaybeUninit<libc::rusage> = MaybeUninit::uninit();
    let mut status = 0;
    let options = 0;

    loop {
        // SAFETY: TODO
        let r = unsafe {
            libc::wait4(
                pid,
                (&mut status) as *mut libc::c_int,
                options,
                usage.as_mut_ptr(),
            )
        };

        if r == -1 {
            panic!("failed to wait4!");
        } else if r == pid {
            break;
        }
    }

    // SAFETY: we have asserted that the return condition is not an error
    (status, unsafe { usage.assume_init() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn timeval(tv_sec: i64, tv_usec: i32) -> libc::timeval {
        libc::timeval { tv_sec, tv_usec }
    }

    #[test]
    fn test_timeval_to_duration() {
        assert_eq!(timeval_to_duration(timeval(0, 0)), Duration::new(0, 0));
        assert_eq!(timeval_to_duration(timeval(0, 42)), Duration::new(0, 42000));
        assert_eq!(timeval_to_duration(timeval(42, 0)), Duration::new(42, 0));
        assert_eq!(
            timeval_to_duration(timeval(42, 42)),
            Duration::new(42, 42000)
        );
    }
}
