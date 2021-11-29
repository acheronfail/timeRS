use serde::Serialize;
use std::fmt::Display;

use crate::cli::OutputFormat;

pub const NO_DATA: &str = "-";

// TODO: use a builder pattern (probably a crate which offers it) rather than this

#[derive(Debug, Default, Serialize)]
pub struct PreExec {
    #[serde(skip)]
    output_format: OutputFormat,
    // info
    pub cmdline: String,
    pub cpu_count: Option<u32>,
    pub mem_total: Option<u64>,
    pub mem_avail: Option<u64>,
    pub page_size: Option<u64>,
}

impl PreExec {
    pub fn new(output_format: OutputFormat) -> PreExec {
        PreExec {
            output_format,
            ..PreExec::default()
        }
    }
}

impl Display for PreExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.output_format {
            OutputFormat::Json => {
                let json_string = serde_json::to_string(self).expect("failed to serialise");
                writeln!(f, "{}", json_string)?;
            }
            OutputFormat::Standard => {
                writeln!(f, "cmdline:          {}", self.cmdline)?;
                writeln!(
                    f,
                    "cpu_count:        {}",
                    self.cpu_count.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    f,
                    "mem_total:        {}",
                    self.mem_total.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    f,
                    "mem_avail:        {}",
                    self.mem_avail.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    f,
                    "page_size:        {}",
                    self.page_size.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct PostExec {
    #[serde(skip)]
    output_format: OutputFormat,
    // info
    pub exit_code: Option<i32>,
    pub term_signal: Option<i32>,
    pub term_signal_name: Option<String>,
    pub time_real: u128,
    pub time_user: u128,
    pub time_sys: u128,
    pub percent_cpu: f64,
    pub max_rss: u64,
    pub hard_page_faults: i64,
    pub soft_page_faults: i64,
    pub disk_inputs: i64,
    pub disk_outputs: i64,
    pub voluntary_csw: i64,
    pub involuntary_csw: i64,
}

impl PostExec {
    pub fn new(output_format: OutputFormat) -> PostExec {
        PostExec {
            output_format,
            ..PostExec::default()
        }
    }
}

impl Display for PostExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.output_format {
            OutputFormat::Json => {
                let json_string = serde_json::to_string(self).expect("failed to serialise");
                writeln!(f, "{}", json_string)?;
            }
            OutputFormat::Standard => {
                writeln!(
                    f,
                    "exit_code:        {}",
                    self.exit_code.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    f,
                    "term_signal:      {}",
                    self.term_signal.map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(
                    f,
                    "term_signal_name: {}",
                    self.term_signal_name.as_ref().map_or(NO_DATA.into(), |x| x.to_string())
                )?;
                writeln!(f, "time_real:        {}", self.time_real)?;
                writeln!(f, "time_user:        {}", self.time_user)?;
                writeln!(f, "time_sys:         {}", self.time_sys)?;
                writeln!(f, "percent_cpu:      {}", self.percent_cpu)?;
                writeln!(f, "max_rss:          {}", self.max_rss)?;
                writeln!(f, "hard_page_faults: {}", self.hard_page_faults)?;
                writeln!(f, "soft_page_faults: {}", self.soft_page_faults)?;
                writeln!(f, "disk_inputs:      {}", self.disk_inputs)?;
                writeln!(f, "disk_outputs:     {}", self.disk_outputs)?;
                writeln!(f, "voluntary_csw:    {}", self.voluntary_csw)?;
                writeln!(f, "involuntary_csw:  {}", self.involuntary_csw)?;
            }
        }

        Ok(())
    }
}
