use std::sync::Arc;

use futures::{stream, Stream, StreamExt};
use tokio::net::TcpStream;

use crate::Port;

pub async fn run<'a>(host: String, ports: Vec<Port>, threads: usize) -> impl Stream<Item = Port> {
    let host = Arc::new(host);
    let tasks = stream::iter(ports).map(move |mut port| {
        let host = host.clone();
        async move {
            port.open |= matches!(TcpStream::connect((host.as_str(), port.num)).await, Ok(_));
            port
        }
    });
    tasks.buffer_unordered(threads)
}
