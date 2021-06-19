mod cli;
mod ffi;
mod fmt;

use cli::Args;
use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::CString;
use std::time::Instant;

fn main() {
    let args = Args::parse();
    let start = Instant::now();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            let usage = ffi::wait_for_pid(child.as_raw());
            let real = start.elapsed();

            let fmt = fmt::duration_formatter(args.time_format);
            let real = fmt(real);
            let user = fmt(ffi::timeval_to_duration(usage.ru_utime));
            let sys = fmt(ffi::timeval_to_duration(usage.ru_stime));

            // SAFETY: `None` is only returned if the iterator is empty.
            let len = *[real.len(), user.len(), sys.len()].iter().max().unwrap();
            println!("real: {:>width$}", real, width = len);
            println!("user: {:>width$}", user, width = len);
            println!("sys:  {:>width$}", sys, width = len);
        }
        Ok(ForkResult::Child) => {
            // TODO: use a logging lib rather than just printing
            eprintln!("program: {}", args.command_line.join(" "));

            let c_args = args
                .command_line
                .into_iter()
                // SAFETY: Is there a way to pass null bytes as arguments on the command line?
                .map(|s| CString::new(s).unwrap())
                .collect::<Vec<_>>();

            match execvp(&c_args[0], &c_args) {
                Ok(_) => {}
                Err(e) => panic!("Failed to exec child: {}", e),
            }
        }
        Err(e) => panic!("failed to fork: {}", e),
    }
}
