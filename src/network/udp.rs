use futures::Stream;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::error::AsyncListenerError;
use tracing::{debug, error, info, span, warn, Level};

#[derive(Debug)]
pub struct UDPPacket {
    pub data: Vec<u8>,
    pub addr: SocketAddr,
}

impl UDPPacket {
    fn new(data: Vec<u8>, addr: SocketAddr) -> Self {
        Self { data, addr }
    }
}

#[tracing::instrument]
pub async fn streaming_udp_packets(
    port: u16,
) -> impl Stream<Item = Result<UDPPacket, AsyncListenerError>> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await;

    async_stream::stream! {
        loop {
            match &socket {
                Ok(socket) => {
                    //Usually for UDP the max UDP MTU is 1460.  Setting to 2048 for keeping memory in increments of 1024.
                    let mut buf = [0u8; 2048];
                    let (size, addr) = &socket.recv_from(&mut buf).await.unwrap();
                    let packet = buf[..*size].to_vec();
                    let event = UDPPacket::new(packet, *addr);
                    yield Ok(event);
                }
                Err(err) => {
                    yield Err(AsyncListenerError::from(err));
                }
            }
        }
    }
}

pub struct UDPBytes {
    pub data: Box<dyn Stream<Item = u8>>,
    pub addr: SocketAddr,
}

impl UDPBytes {
    fn new(data: Box<dyn Stream<Item = u8>>, addr: SocketAddr) -> Self {
        Self { data, addr }
    }
}

//todo.  Create a hash of differnt clients and stream of bytes for each client.
// so a stream of stream of udp bytes.
// #[tracing::instrument]
// pub async fn streaming_udp_bytes(
//     port: u16,
// ) -> impl Stream<Item = Result<UDPPacket, AsyncListenerError>> {
//     let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await;

//     async_stream::stream! {
//         loop {
//             match &socket {
//                 Ok(socket) => {
//                     let mut buf = [0u8; 1024];
//                     let (size, addr) = &socket.recv_from(&mut buf).await.unwrap();
//                     let packet = buf[..*size].to_vec();
//                     let event = UDPPacket::new(packet, *addr);
//                     yield Ok(event);
//                 }
//                 Err(err) => {
//                     yield Err(AsyncListenerError::from(err));
//                 }
//             }
//         }
//     }
// }
