use anyhow::{anyhow, bail, Result};
use std::{cmp, fs};

fn read_number_from_line(s: &str) -> Result<i64> {
    s.chars()
        .skip_while(|c| !c.is_digit(10))
        .take_while(|c| c.is_digit(10))
        .collect::<String>()
        .parse()
        .map_err(|e| anyhow!("{}", e))
}

pub fn memory_available() -> Result<u64> {
    let mut avail = None;
    let mut free = None;
    let mut inactive_file = None;
    let mut active_file = None;
    let mut s_reclaimable = None;
    let f = fs::read_to_string("/proc/meminfo")?;
    for line in f.lines() {
        if line.starts_with("MemAvailable:") {
            avail = Some(read_number_from_line(line)?)
        } else if line.starts_with("MemFree:") {
            free = Some(read_number_from_line(line)?)
        } else if line.starts_with("Inactive(file):") {
            inactive_file = Some(read_number_from_line(line)?)
        } else if line.starts_with("Active(file):") {
            active_file = Some(read_number_from_line(line)?)
        } else if line.starts_with("SReclaimable:") {
            s_reclaimable = Some(read_number_from_line(line)?)
        }
    }

    let check_option = |o| match o {
        Some(x) => Ok(x),
        None => bail!("Failed to parse /proc/meminfo"),
    };

    let mut avail = check_option(avail)?;
    let free = check_option(free)?;
    let mem_file = check_option(inactive_file)? - check_option(active_file)?;
    let s_reclaimable = check_option(s_reclaimable)?;

    // https://github.com/attractivechaos/runlog/blob/f09830c8c5bf71d3451dcbd2fed04fbfcc4be83a/runlog.c#L175
    if avail < 0 {
        let min_free = fs::read_to_string("/proc/sys/vm/min_free_kbytes")?.parse::<i64>()?;
        let low = min_free * 5 / 4;
        let mem_file_low = cmp::max(mem_file / 2, low);
        let s_reclaimable_low = cmp::max(s_reclaimable / 2, low);
        avail = cmp::max(
            free - low + mem_file - mem_file_low + s_reclaimable - s_reclaimable_low,
            0,
        );
    }

    Ok(avail as u64 * 1024)
}
