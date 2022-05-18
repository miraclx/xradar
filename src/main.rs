use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use clap::Parser;
use futures::stream::{self, StreamExt};
use tokio::net::TcpStream;
use tokio::time;

mod cli;

#[derive(Debug)]
pub struct Port {
    num: u16,
    status: Status,
}

#[derive(Debug)]
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

    eprintln!("Host   : {}", host);
    eprintln!("Threads: {}", threads);
    eprintln!(
        "Timeout: {}s",
        tokio::time::Duration::from_millis(timeout).as_secs_f32(),
    );
    eprintln!("Retries: {}", retries);

    let host = Arc::new(host);

    let mut ports = stream::iter(ports)
        .map(move |mut port| {
            let host = host.clone();
            async move {
                for _ in 1..=retries {
                    tokio::select! {
                        _ = async {
                            if let Ok(_) = TcpStream::connect((host.as_str(), port.num)).await {
                                port.status = Status::Open;
                            }
                        } => break,
                        _ = time::sleep(time::Duration::from_millis(timeout)) => {
                            port.status = Status::TimedOut;
                        },
                    };
                }
                port
            }
        })
        .buffer_unordered(threads);

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
