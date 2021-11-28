use clap::AppSettings::{TrailingVarArg};
use clap::ArgSettings::{AllowHyphenValues, Required};
use clap::{ArgEnum, Parser, crate_authors, crate_version};

#[derive(Debug, ArgEnum, Clone, Copy)]
pub enum TimeFormat {
    Normal,
    Seconds,
    Milli,
    Nano,
    Micro
}

#[derive(Debug, ArgEnum, Clone, Copy)]
pub enum OutputFormat {
    Standard,
    Json
}

#[derive(Parser, Debug)]
#[clap(
  version = crate_version!(),
  author = crate_authors!(),
  setting = TrailingVarArg,
)]
pub struct Args {
    #[clap(short = 't', long = "time", arg_enum)]
    pub time_format: Option<TimeFormat>,

    #[clap(setting = AllowHyphenValues, setting = Required)]
    pub command_line: Vec<String>,

    #[clap(short = 'f', long = "format", arg_enum)]
    pub format: Option<OutputFormat>
}

impl Args {
    pub fn parse() -> Args {
        let mut parsed = <Args as Parser>::parse();
        if parsed.format.is_none() {
            parsed.format = Some(OutputFormat::Standard);
        }

        parsed
    }
}