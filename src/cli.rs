use std::num::{NonZeroU16, NonZeroU64, NonZeroUsize};
use std::str::FromStr;

use clap::{ArgEnum, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// The host to connect to
    #[clap(default_value = "localhost")]
    pub host: String,
    /// Port number(s) to check (defaults to scanning all ports)
    ///
    /// Format: `22`, `80,8080`, `1..1024` or `10-1024`
    #[clap(
        multiple_occurrences = true,
        use_value_delimiter = true,
        default_value = "-"
    )]
    pub port: Vec<CliPort>,
    /// Show closed ports (defaults to false)
    #[clap(short = 'a', long)]
    pub all: bool,
    /// Timeout for port checks (ms).
    #[clap(short = 't', long, default_value = "2000")]
    pub timeout: NonZeroU64,
    /// Number of retries per port
    #[clap(short = 'r', long, default_value = "2")]
    pub retries: NonZeroUsize,
    /// Set the number of threads to use (defaults to the number of available CPU cores)
    #[clap(short = 'j', long, name = "NUM")]
    pub threads: Option<NonZeroUsize>,
    /// Inspect ports, showing extended information, if any (defaults to false)
    #[clap(short = 'i', long)]
    pub inspect: bool,
    /// Specify when to use colored output
    ///
    /// Defaults to true if an interactive terminal is detected
    #[clap(arg_enum, short = 'c', long, default_value_t = CliColors::Auto)]
    pub color: CliColors,
}

#[derive(ArgEnum, Eq, Copy, Clone, Debug, PartialEq)]
pub enum CliColors {
    Auto,
    #[clap(alias("a"), alias("y"))]
    Always,
    #[clap(alias("n"))]
    Never,
}

impl CliColors {
    pub fn paint(&self, msg: &str, color: &str) -> String {
        let should_paint = match self {
            CliColors::Auto => atty::is(atty::Stream::Stdout),
            CliColors::Always => true,
            CliColors::Never => false,
        };
        if should_paint {
            format!("{}{}\x1b[0m", color, msg)
        } else {
            msg.to_string()
        }
    }
}

#[derive(Debug)]
pub struct CliPort(pub Vec<super::Port>);

impl FromStr for CliPort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // support ranges like "80..8888", "80-8888" or "-" and ".." for all ports
        let ports = if let Some((l, r)) = s.split_once("..").or_else(|| s.split_once("-")) {
            let l = if l.is_empty() {
                1
            } else {
                l.parse::<NonZeroU16>().map_err(|e| e.to_string())?.get()
            };

            let r = if r.is_empty() {
                u16::max_value()
            } else {
                r.parse::<NonZeroU16>().map_err(|e| e.to_string())?.get()
            };

            (l..=r)
                .map(|port| super::Port {
                    num: port,
                    stat: super::Status::Closed,
                    meta: None,
                })
                .collect::<Vec<_>>()
        } else {
            vec![super::Port {
                num: s.parse::<NonZeroU16>().map_err(|e| e.to_string())?.get(),
                stat: super::Status::Closed,
                meta: None,
            }]
        };
        Ok(Self(ports))
    }
}
