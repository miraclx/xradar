use std::collections::HashSet;
use std::fmt;

use clap::Parser;
use futures::stream::StreamExt;

mod cli;
mod methods;

#[derive(Eq, Ord, Hash, Debug, PartialEq, PartialOrd)]
pub struct Port {
    num: u16,
    status: Status,
}

#[derive(Eq, Ord, Hash, Debug, PartialEq, PartialOrd)]
pub enum Status {
    Open,
    Closed,
    TimedOut,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Open => f.pad("open"),
            Status::Closed => f.pad("closed"),
            Status::TimedOut => f.pad("timeout"),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    let host = args.host.unwrap_or("localhost".to_string());

    let mut ports = args
        .port
        .into_iter()
        .map(|p| p.0)
        .flatten()
        .collect::<Vec<_>>();

    let mut uniques = HashSet::new();
    ports.retain(|p| uniques.insert(p.num));

    let threads = args
        .threads
        .unwrap_or(std::thread::available_parallelism()?)
        .get();

    let retries = args.retries.get();
    let timeout = args.timeout.get();

    println!("Host   : {}", host);
    println!("Threads: {}", threads);
    println!(
        "Timeout: {}s ({}ms)",
        tokio::time::Duration::from_millis(timeout).as_secs_f32(),
        timeout
    );
    println!("Retries: {}", retries);

    let mut ports = match args.method {
        cli::Method::Slow => {
            println!("Method : Slow");
            methods::client::run(host, ports, threads, timeout, retries).await
        }
        cli::Method::Fast => {
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
            if args.all || matches!(port.status, Status::Open | Status::TimedOut) {
                println!("│ {:>5} │ {:^7}│", port.num, port.status);
            }
        }
        println!("└───────┴────────┘");
    } else {
        println!("----------------");
        while let Some(port) = ports.next().await {
            if args.all || matches!(port.status, Status::Open | Status::TimedOut) {
                println!(" {:>5} | {}", port.num, port.status);
            }
        }
    }

    Ok(())
}
