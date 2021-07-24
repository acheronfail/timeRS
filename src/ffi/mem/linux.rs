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
    let mut m_avail = None;
    let mut m_free = None;
    let mut m_inactive_file = None;
    let mut m_active_file = None;
    let mut m_s_reclaimable = None;
    let f = fs::read_to_string("/proc/meminfo")?;
    for line in f.lines() {
        if line.starts_with("MemAvailable:") {
            m_avail = Some(read_number_from_line(line)?)
        } else if line.starts_with("MemFree:") {
            m_free = Some(read_number_from_line(line)?)
        } else if line.starts_with("Inactive(file):") {
            m_inactive_file = Some(read_number_from_line(line)?)
        } else if line.starts_with("Active(file):") {
            m_active_file = Some(read_number_from_line(line)?)
        } else if line.starts_with("SReclaimable:") {
            m_s_reclaimable = Some(read_number_from_line(line)?)
        }
    }

    // TODO: more error information here could be useful (use a macro to stringify parameter, etc)
    let check_option = |o| match o {
        Some(x) => Ok(x),
        None => bail!("Failed to parse /proc/meminfo"),
    };

    let mut m_avail = check_option(m_avail)?;
    let m_free = check_option(m_free)?;
    let m_file = check_option(m_inactive_file)? - check_option(m_active_file)?;
    let m_s_reclaimable = check_option(m_s_reclaimable)?;

    // https://github.com/attractivechaos/runlog/blob/f09830c8c5bf71d3451dcbd2fed04fbfcc4be83a/runlog.c#L175
    if m_avail < 0 {
        let min_free = fs::read_to_string("/proc/sys/vm/min_free_kbytes")?.parse::<i64>()?;
        let low = min_free * 5 / 4;
        let m_file_low = cmp::max(m_file / 2, low);
        let m_s_reclaimable_low = cmp::max(m_s_reclaimable / 2, low);
        m_avail = cmp::max(
            m_free - low + m_file - m_file_low + m_s_reclaimable - m_s_reclaimable_low,
            0,
        );
    }

    Ok(m_avail as u64 * 1024)
}
