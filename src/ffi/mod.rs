pub mod mem;

use anyhow::{bail, Result};
use nix::errno::{errno, Errno};
use std::mem::MaybeUninit;
use std::time::Duration;

pub fn cpu_count() -> Result<u32> {
    sysconf(libc::_SC_NPROCESSORS_ONLN).map(|x| x as u32)
}

pub fn timeval_to_duration(t: libc::timeval) -> Duration {
    Duration::new(t.tv_sec as u64, (t.tv_usec as u32) * 1_000)
}

pub fn sysconf(var: libc::c_int) -> Result<i64> {
    // SAFETY: we're checking the return code and errno, should be good enough for our use cases
    let raw = unsafe {
        Errno::clear();
        libc::sysconf(var as libc::c_int)
    };

    if raw == -1 && errno() != 0 {
        bail!("Call to sysconf failed, errno: {}", Errno::last());
    }

    Ok(raw)
}

pub fn wait_for_pid(pid: libc::pid_t) -> Result<(i32, libc::rusage)> {
    let mut usage: MaybeUninit<libc::rusage> = MaybeUninit::uninit();
    let mut status = 0;
    let options = 0;

    loop {
        let r = unsafe {
            Errno::clear();
            libc::wait4(
                pid,
                (&mut status) as *mut libc::c_int,
                options,
                usage.as_mut_ptr(),
            )
        };

        if r == -1 {
            bail!("Call to wait4 failed, errno: {}", Errno::last());
        }

        // The child process we were waiting for (pid) terminated
        if r == pid {
            break;
        }
    }

    // SAFETY: we have asserted that the return condition is not an error
    Ok((status, unsafe { usage.assume_init() }))
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
