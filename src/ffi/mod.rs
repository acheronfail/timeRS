use lazy_static::lazy_static;
use std::{mem::MaybeUninit, time::Duration};

pub fn timeval_to_duration(t: libc::timeval) -> Duration {
    Duration::new(t.tv_sec as u64, (t.tv_usec as u32) * 1_000)
}

pub fn timespec_to_duration(t: libc::timespec) -> Duration {
    Duration::new(t.tv_sec as u64, t.tv_nsec as u32)
}

#[allow(unused)]
pub fn utime() -> Duration {
    let mut u: MaybeUninit<libc::rusage> = MaybeUninit::uninit();
    unsafe {
        libc::getrusage(libc::RUSAGE_SELF, u.as_mut_ptr());
    }

    timeval_to_duration(unsafe { u.assume_init() }.ru_utime)
}

#[allow(unused)]
pub fn stime() -> Duration {
    let mut u: MaybeUninit<libc::rusage> = MaybeUninit::uninit();
    unsafe {
        libc::getrusage(libc::RUSAGE_SELF, u.as_mut_ptr());
    }

    timeval_to_duration(unsafe { u.assume_init() }.ru_stime)
}

lazy_static! {
    static ref RTIME_FN: fn() -> Duration = {
        #[cfg(target_os = "macos")]
        let rtime_fn = {
            use os_info::Version;
            let info = os_info::get();
            let vers = info.version();
            return if vers <= &Version::Semantic(10, 12, 0) {
                rtime_gettimeofday
            } else {
                rtime_clock_gettime
            };
        };

        #[cfg(not(target_os = "macos"))]
        rtime_clock_gettime
    };
}

#[cfg(target_os = "macos")]
fn rtime_gettimeofday() -> Duration {
    let mut t: MaybeUninit<libc::timeval> = MaybeUninit::uninit();
    unsafe {
        libc::gettimeofday(t.as_mut_ptr(), std::ptr::null_mut() as *mut libc::c_void);
    }

    timeval_to_duration(unsafe { t.assume_init() })
}

fn rtime_clock_gettime() -> Duration {
    let mut t: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
    unsafe {
        libc::clock_gettime(libc::CLOCK_MONOTONIC, t.as_mut_ptr());
    }

    timespec_to_duration(unsafe { t.assume_init() })
}

pub fn rtime() -> Duration {
    RTIME_FN()
}

pub fn wait_for_pid(pid: libc::pid_t) -> libc::rusage {
    let mut u: MaybeUninit<libc::rusage> = MaybeUninit::uninit();
    let mut status = 0;
    let options = 0;

    loop {
        unsafe {
            let r = libc::wait4(
                pid,
                (&mut status) as *mut libc::c_int,
                options,
                u.as_mut_ptr(),
            );
            if r == -1 {
                panic!("failed to wait4!");
            } else if r == pid {
                break;
            }
        }
    }

    unsafe { u.assume_init() }
}
