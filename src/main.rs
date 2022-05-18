use std::collections::HashSet;
use std::sync::Arc;

use clap::Parser;
use futures::stream::{self, StreamExt};
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time;

mod cli;

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
        println!(
            "│  {} │ {} │",
            args.colors.paint("Port", "\x1b[1m"),
            args.colors.paint("Status", "\x1b[1m")
        );
        println!("├───────┼────────┤");
        while let Some(port) = ports.next().await {
            if args.all || matches!(port.stat, Status::Open | Status::TimedOut) {
                println!(
                    "│ {} │ {}│",
                    args.colors.paint(&format!("{:>5}", port.num), "\x1b[1m"),
                    port.stat.display(7, args.colors)
                );
            }
        }
        println!("└───────┴────────┘");
    } else {
        println!("----------------");
        while let Some(port) = ports.next().await {
            if args.all || matches!(port.stat, Status::Open | Status::TimedOut) {
                println!(
                    " {} ({})",
                    args.colors.paint(&port.num.to_string(), "\x1b[1m"),
                    port.stat.display(0, args.colors)
                );
                if let Some(data) = port.meta {
                    let report = match data {
                        Ok(Ok(stat)) if stat.is_empty() => {
                            format!("({})", args.colors.paint("no data", "\x1b[33m"))
                        }
                        Ok(Ok(stat)) => stat,
                        Ok(Err((code, err))) if err.is_empty() => {
                            format!("inspection failed with exit code {}", code)
                        }
                        Ok(Err((code, err))) => {
                            format!(
                                "inspection failed with exit code {}: {}",
                                code,
                                args.colors.paint(&err, "\x1b[33m")
                            )
                        }
                        Err(err) => format!(
                            "inspection failed: {}",
                            args.colors.paint(&err.to_string(), "\x1b[31m")
                        ),
                    };
                    for line in report.lines() {
                        println!(
                            "   {} {}",
                            args.colors.paint("│", "\x1b[1m\x1b[38;5;243m"),
                            line
                        );
                    }
                }
            }
        }
        println!("----------------");
    }

    Ok(())
}

#[derive(Debug)]
pub struct Port {
    num: u16,
    meta: Option<anyhow::Result<Result<String, (i32, String)>>>,
    stat: Status,
}

#[derive(Debug)]
pub enum Status {
    Open,
    Closed,
    TimedOut,
}

impl Status {
    fn display(&self, pad: usize, colors: cli::CliColors) -> String {
        match self {
            Status::Open => colors.paint(&format!("{:^pad$}", "open", pad = pad), "\x1b[32m"),
            Status::Closed => {
                colors.paint(&format!("{:^pad$}", "closed", pad = pad), "\x1b[38;5;249m")
            }
            Status::TimedOut => {
                colors.paint(&format!("{:^pad$}", "timeout", pad = pad), "\x1b[33m")
            }
        }
    }
}

#[cfg(target_family = "windows")]
async fn port_stat(port: u16) -> anyhow::Result<String> {
    todo!("windows?")
}

#[cfg(target_family = "unix")]
async fn port_stat(port: u16) -> anyhow::Result<Result<String, (i32, String)>> {
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
        return Ok(Ok(String::from_utf8(output.stdout)?.trim().to_owned()));
    } else {
        match output.status.code() {
            Some(code) => Ok(Err((
                code,
                String::from_utf8(output.stderr)?.trim().to_owned(),
            ))),
            None => anyhow::bail!("process terminated by signal"),
        }
    }
}
