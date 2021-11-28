mod cli;
mod ffi;
mod fmt;

use anyhow::Result;
use bytesize::ByteSize;
use cli::{Args, OutputFormat};
use flexi_logger::{colored_default_format, Logger};
use nix::unistd::{execvp, fork, ForkResult};
use serde::Serialize;
use std::ffi::{CStr, CString};
use std::fmt::Write;
use std::process;
use std::time::Instant;

const NO_DATA: &str = "-";

#[derive(Debug, Default, Serialize)]
struct PreExec {
    cmdline: String,
    cpu_count: Option<u32>,
    mem_total: Option<u64>,
    mem_avail: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, Default, Serialize)]
struct PostExec {
    exit_code: Option<i32>,
    term_signal: Option<i32>,
    term_signal_name: Option<String>,
    time_real: u128,
    time_user: u128,
    time_sys: u128,
    percent_cpu: f64,
    max_rss: u64,
    hard_page_faults: i64,
    soft_page_faults: i64,
    disk_inputs: i64,
    disk_outputs: i64,
    voluntary_csw: i64,
    involuntary_csw: i64,
}

impl PreExec {
    pub fn to_string(&self, format: OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Json => Ok(serde_json::to_string(self)?),
            OutputFormat::Standard => {
                let mut out = String::new();
                writeln!(&mut out, "cmdline:          {}", self.cmdline)?;
                writeln!(
                    &mut out,
                    "cpu_count:        {}",
                    self.cpu_count.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    &mut out,
                    "mem_total:        {}",
                    self.mem_total.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    &mut out,
                    "mem_avail:        {}",
                    self.mem_avail.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    &mut out,
                    "page_size:        {}",
                    self.page_size.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                Ok(out)
            }
        }
    }
}

impl PostExec {
    pub fn to_string(&self, format: OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Json => Ok(serde_json::to_string(self)?),
            OutputFormat::Standard => {
                let mut out = String::new();
                writeln!(
                    &mut out,
                    "exit_code:        {}",
                    self.exit_code.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    &mut out,
                    "term_signal:      {}",
                    self.term_signal.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    &mut out,
                    "term_signal_name: {}",
                    self.term_signal_name.as_ref().map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(&mut out, "time_real:        {}", self.time_real)?;
                writeln!(&mut out, "time_user:        {}", self.time_user)?;
                writeln!(&mut out, "time_sys:         {}", self.time_sys)?;
                writeln!(&mut out, "percent_cpu:      {}", self.percent_cpu)?;
                writeln!(&mut out, "max_rss:          {}", self.max_rss)?;
                writeln!(&mut out, "hard_page_faults: {}", self.hard_page_faults)?;
                writeln!(&mut out, "soft_page_faults: {}", self.soft_page_faults)?;
                writeln!(&mut out, "disk_inputs:      {}", self.disk_inputs)?;
                writeln!(&mut out, "disk_outputs:     {}", self.disk_outputs)?;
                writeln!(&mut out, "voluntary_csw:    {}", self.voluntary_csw)?;
                writeln!(&mut out, "involuntary_csw:  {}", self.involuntary_csw)?;
                Ok(out)
            }
        }
    }
}

fn main() {
    Logger::try_with_env()
        .expect("Failed to initialise logger")
        .format(colored_default_format)
        .start()
        .expect("Failed to initialise logger");

    let args = Args::parse();
    let mut pre_exec = PreExec::default();

    pre_exec.cmdline = args.command_line.join(" ");
    pre_exec.cpu_count = ffi::cpu_count().ok();
    pre_exec.mem_total = ffi::mem::memory_total().ok();
    pre_exec.mem_avail = ffi::mem::memory_available().ok();
    pre_exec.page_size = ffi::mem::page_size().ok();
    println!("{}", pre_exec.to_string(args.format.unwrap()).unwrap());

    log::trace!("{:#?}", args);
    log::info!("cmdline:          {}", pre_exec.cmdline);

    // CPU information
    log::info!(
        "cpu_count:        {}",
        pre_exec.cpu_count.map_or(NO_DATA.into(), |n| n.to_string())
    );

    // System memory information
    let fmt_bytes = |b| format!("{} ({})", b, ByteSize(b).to_string_as(true));
    let fmt_res = |r: Option<u64>| r.map_or(NO_DATA.into(), |n| fmt_bytes(n));

    log::info!("mem_total:        {}", fmt_res(pre_exec.mem_total));
    log::info!("mem_avail:        {}", fmt_res(pre_exec.mem_avail));
    log::info!("page_size:        {}", fmt_res(pre_exec.page_size));

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

            let (status, usage) = ffi::wait_for_pid(child.as_raw()).expect("Failed waiting for child");
            let real = start.elapsed();
            // NOTE: REAL_TIMER END: immediately after forked process has terminated

            log::trace!("{:#?}", usage);

            let user = ffi::timeval_to_duration(usage.ru_utime);
            let sys = ffi::timeval_to_duration(usage.ru_stime);
            let pct_cpu = 100.0 * (user.as_secs_f64() + sys.as_secs_f64()) / real.as_secs_f64();
            // NOTE: On Linux this value is in kilobytes
            #[cfg(target_os = "linux")]
            let rss = usage.ru_maxrss as u64 * 1024;
            #[cfg(not(target_os = "linux"))]
            let rss = usage.ru_maxrss as u64;

            let mut post_exec = PostExec::default();
            post_exec.time_real = real.as_nanos();
            post_exec.time_user = user.as_nanos();
            post_exec.time_sys = sys.as_nanos();
            post_exec.percent_cpu = pct_cpu;
            post_exec.max_rss = rss;
            post_exec.hard_page_faults = usage.ru_majflt;
            post_exec.soft_page_faults = usage.ru_minflt;
            post_exec.disk_inputs = usage.ru_inblock;
            post_exec.disk_outputs = usage.ru_oublock;
            post_exec.voluntary_csw = usage.ru_nvcsw;
            post_exec.involuntary_csw = usage.ru_nivcsw;

            let fmt = fmt::duration_formatter(args.time_format);
            let real = fmt(real);
            let user = fmt(user);
            let sys = fmt(sys);

            // Exit code
            if libc::WIFEXITED(status) {
                let exit_code = libc::WEXITSTATUS(status);
                log::info!("exit code:        {}", exit_code);
                post_exec.exit_code = Some(exit_code);
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
                post_exec.term_signal = Some(signal);
                post_exec.term_signal_name = Some(signal_name.to_string());
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
            log::info!("max_rss:          {}", fmt_bytes(rss));
            // Page faults
            log::info!("hard_page_faults: {}", usage.ru_majflt);
            log::info!("soft_page_faults: {}", usage.ru_minflt);
            // Number of time the filesystem had to perform real IO (doesn't account for caches)
            log::info!("disk_inputs:      {}", usage.ru_inblock);
            log::info!("disk_outputs:     {}", usage.ru_oublock);
            // Context switches
            log::info!("voluntary_csw:    {}", usage.ru_nvcsw);
            log::info!("involuntary_csw:  {}", usage.ru_nivcsw);

            println!("{}", post_exec.to_string(args.format.unwrap()).unwrap());

            // Exit with either the status code or the signal number of the forked process
            process::exit(post_exec.exit_code.unwrap_or(0));
        }
        Ok(ForkResult::Child) => {
            let err = execvp(&c_args[0], &c_args).unwrap_err();
            eprintln!("{}", err);
        }
        Err(e) => panic!("Failed to fork: {}", e),
    }
}
