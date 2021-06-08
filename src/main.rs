use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::CString;
use std::time::Duration;

macro_rules! timespec {
    () => {
        libc::timespec {
            tv_sec: 0 as libc::time_t,
            tv_nsec: 0 as libc::c_long,
        }
    };
}

macro_rules! timeval {
    () => {
        libc::timeval {
            tv_sec: 0 as libc::time_t,
            tv_usec: 0 as libc::suseconds_t,
        }
    };
}

macro_rules! rusage {
    () => {
        libc::rusage {
            ru_utime: timeval!(),
            ru_stime: timeval!(),
            ru_maxrss: 0 as libc::c_long,
            ru_ixrss: 0 as libc::c_long,
            ru_idrss: 0 as libc::c_long,
            ru_isrss: 0 as libc::c_long,
            ru_minflt: 0 as libc::c_long,
            ru_majflt: 0 as libc::c_long,
            ru_nswap: 0 as libc::c_long,
            ru_inblock: 0 as libc::c_long,
            ru_oublock: 0 as libc::c_long,
            ru_msgsnd: 0 as libc::c_long,
            ru_msgrcv: 0 as libc::c_long,
            ru_nsignals: 0 as libc::c_long,
            ru_nvcsw: 0 as libc::c_long,
            ru_nivcsw: 0 as libc::c_long,
        }
    };
}

fn timeval_to_duration(t: libc::timeval) -> Duration {
    Duration::new(t.tv_sec as u64, (t.tv_usec as u32) * 1_000)
}

fn timespec_to_duration(t: libc::timespec) -> Duration {
    Duration::new(t.tv_sec as u64, t.tv_nsec as u32)
}

pub fn utime() -> Duration {
    let mut u = rusage!();
    unsafe {
        libc::getrusage(libc::RUSAGE_SELF, (&mut u) as *mut libc::rusage);
    }

    timeval_to_duration(u.ru_utime)
}

pub fn stime() -> Duration {
    let mut u = rusage!();
    unsafe {
        libc::getrusage(libc::RUSAGE_SELF, (&mut u) as *mut libc::rusage);
    }

    timeval_to_duration(u.ru_stime)
}

// TODO: compilation directive for older versions of macos (< 10.12)
#[cfg(target_os = "macos")]
pub fn rtime_gettimeofday() -> Duration {
    let mut t = timeval!();
    unsafe {
        libc::gettimeofday(
            (&mut t) as *mut libc::timeval,
            std::ptr::null_mut() as *mut libc::c_void,
        );
    }
    timeval_to_duration(t)
}

fn rtime() -> Duration {
    let mut t = timespec!();
    unsafe {
        // TODO: use libc::CLOCK_MONOTONIC with clock_gettime ? should be available on macos 10.12+
        libc::clock_gettime(libc::CLOCK_MONOTONIC, (&mut t) as *mut libc::timespec);
    }

    timespec_to_duration(t)
}

fn wait_for_pid(pid: libc::pid_t) -> libc::rusage {
    let mut u = rusage!();
    let mut status = 0 as libc::c_int;
    let options = 0;

    loop {
        unsafe {
            let r = libc::wait4(
                pid,
                (&mut status) as *mut libc::c_int,
                options,
                (&mut u) as *mut libc::rusage,
            );
            if r == -1 {
                panic!("failed to wait4!");
            } else if r == pid {
                break;
            }
        }
    }

    u
}

// https://stackoverflow.com/a/12480485/5552584

fn main() {
    #[cfg(target_os = "macos")]
    let rtime_fn = {
        use os_info::Version;
        let info = os_info::get();
        let vers = info.version();
        if vers <= &Version::Semantic(10, 12, 0) {
            rtime_gettimeofday
        } else {
            rtime
        }
    };

    #[cfg(not(target_os = "macos"))]
    let rtime_fn = rtime;

    let start = rtime_fn();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            let u = wait_for_pid(child.as_raw());
            println!("real: {:?}", (rtime_fn() - start));
            println!("user: {:?}", (timeval_to_duration(u.ru_utime)));
            println!("sys:  {:?}", (timeval_to_duration(u.ru_stime)));
        }
        Ok(ForkResult::Child) => {
            let cmd = CString::new("sleep").unwrap();
            let _ = execvp(cmd.clone().as_c_str(), &[cmd, CString::new("0.1").unwrap()]);
            panic!("if we got here then something bad happened");
        }
        Err(e) => panic!("failed to fork: {}", e),
    }
}
