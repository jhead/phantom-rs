use log::{debug, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::actor::{behavior, Actor, ActorRef, RunningActor};
use crate::proto::unconnected_pong::UnconnectedPong;
use crate::proxy::socket::read_cancellable;
use tokio::net::UdpSocket;

use bytes::Bytes;

use super::socket::CancellablePacketReader;

#[derive(Debug, Clone)]
struct RouterState {
    remote_addr: SocketAddr,
    proxy_port: u16,
    client_map: HashMap<SocketAddr, ClientConnectionPair>,
}

#[derive(Debug, Clone)]
pub enum RouterMessage {
    PacketFromClient {
        data: Bytes,
        client_addr: SocketAddr,
        to_client: Arc<UdpSocket>,
    },
}

#[derive(Debug, Clone)]
struct ClientConnectionPair {
    to_server: Arc<UdpSocket>,
}

pub type Router = RunningActor<RouterMessage>;
type RouterRef = ActorRef<RouterMessage>;

pub fn create_router(remote_addr: SocketAddr, proxy_port: u16) -> Router {
    let initial_state = RouterState {
        remote_addr,
        proxy_port,
        client_map: HashMap::new(),
    };

    Actor::run(initial_state, behavior(router_handler_message))
}

async fn router_handler_message(
    self_ref: RouterRef,
    message: RouterMessage,
    mut state: RouterState,
) -> RouterState {
    let RouterMessage::PacketFromClient {
        data,
        client_addr,
        to_client,
    } = message;

    try_add_connection(&self_ref, &mut state, client_addr, to_client).await;

    if let Some(client_pair) = state.client_map.get(&client_addr) {
        // Forward the packet to the remote server
        client_pair
            .to_server
            .send_to(&data, state.remote_addr)
            .await
            .unwrap();

        debug!(
            "[router] Forwarded {} bytes from {} via {} to remote server {}",
            data.len(),
            client_addr,
            client_pair.to_server.local_addr().unwrap(),
            state.remote_addr
        );
    }

    state
}

async fn try_add_connection(
    router_ref: &RouterRef,
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

        router_ref.attach_child(proxy_remote_read_loop(
            to_server,
            to_client_clone,
            client_addr,
            proxy_port,
        ));
    }
}

fn proxy_remote_read_loop(
    to_server: Arc<UdpSocket>,
    to_client: Arc<UdpSocket>,
    client_addr: SocketAddr,
    proxy_port: u16,
) -> CancellablePacketReader {
    info!(
        "[remote-read] Listening for data from remote server on {}",
        to_server.local_addr().unwrap()
    );

    read_cancellable(to_server, move |packet| {
        let to_client = to_client.clone();
        async move {
            if let Ok(original_pong) = UnconnectedPong::from_bytes(packet.data.clone()) {
                let mut new_pong = original_pong.clone();
                new_pong.pong.port4 = proxy_port.to_string();
                let new_bytes = new_pong.build();
                to_client.send_to(&new_bytes, client_addr).await.unwrap();
            } else {
                to_client.send_to(&packet.data, client_addr).await.unwrap();
            }
        }
    })
}
