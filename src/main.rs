use std::collections::HashSet;
use std::num::{NonZeroU16, NonZeroUsize};
use std::str::FromStr;

use clap::{ArgEnum, Parser};
use futures::stream::StreamExt;

mod methods;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct CLIArgs {
    /// The host to connect to. (defaults to "localhost")
    host: Option<String>,
    /// An optional, non-zero port number to check. (defaults to scanning all ports)
    ///
    /// Format: `80`, `8000..8888`, `666,777,888,999`.
    #[clap(
        multiple_occurrences = true,
        use_value_delimiter = true,
        default_value = "-"
    )]
    port: Vec<CliPort>,
    /// Port test method to use.
    #[clap(arg_enum, short = 'm', default_value_t = Method::Fast)]
    method: Method,
    /// Show status of all ports, open or closed. (defaults to false)
    #[clap(short = 'a')]
    all: bool,
    /// The number of threads to use. (defaults to the number of CPUs)
    #[clap(short = 'n')]
    threads: Option<NonZeroUsize>,
    /// Be verbose. Query information about each port. (defaults to false)
    #[clap(short = 'v')]
    verbose: bool,
}

#[derive(ArgEnum, Eq, Clone, Debug, PartialEq)]
enum Method {
    #[clap(alias("f"))]
    Fast,
    #[clap(alias("s"))]
    Slow,
}

#[derive(Eq, Ord, Hash, Debug, PartialEq, PartialOrd)]
pub struct Port {
    num: u16,
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
                .map(|port| Port {
                    num: port,
                    open: false,
                })
                .collect::<Vec<_>>()
        } else {
            vec![Port {
                num: s.parse::<NonZeroU16>().map_err(|e| e.to_string())?.get(),
                open: false,
            }]
        };
        Ok(Self(ports))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CLIArgs::parse();

    let host = args.host.unwrap_or("localhost".to_string());

    let mut ports = args
        .port
        .into_iter()
        .map(|CliPort(p)| p)
        .flatten()
        .collect::<Vec<_>>();

    let mut uniques = HashSet::new();
    ports.retain(|p| uniques.insert(p.num));

    let threads = args
        .threads
        .unwrap_or(std::thread::available_parallelism()?)
        .get();

    println!("Host   : {}", host);
    println!("Threads: {}", threads);

    let mut ports = match args.method {
        Method::Slow => {
            println!("Method : Slow");
            methods::client::run(host, ports, threads).await
        }
        Method::Fast => {
            println!("Method : Fast");
            todo!()
            // methods::server::run(args.host, ports, threads);
        }
    };
    if !args.verbose {
        println!("┌───────┬────────┐");
        println!("│  Port │ Status │");
        println!("├───────┼────────┤");
        while let Some(port) = ports.next().await {
            if args.all || port.open {
                println!(
                    "│ {:>5} │ {:^6} │",
                    port.num,
                    if port.open { "open" } else { "closed" }
                );
            }
        }
        println!("└───────┴────────┘");
    } else {
        println!("----------------");
        while let Some(port) = ports.next().await {
            if args.all || port.open {
                println!(
                    " {:>5} | {}",
                    port.num,
                    if port.open { "open" } else { "closed" }
                );
            }
        }
    }

    Ok(())
}
