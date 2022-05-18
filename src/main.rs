use std::collections::HashSet;
use std::num::{NonZeroU16, NonZeroUsize};
use std::str::FromStr;

use clap::{ArgEnum, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct CLIArgs {
    /// The host to connect to. (defaults to "localhost")
    host: Option<String>,
    /// An optional, non-zero port number to check. (defaults to scanning all ports)
    ///
    /// Format: `80`, `8000..8888`, `666,777,888,999`.
    #[clap(multiple_occurrences = true, use_value_delimiter = true)]
    port: Vec<CliPort>,
    /// Port test method to use.
    #[clap(arg_enum, short = 'm', default_value_t = Method::Fast)]
    method: Method,
    /// The number of threads to use. (defaults to the number of CPUs)
    #[clap(short = 'n')]
    threads: Option<NonZeroUsize>,
}

#[derive(ArgEnum, Eq, Clone, Debug, PartialEq)]
enum Method {
    #[clap(alias("f"))]
    Fast,
    #[clap(alias("s"))]
    Slow,
}

#[derive(Eq, Ord, Hash, Debug, PartialEq, PartialOrd)]
struct Port {
    port: u16,
    open: bool,
}

#[derive(Debug)]
struct CliPort(Vec<Port>);

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
                .map(|port| Port { port, open: false })
                .collect::<Vec<_>>()
        } else {
            vec![Port {
                port: s.parse::<NonZeroU16>().map_err(|e| e.to_string())?.get(),
                open: false,
            }]
        };
        Ok(Self(ports))
    }
}

fn main() -> anyhow::Result<()> {
    let args = CLIArgs::parse();

    println!(
        "Host: {}",
        args.host.as_ref().unwrap_or(&"localhost".to_string())
    );

    let mut ports = args
        .port
        .into_iter()
        .map(|CliPort(p)| p)
        .flatten()
        .collect::<Vec<_>>();

    let mut uniques = HashSet::new();
    ports.retain(|p| uniques.insert(p.port));

    println!("Ports: {:#?}", ports);

    let threads = args
        .threads
        .unwrap_or(std::thread::available_parallelism()?);

    println!("Threads: {}", threads);

    Ok(())
}
