use std::collections::HashSet;

use clap::Parser;
use futures::stream::StreamExt;

mod cli;
mod methods;

#[derive(Eq, Ord, Hash, Debug, PartialEq, PartialOrd)]
pub struct Port {
    num: u16,
    open: bool,
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

    println!("Host   : {}", host);
    println!("Threads: {}", threads);

    let mut ports = match args.method {
        cli::Method::Slow => {
            println!("Method : Slow");
            methods::client::run(host, ports, threads).await
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
