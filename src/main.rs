mod cli;
mod ffi;
mod fmt;

use cli::Args;
use flexi_logger::{colored_default_format, Logger};
use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::{CStr, CString};
use std::time::Instant;
use std::process;

fn main() {
    Logger::try_with_env_or_str("info")
        .expect("Failed to initialise logger")
        .format(colored_default_format)
        .start()
        .expect("Failed to initialise logger");

    let args = Args::parse();
    log::trace!("{:?}", args);
    log::info!("cmdline:   {}", args.command_line.join(" "));

    let c_args = args
        .command_line
        .into_iter()
        // SAFETY: Is there a way to pass null bytes as arguments on the command line?
        .map(|s| CString::new(s).unwrap())
        .collect::<Vec<_>>();

    let start = Instant::now();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            log::info!("pid:       {}", child);
            let (status, usage) = ffi::wait_for_pid(child.as_raw());
            let real = start.elapsed();

            let fmt = fmt::duration_formatter(args.time_format);
            let real = fmt(real);
            let user = fmt(ffi::timeval_to_duration(usage.ru_utime));
            let sys = fmt(ffi::timeval_to_duration(usage.ru_stime));

            let mut return_code = 0;
            if libc::WIFEXITED(status) {
                let exit_code = libc::WEXITSTATUS(status);
                log::info!("exit code: {}", exit_code);
                return_code = exit_code;
            }
            if libc::WIFSIGNALED(status) {
                let signal = libc::WTERMSIG(status);
                // SAFETY: TODO
                let signal_name = unsafe { CStr::from_ptr(libc::strsignal(signal)) };
                let signal_name = signal_name.to_str().unwrap();
                // Seems that macOS's implementation of `strsignal` includes the signal number
                #[cfg(target_os = "macos")]
                log::info!("signal:    {}", signal_name);
                #[cfg(not(target_os = "macos"))]
                log::info!("signal:    {} ({})", signal_name, signal);
                return_code = signal;
            }

            // SAFETY: `None` is only returned if the iterator is empty.
            let len = *[real.len(), user.len(), sys.len()].iter().max().unwrap() - 1;
            log::info!("real:      {:>width$}", real, width = len);
            log::info!("user:      {:>width$}", user, width = len);
            log::info!("sys:       {:>width$}", sys, width = len);

            process::exit(return_code);
        }
        Ok(ForkResult::Child) => {
            // TODO: pass input?
            let err = execvp(&c_args[0], &c_args).unwrap_err();
            eprintln!("{}", err);
        }
        Err(e) => panic!("Failed to fork: {}", e),
    }
}
