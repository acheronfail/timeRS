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
fn duration_nanos_u64(d: Duration) -> u64 {
    (d.as_secs() * 1_000_000_000) + (d.subsec_nanos() as u64)
}

fn main() {
    let args = Args::parse();
    let start = Instant::now();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            let usage = ffi::wait_for_pid(child.as_raw());
            let real = start.elapsed();

            let fmt = match args.time_format {
                None | Some(TimeFormat::Normal) => |d: Duration| {
                    if d.as_secs() > 0 {
                        format!("{:.9}s ", d.as_secs_f64())
                    } else {
                        let nanos = d.subsec_nanos();
                        if nanos >= 1_000_000 {
                            format!("{:.6}ms", duration_millis_f64(d))
                        } else if nanos >= 1_000 {
                            format!("{:.3}µs", duration_micros_f64(d))
                        } else {
                            format!("{}ns", duration_nanos_u64(d))
                        }
                    }
                },
                Some(TimeFormat::Seconds) => |d: Duration| format!("{:.9}s", d.as_secs_f64()),
                Some(TimeFormat::Milli) => |d: Duration| format!("{:.6}ms", duration_millis_f64(d)),
                Some(TimeFormat::Micro) => |d: Duration| format!("{:.3}µs", duration_micros_f64(d)),
                Some(TimeFormat::Nano) => |d: Duration| format!("{}ns", duration_nanos_u64(d)),
            };

            let real = fmt(real);
            let user = fmt(ffi::timeval_to_duration(usage.ru_utime));
            let sys = fmt(ffi::timeval_to_duration(usage.ru_stime));

            // SAFETY: None is only returned if the iterator is empty.
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
