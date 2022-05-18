use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use clap::Parser;
use futures::stream::{self, StreamExt};
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time;

mod cli;

#[derive(Debug)]
pub struct Port {
    num: u16,
    meta: Option<anyhow::Result<Result<String, String>>>,
    stat: Status,
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
                                port.stat = Status::Open;
                                if args.inspect {
                                    port.meta = Some(port_stat(port.num).await);
                                }
                            }
                        } => break,
                        _ = time::sleep(time::Duration::from_millis(timeout)) => {
                            port.stat = Status::TimedOut;
                        },
                    };
                }
                port
            }
        })
        .buffer_unordered(threads);

    if !args.inspect {
        println!("┌───────┬────────┐");
        println!("│  Port │ Status │");
        println!("├───────┼────────┤");
        while let Some(port) = ports.next().await {
            if args.all || matches!(port.stat, Status::Open | Status::TimedOut) {
                println!("│ {:>5} │ {:^7}│", port.num, port.stat);
            }
        }
        println!("└───────┴────────┘");
    } else {
        println!("----------------");
        while let Some(port) = ports.next().await {
            if args.all || matches!(port.stat, Status::Open | Status::TimedOut) {
                println!(" {} ({})", port.num, port.stat);
                if let Some(data) = port.meta {
                    let report = match data {
                        Ok(Ok(stat)) if stat.is_empty() => "(no data)".to_string(),
                        Ok(Ok(stat)) => stat,
                        Ok(Err(err)) if err.is_empty() => {
                            "(inspection failed, no data)".to_string()
                        }
                        Ok(Err(err)) => err,
                        Err(err) => format!("{}", err),
                    };
                    for line in report.lines() {
                        println!("   │ {}", line);
                    }
                }
            }
        }
        println!("----------------");
    }

    Ok(())
}

#[cfg(target_family = "windows")]
async fn port_stat(port: u16) -> anyhow::Result<String> {
    todo!("windows?")
}

#[cfg(target_family = "unix")]
async fn port_stat(port: u16) -> anyhow::Result<Result<String, String>> {
    let output = Command::new("lsof")
        .arg("-i")
        .arg(format!("tcp:{}", port))
        .arg("-s")
        .arg("tcp:listen")
        .arg("-P")
        .arg("-n")
        .kill_on_drop(true)
        .output()
        .await?;
    if output.status.success() {
        return Ok(Ok(String::from_utf8(output.stdout)?));
    } else {
        match output.status.code() {
            Some(_) => Ok(Err(String::from_utf8(output.stderr)?)),
            None => anyhow::bail!("process terminated by signal"),
        }
    }
}
