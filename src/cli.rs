use anyhow::{bail, Result};
use lexopt::Parser;
use std::{env, ffi::OsString, process};

fn print_help() {
    println!(
        "{}",
        format!(
            r#"
{crate_name} {crate_version}
{crate_authors}

Project home page: {crate_homepage}

USAGE:
    {bin} [OPTIONS] [--] <COMMAND_LINE>...

OPTIONS:
    -t, --time <TIME_FORMAT>    [possible values: normal, seconds, milli, micro, nano]
    -h, --help                  Print help information
    -V, --version               Print version information

EXAMPLES:
    {bin} -- cat some/file
    {bin} --time nano -- sh -c 'echo "do something"'

    "#,
            bin = env!("CARGO_BIN_NAME"),
            crate_name = env!("CARGO_PKG_NAME"),
            crate_version = env!("CARGO_PKG_VERSION"),
            crate_homepage = env!("CARGO_PKG_HOMEPAGE"),
            crate_authors = env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>().join("\n"),
        )
        .trim(),
    );
}

#[derive(Debug, Clone, Copy)]
pub enum TimeFormat {
    Normal,
    Seconds,
    Milli,
    Micro,
    Nano,
}

impl ToString for TimeFormat {
    fn to_string(&self) -> String {
        match self {
            Self::Normal => "normal".into(),
            Self::Seconds => "seconds".into(),
            Self::Milli => "milli".into(),
            Self::Micro => "micro".into(),
            Self::Nano => "nano".into(),
        }
    }
}

impl From<String> for TimeFormat {
    fn from(value: String) -> Self {
        match value.as_str() {
            "normal" => Self::Normal,
            "seconds" => Self::Seconds,
            "milli" => Self::Milli,
            "micro" => Self::Micro,
            "nano" => Self::Nano,
            _ => {
                eprintln!(
                    "Unrecognised time format: '{value}', defaulting to {}",
                    Self::Normal.to_string()
                );
                Self::Normal
            }
        }
    }
}

#[derive(Debug)]
pub struct Args {
    pub time_format: Option<TimeFormat>,
    // TODO: JSON output
    pub args: Vec<OsString>,
}

impl Args {
    pub fn parse() -> Result<Args> {
        use lexopt::prelude::*;

        let mut time_format = None;
        let mut command_line = vec![];

        let mut parser = Parser::from_env();
        while let Some(arg) = parser.next()? {
            match arg {
                Short('t') | Long("time") if command_line.is_empty() => {
                    time_format = Some(parser.value()?.string()?.into())
                }
                Short('h') | Long("help") if command_line.is_empty() => {
                    print_help();
                    process::exit(0);
                }
                Short('v') | Long("version") if command_line.is_empty() => {
                    println!(
                        "{crate_name} {crate_version}",
                        crate_name = env!("CARGO_PKG_NAME"),
                        crate_version = env!("CARGO_PKG_VERSION")
                    );
                    process::exit(0);
                }
                Short(_) | Long(_) => command_line.push(parser.value()?),
                Value(val) => {
                    command_line.push(val);
                    command_line.extend(parser.raw_args()?);
                }
            }
        }

        if command_line.is_empty() {
            bail!("no command given");
        }

        Ok(Args {
            time_format,
            args: command_line,
        })
    }
}
