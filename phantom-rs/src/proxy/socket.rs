use std::{future::Future, net::SocketAddr, sync::Arc};

use bytes::Bytes;
use log::{debug, error};
use tokio::net::UdpSocket;

use crate::task::TokioTask;

pub struct IncomingPacket {
    pub data: Bytes,
    pub client_addr: SocketAddr,
}

pub type CancellablePacketReader = TokioTask;

pub fn read_cancellable<F: Send + 'static, Fut>(
    socket: Arc<UdpSocket>,
    handler: F,
) -> CancellablePacketReader
where
    F: Fn(IncomingPacket) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    TokioTask::spawn(move |cancellation_token| async move {
        let mut buf = vec![0; 1024];

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("[socket-read] Cancellation signal received, stopping socket read loop.");
                    break;
                }
                read_res = socket.recv_from(&mut buf) => {
                    match read_res {
                        Ok((len, client_addr)) => {
                            let data = &buf[..len];
                            debug!(
                                "[socket-read] Received {} bytes from {} packet ID {}",
                                len, client_addr, data[0]
                            );
                            handler(IncomingPacket {
                                data: Bytes::from(data.to_vec()),
                                client_addr,
                            }).await;
                        }
                        Err(e) => {
                            error!("Error receiving data: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        debug!(
            "Socket {} shut down",
            socket
                .local_addr()
                .map(|a| a.to_string())
                .unwrap_or_else(|_| "unknown".to_string())
        );
    })
}
