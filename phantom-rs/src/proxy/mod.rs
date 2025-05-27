mod router;

use bytes::Bytes;
use log::{debug, error, info};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, Mutex, Notify};
use tokio::task::JoinHandle;

use crate::actor::ActorRef;
use crate::api::{PhantomError, PhantomOpts};
use router::{Router, RouterMessage};

#[derive(uniffi::Object)]
pub struct ProxyInstance {
    running: AtomicBool,
    opts: PhantomOpts,
    shutdown_tx: broadcast::Sender<()>,
    notify_on_shutdown: Notify,
    tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl ProxyInstance {
    pub fn new(opts: PhantomOpts) -> Result<Self, PhantomError> {
        let (shutdown_tx, _) = broadcast::channel(1);
        let notify_on_shutdown = Notify::new();

        Ok(ProxyInstance {
            running: AtomicBool::new(false),
            opts,
            shutdown_tx,
            notify_on_shutdown,
            tasks: Arc::new(Mutex::new(Vec::new())),
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

        self.handle_shutdown().await;
        Ok(())
    }

    pub async fn wait(&self) {
        if !self.is_running() {
            debug!("Phantom instance is not running, nothing to wait for");
            return;
        }

        debug!("Waiting for Phantom instance to shut down...");
        self.notify_on_shutdown.notified().await;
    }

    async fn start_listeners(&self, remote_addr: SocketAddr) -> Result<(), PhantomError> {
        let broadcast_socket = bind_socket(&self.opts.bind, 19132).await?;
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

        let router = Router::new(remote_addr, proxy_port)
            .map_err(|e| PhantomError::FailedToStart(e.to_string()))?;

        self.spawn_socket_reader(broadcast_socket, &router).await;
        self.spawn_socket_reader(proxy_socket, &router).await;

        Ok(())
    }

    async fn spawn_socket_reader(&self, socket: Arc<UdpSocket>, router: &Router) {
        let router_ref = router.actor_ref.clone();
        let shutdown_rx = self.shutdown_tx.subscribe();

        let task = tokio::spawn(async move {
            socket_pipe_to_router(socket, router_ref, shutdown_rx).await;
        });

        let mut tasks = self.tasks.lock().await;
        tasks.push(task);
    }

    pub async fn shutdown(&self) -> Result<(), PhantomError> {
        // Notify all tasks to shut down
        self.shutdown_tx
            .send(())
            .map_err(|e| PhantomError::UnknownError(e.to_string()))?;

        debug!("Shutdown signal sent to all tasks");
        Ok(())
    }

    async fn handle_shutdown(&self) {
        self.shutdown_tx.subscribe().recv().await.ok();
        debug!("Phantom received shutdown request");

        let mut tasks = self.tasks.lock().await;
        let mut task_count = tasks.len();

        for task in tasks.drain(..) {
            info!(
                "Waiting for all tasks to complete... ({} tasks running)",
                task_count
            );

            let _ = task.await;
            task_count -= 1;
        }

        self.running.store(false, Ordering::SeqCst);
        info!("All tasks completed, Phantom instance shut down successfully");

        self.notify_on_shutdown.notify_waiters();
    }
}

async fn socket_pipe_to_router(
    socket: Arc<UdpSocket>,
    router: ActorRef<RouterMessage>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut buf = vec![0; 1024];

    loop {
        // select either socket or shutdown signal
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, stopping socket read loop.");
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

                        router
                            .send(RouterMessage::PacketFromClient {
                                data: Bytes::from(data.to_vec()),
                                client_addr,
                                to_client: socket.clone(),
                            })
                            .unwrap_or_else(|e| error!("Error sending message to router: {}", e));
                    }
                    Err(e) => {
                        error!("Error receiving data: {}", e);
                        break;
                    }
                }
                break;
            }
        }
    }

    info!(
        "Socket {} shut down",
        socket
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    );
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

async fn bind_socket(bind: &str, port: u16) -> Result<Arc<UdpSocket>, PhantomError> {
    let addr = format!("{}:{}", bind, port);
    UdpSocket::bind(&addr)
        .await
        .map_err(|e| PhantomError::FailedToBind(e.to_string()))
        .map(Arc::new)
}
