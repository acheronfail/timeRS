mod cli;
mod ffi;

use cli::{Args, TimeFormat};
use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::CString;
use std::time::{Duration, Instant};

/// Returns the milliseconds, with nanoseconds contained in an `f64`.
fn duration_millis_f64(d: Duration) -> f64 {
    (d.as_secs() * 1_000) as f64 + (d.subsec_nanos() as f64 / 1_000_000.0)
}

/// Returns the microseconds, with nanoseconds contained in an `f64`.
fn duration_micros_f64(d: Duration) -> f64 {
    (d.as_secs() * 1_000_000) as f64 + (d.subsec_nanos() as f64 / 1_000.0)
}

/// The lowest form of measurement, as such it does not return an `f64` but a `u64`.
fn duration_nanos_f64(d: Duration) -> u64 {
    (d.as_secs() * 1_000_000_000) + (d.subsec_nanos() as u64)
}

fn main() {
    let args = Args::parse();
    let start = Instant::now();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            let u = ffi::wait_for_pid(child.as_raw());
            let real = start.elapsed();
            let user = ffi::timeval_to_duration(u.ru_utime);
            let sys = ffi::timeval_to_duration(u.ru_stime);

            // TODO: right align measurements
            match args.time_format {
                None | Some(TimeFormat::Normal) => {
                    println!("real: {:?}", real);
                    println!("user: {:?}", user);
                    println!("sys:  {:?}", sys);
                }
                Some(TimeFormat::Seconds) => {
                    println!("real: {}s", real.as_secs_f64());
                    println!("user: {}s", user.as_secs_f64());
                    println!("sys:  {}s", sys.as_secs_f64());
                }
                Some(TimeFormat::Milli) => {
                    println!("real: {}ms", duration_millis_f64(real));
                    println!("user: {}ms", duration_millis_f64(user));
                    println!("sys:  {}ms", duration_millis_f64(sys));
                }
                Some(TimeFormat::Micro) => {
                    println!("real: {}µs", duration_micros_f64(real));
                    println!("user: {}µs", duration_micros_f64(user));
                    println!("sys:  {}µs", duration_micros_f64(sys));
                }
                Some(TimeFormat::Nano) => {
                    println!("real: {}ns", duration_nanos_f64(real));
                    println!("user: {}ns", duration_nanos_f64(user));
                    println!("sys:  {}ns", duration_nanos_f64(sys));
                }
            }
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
