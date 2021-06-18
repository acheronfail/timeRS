use std::mem::MaybeUninit;
use std::time::Duration;

pub fn timeval_to_duration(t: libc::timeval) -> Duration {
    Duration::new(t.tv_sec as u64, (t.tv_usec as u32) * 1_000)
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
