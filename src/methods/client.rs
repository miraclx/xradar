use std::sync::Arc;

use futures::stream::{self, BoxStream, StreamExt};
use tokio::net::TcpStream;
use tokio::time;

use crate::{Port, Status};

pub async fn run(
    host: String,
    ports: Vec<Port>,
    threads: usize,
    timeout: u64,
    retries: usize,
) -> BoxStream<'static, Port> {
    let host = Arc::new(host);
    let tasks = stream::iter(ports).map(move |mut port| {
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
    });
    Box::pin(tasks.buffer_unordered(threads))
}
