mod router;
mod socket;

use log::{debug, error, info};
use socket::{read_cancellable, CancellablePacketReader};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Notify;

use crate::actor::ActorRef;
use crate::api::{PhantomError, PhantomOpts};
use crate::task::TaskManager;
use router::{create_router, Router, RouterMessage};

#[derive(uniffi::Object)]
pub struct ProxyInstance {
    running: AtomicBool,
    opts: PhantomOpts,
    manager: TaskManager,
    notify_shutdown: Notify,
}

impl ProxyInstance {
    pub fn new(opts: PhantomOpts) -> Result<Self, PhantomError> {
        Ok(ProxyInstance {
            running: AtomicBool::new(false),
            opts,
            manager: TaskManager::new(),
            notify_shutdown: Notify::new(),
        })
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn listen(&self) -> Result<(), PhantomError> {
        self.running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| PhantomError::AlreadyRunning)?;

        let remote_server = resolve_remote_address(&self.opts.server).await?;
        self.start_listeners(remote_server).await?;

        Ok(())
    }

    async fn start_listeners(&self, remote_addr: SocketAddr) -> Result<(), PhantomError> {
        let broadcast_socket = bind_socket_reuse(&self.opts.bind, 19132).await?;
        let broadcast_local_addr = broadcast_socket
            .local_addr()
            .map_err(|e| PhantomError::FailedToBind(e.to_string()))?;

        info!("Broadcast server listening on {}", broadcast_local_addr);

        let proxy_socket = bind_socket(&self.opts.bind, self.opts.bind_port).await?;
        let proxy_local_addr = proxy_socket
            .local_addr()
            .map_err(|e| PhantomError::FailedToBind(e.to_string()))?;

        info!("Proxy server listening on {}", proxy_local_addr);

        let proxy_port = proxy_local_addr.port();

        let router = create_router(remote_addr, proxy_port);
        self.spawn_socket_reader(broadcast_socket, &router).await;
        self.spawn_socket_reader(proxy_socket, &router).await;
        self.manager.add_task(router);

        Ok(())
    }

    async fn spawn_socket_reader(&self, socket: UdpSocket, router: &Router) {
        let task = socket_pipe_to_router(socket, router);
        self.manager.add_task(task);
    }

    pub async fn join(&self) {
        self.notify_shutdown.notified().await;
        debug!("All tasks completed");
    }

    pub async fn shutdown(&self) -> Result<(), PhantomError> {
        debug!("Shutdown signal sent to all tasks");
        self.manager.shutdown().await;
        self.running.store(false, Ordering::SeqCst);
        self.notify_shutdown.notify_waiters();
        Ok(())
    }
}

fn socket_pipe_to_router(
    socket: UdpSocket,
    router: &ActorRef<RouterMessage>,
) -> CancellablePacketReader {
    let socket: Arc<UdpSocket> = Arc::new(socket);
    let router = router.clone();

    read_cancellable(socket.clone(), move |packet| {
        let router = router.clone();
        let socket = socket.clone();
        async move {
            router
                .send(RouterMessage::PacketFromClient {
                    data: packet.data,
                    client_addr: packet.client_addr,
                    to_client: socket,
                })
                .unwrap_or_else(|e| error!("Error sending message to router: {}", e));
        }
    })
}

async fn resolve_remote_address(server: &str) -> Result<SocketAddr, PhantomError> {
    server
        .to_socket_addrs()
        .map_err(|e| PhantomError::InvalidAddress(e.to_string()))?
        .next()
        .ok_or(PhantomError::InvalidAddress(
            "Remote server address not found".to_string(),
        ))
}

async fn bind_socket_reuse(bind: &str, port: u16) -> Result<UdpSocket, PhantomError> {
    let addr: SocketAddr = format!("{}:{}", bind, port).parse().unwrap();

    // TODO: Support ipv6
    let socket = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .map_err(|e| PhantomError::FailedToBind(e.to_string()))?;

    socket
        .set_reuse_port(true)
        .map_err(|e| PhantomError::FailedToBind(e.to_string()))?;

    socket
        .set_reuse_address(true)
        .map_err(|e| PhantomError::FailedToBind(e.to_string()))?;

    socket
        .set_nonblocking(true)
        .map_err(|e| PhantomError::FailedToBind(e.to_string()))?;

    socket
        .bind(&addr.into())
        .map_err(|e| PhantomError::FailedToBind(e.to_string()))?;

    let socket_std = std::net::UdpSocket::from(socket);

    UdpSocket::from_std(socket_std).map_err(|e| PhantomError::FailedToBind(e.to_string()))
}

async fn bind_socket(bind: &str, port: u16) -> Result<UdpSocket, PhantomError> {
    let addr = format!("{}:{}", bind, port);
    UdpSocket::bind(&addr)
        .await
        .map_err(|e| PhantomError::FailedToBind(e.to_string()))
}
