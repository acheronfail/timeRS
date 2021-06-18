use clap::AppSettings::{ColoredHelp, TrailingVarArg};
use clap::ArgSettings::{AllowEmptyValues, AllowHyphenValues};
use clap::{Clap, crate_authors, crate_version};

#[derive(Debug, Clap)]
pub enum TimeFormat {
    Normal,
    Seconds,
    Milli,
    Nano,
    Micro
}

#[derive(Clap, Debug)]
#[clap(
  version = crate_version!(),
  author = crate_authors!(),
  setting = ColoredHelp,
  setting = TrailingVarArg,
)]
pub struct Args {
    #[clap(short = 't', long = "time", arg_enum)]
    pub time_format: Option<TimeFormat>,

    #[clap(setting = AllowHyphenValues, setting = AllowEmptyValues)]
    pub args: Vec<String>,
}

impl Args {
    pub fn parse() -> Args {
        <Args as Clap>::parse()
    }
}