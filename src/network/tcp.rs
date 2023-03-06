use futures::Stream;
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

use crate::error::AsyncListenerError;
use tracing::{debug, error, info, span, warn, Level};

#[derive(Debug)]
pub struct TCPPacket {
    pub data: Vec<u8>,
    pub addr: SocketAddr,
}

impl TCPPacket {
    fn new(data: Vec<u8>, addr: SocketAddr) -> Self {
        Self { data, addr }
    }
}

#[tracing::instrument]
pub async fn streaming_tcp_packets(
    port: u16,
) -> impl Stream<Item = Result<TCPPacket, AsyncListenerError>> {
    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await;
    async_stream::stream! {
        loop {
            match &tcp_listener {
                Ok(tcp_listener) => {
                    let (stream, addr) = tcp_listener.accept().await.unwrap();
                    stream.readable().await?;
                    //Usually for TCP in most cloud providers the max UDP MTU is 1500.  Setting to 2048 for keeping memory in increments of 1024.
                    let mut buf = [0u8; 2048];
                    match stream.try_read(&mut buf) {
                        Ok(size) => {
                            let packet = buf[..size].to_vec();
                            let event = TCPPacket::new(packet, addr);
                            yield Ok(event);
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(err) => {
                            yield Err(AsyncListenerError::from(err));
                        }
                    }
                }
                Err(err) => {
                    yield Err(AsyncListenerError::from(err));
                }
            }
        }
    }
}
