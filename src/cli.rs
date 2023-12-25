use clap::AppSettings::TrailingVarArg;
use clap::ArgSettings::{AllowHyphenValues, Required};
use clap::{crate_authors, crate_version, ArgEnum, Parser};

#[derive(Debug, ArgEnum, Clone, Copy)]
pub enum TimeFormat {
    Normal,
    Seconds,
    Milli,
    Nano,
    Micro,
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
    // TODO: JSON output
}

impl Args {
    pub fn parse() -> Args {
        <Args as Parser>::parse()
    }
}
