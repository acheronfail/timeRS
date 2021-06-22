mod cli;
mod ffi;
mod fmt;

use bytesize::ByteSize;
use cli::Args;
use flexi_logger::{colored_default_format, Logger};
use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::{CStr, CString};
use std::process;
use std::time::Instant;

// TODO: JSON output

fn main() {
    Logger::try_with_env_or_str("info")
        .expect("Failed to initialise logger")
        .format(colored_default_format)
        .start()
        .expect("Failed to initialise logger");

    let args = Args::parse();
    log::trace!("{:#?}", args);
    log::info!("cmdline:          {}", args.command_line.join(" "));

    // CPU information
    log::info!("cpu count:        {}", ffi::cpu_count());

    // System memory information
    let fmt_bytes = |b| format!("{} ({})", b, ByteSize(b).to_string_as(true));
    let total = ffi::mem::memory_total();
    let page_size = ffi::mem::page_size();
    log::info!("mem_total:        {}", fmt_bytes(total));
    match ffi::mem::memory_available() {
        Ok(avail) => log::info!("mem_avail:        {}", fmt_bytes(avail)),
        Err(e) => log::error!("{}", e)
    }
    log::info!("page_size:        {}", fmt_bytes(page_size));

    let c_args = args
        .command_line
        .into_iter()
        // SAFETY: Is there a way to pass null bytes as arguments on the command line?
        .map(|s| CString::new(s).unwrap())
        .collect::<Vec<_>>();

    // NOTE: REAL_TIMER START: immediately before forking the process
    let start = Instant::now();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            // This log line may have an effect on short-lived programs real time, since there's a chance
            // that the forked process will exit before this log completes
            #[cfg(debug_assertions)]
            log::info!("pid:              {}", child);

            let (status, usage) = ffi::wait_for_pid(child.as_raw());
            let real = start.elapsed();
            // NOTE: REAL_TIMER END: immediately after forked process has terminated

            log::trace!("{:#?}", usage);

            let user = ffi::timeval_to_duration(usage.ru_utime);
            let sys = ffi::timeval_to_duration(usage.ru_stime);
            let pct_cpu = 100.0 * (user.as_secs_f64() + sys.as_secs_f64()) / real.as_secs_f64();

            let fmt = fmt::duration_formatter(args.time_format);
            let real = fmt(real);
            let user = fmt(user);
            let sys = fmt(sys);

            // Exit code
            let mut return_code = 0;
            if libc::WIFEXITED(status) {
                let exit_code = libc::WEXITSTATUS(status);
                log::info!("exit code:        {}", exit_code);
                return_code = exit_code;
            } else {
                log::info!("exit code:        -");
            }
            // Signal number
            if libc::WIFSIGNALED(status) {
                let signal = libc::WTERMSIG(status);
                // SAFETY: the string returned by `strsignal` does not need to be freed, on Linux systems it should only
                // be used until the next call to `strsignal`, but we only call it once here anyway
                let signal_name = unsafe { CStr::from_ptr(libc::strsignal(signal)) };
                let signal_name = signal_name.to_str().unwrap();
                // Seems that macOS's implementation of `strsignal` includes the signal number
                #[cfg(target_os = "macos")]
                log::info!("term_signal:      {}", signal_name);
                #[cfg(not(target_os = "macos"))]
                log::info!("term_signal:      {} ({})", signal_name, signal);
                return_code = signal;
            } else {
                log::info!("term_signal:      -");
            }

            // Timers
            // SAFETY: `None` is only returned if the iterator is empty
            let len = *[real.len(), user.len(), sys.len()].iter().max().unwrap() - 1;
            log::info!("real:             {:>width$}", real, width = len);
            log::info!("user:             {:>width$}", user, width = len);
            log::info!("sys:              {:>width$}", sys, width = len);
            log::info!("percent_cpu:      {:.4}%", pct_cpu);
            // Maximum resident set size (approximate maximum memory used by the process)
            // NOTE: On Linux this value is in kilobytes
            #[cfg(target_os = "linux")]
            let rss = fmt_bytes(usage.ru_maxrss as u64 * 1024);
            #[cfg(not(target_os = "linux"))]
            let rss = fmt_bytes(usage.ru_maxrss as u64);
            log::info!("max_rss:          {}", rss);
            // Page faults
            log::info!("hard_page_faults: {}", usage.ru_majflt);
            log::info!("soft_page_faults: {}", usage.ru_minflt);
            // Number of time the filesystem had to perform real IO (doesn't account for caches)
            log::info!("disk_inputs:      {}", usage.ru_inblock);
            log::info!("disk_outputs:     {}", usage.ru_oublock);
            // Context switches
            log::info!("voluntary_csw:    {}", usage.ru_nvcsw);
            log::info!("involuntary_csw:  {}", usage.ru_nivcsw);

            // Exit with either the status code or the signal number of the forked process
            process::exit(return_code);
        }
        Ok(ForkResult::Child) => {
            let err = execvp(&c_args[0], &c_args).unwrap_err();
            eprintln!("{}", err);
        }
        Err(e) => panic!("Failed to fork: {}", e),
    }
}
