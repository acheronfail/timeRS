use crate::ffi::convert::timespec_to_duration;
use lazy_static::lazy_static;
use std::{mem::MaybeUninit, time::Duration};

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
