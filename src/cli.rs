use std::num::{NonZeroU16, NonZeroU64, NonZeroUsize};
use std::str::FromStr;

use clap::{ArgEnum, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// The host to connect to. (defaults to "localhost")
    pub host: Option<String>,
    /// An optional, non-zero port number to check. (defaults to scanning all ports)
    ///
    /// Format: `80`, `8000..8888`, `666,777,888,999`.
    #[clap(
        multiple_occurrences = true,
        use_value_delimiter = true,
        default_value = "-"
    )]
    pub port: Vec<CliPort>,
    /// Port test method to use.
    #[clap(arg_enum, short = 'm', long, default_value_t = Method::Fast)]
    pub method: Method,
    /// Show status of all ports, open or closed. (defaults to false)
    #[clap(short = 'a', long)]
    pub all: bool,
    /// Timeout for port checks (ms).
    #[clap(short = 't', long, default_value = "2000")]
    pub timeout: NonZeroU64,
    /// Number of retries per port, on timeout.
    #[clap(short = 'r', long, default_value = "2")]
    pub retries: NonZeroUsize,
    /// The number of threads to use. (defaults to the number of CPUs)
    #[clap(short = 'n', long)]
    pub threads: Option<NonZeroUsize>,
    /// Be verbose. Query information about each port. (defaults to false)
    #[clap(short = 'v', long)]
    pub verbose: bool,
}

#[derive(ArgEnum, Eq, Clone, Debug, PartialEq)]
pub enum Method {
    #[clap(alias("f"))]
    Fast,
    #[clap(alias("s"))]
    Slow,
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
                    status: super::Status::Closed,
                })
                .collect::<Vec<_>>()
        } else {
            vec![super::Port {
                num: s.parse::<NonZeroU16>().map_err(|e| e.to_string())?.get(),
                status: super::Status::Closed,
            }]
        };
        Ok(Self(ports))
    }
}
