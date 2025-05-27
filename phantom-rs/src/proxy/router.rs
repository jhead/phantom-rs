use log::{error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::actor::{behavior, Actor, ActorError, ActorRef};
use crate::proto::unconnected_pong::UnconnectedPong;
use tokio::net::UdpSocket;

use bytes::Bytes;

#[derive(Debug, Clone)]
struct RouterState {
    remote_addr: SocketAddr,
    proxy_port: u16,
    client_map: HashMap<SocketAddr, ClientConnectionPair>,
}

#[derive(Debug, Clone)]
struct ClientConnectionPair {
    to_server: Arc<UdpSocket>,
}

pub struct Router {
    pub actor_ref: ActorRef<RouterMessage>,
}

impl Router {
    pub fn new(remote_addr: SocketAddr, proxy_port: u16) -> Result<Self, ActorError> {
        let initial_state = RouterState {
            remote_addr,
            proxy_port,
            client_map: HashMap::new(),
        };

        Actor::new(initial_state, behavior(Router::handle_message))
            .start()
            .map(|actor_ref| Self { actor_ref })
    }

    async fn handle_message(message: RouterMessage, mut state: RouterState) -> RouterState {
        match message {
            RouterMessage::PacketFromClient {
                data,
                client_addr,
                to_client,
            } => {
                Self::try_add_connection(&mut state, client_addr, to_client).await;

                if let Some(client_pair) = state.client_map.get(&client_addr) {
                    // Forward the packet to the remote server
                    client_pair
                        .to_server
                        .send_to(&data, state.remote_addr)
                        .await
                        .unwrap();

                    info!(
                        "[router] Forwarded {} bytes from {} via {} to remote server {}",
                        data.len(),
                        client_addr,
                        client_pair.to_server.local_addr().unwrap(),
                        state.remote_addr
                    );
                }
            }
        }
        state
    }

    async fn try_add_connection(
        state: &mut RouterState,
        client_addr: SocketAddr,
        to_client: Arc<UdpSocket>,
    ) {
        if !state.client_map.contains_key(&client_addr) {
            let to_server = Arc::new(UdpSocket::bind("0.0.0.0:0").await.unwrap());
            info!(
                "[router] New client connected {} -> {}",
                client_addr,
                to_server.local_addr().unwrap()
            );

            state.client_map.insert(
                client_addr,
                ClientConnectionPair {
                    to_server: to_server.clone(),
                },
            );

            let to_client_clone = to_client.clone();
            let proxy_port = state.proxy_port;
            tokio::spawn(async move {
                proxy_remote_read_loop(to_server, to_client_clone, client_addr, proxy_port).await;
            });
        }
    }
}

#[derive(Debug, Clone)]
pub enum RouterMessage {
    PacketFromClient {
        data: Bytes,
        client_addr: SocketAddr,
        to_client: Arc<UdpSocket>,
    },
}

async fn proxy_remote_read_loop(
    to_server: Arc<UdpSocket>,
    to_client: Arc<UdpSocket>,
    client_addr: SocketAddr,
    proxy_port: u16,
) {
    info!(
        "[remote-read] Listening for data from remote server on {}",
        to_server.local_addr().unwrap()
    );

    let mut buf = vec![0; 1024];
    loop {
        match to_server.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                let data = &buf[..len];
                let bytes = Bytes::from(data.to_vec());
                info!("[remote-read] Received {} bytes from {}", len, addr);

                if let Ok(original_pong) = UnconnectedPong::from_bytes(bytes) {
                    let mut new_pong = original_pong.clone();
                    new_pong.pong.port4 = proxy_port.to_string();
                    let new_bytes = new_pong.build();
                    to_client.send_to(&new_bytes, client_addr).await.unwrap();
                } else {
                    to_client.send_to(&data, client_addr).await.unwrap();
                }
            }
            Err(e) => {
                error!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
