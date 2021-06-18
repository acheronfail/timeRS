mod ffi;

use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::CString;

fn main() {
    let start = ::std::time::Instant::now();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            let u = ffi::wait_for_pid(child.as_raw());
            println!("real: {:?}", start.elapsed());
            println!("user: {:?}", ffi::timeval_to_duration(u.ru_utime));
            println!("sys:  {:?}", ffi::timeval_to_duration(u.ru_stime));
        }
        Ok(ForkResult::Child) => {
            let cmd = CString::new("sleep").unwrap();
            let _ = execvp(cmd.clone().as_c_str(), &[cmd, CString::new("1").unwrap()]);
            panic!("if we got here then something bad happened");
        }
        Err(e) => panic!("failed to fork: {}", e),
    }
}
