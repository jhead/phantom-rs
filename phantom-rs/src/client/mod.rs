use std::time::Instant;

use bytes::Bytes;
use log::debug;
use once_cell::sync::Lazy;
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::runtime::{Handle, Runtime};
use tokio::time::{timeout, Duration};
use uniffi::Record;

use crate::proto::unconnected_ping::UnconnectedPing;
use crate::proto::unconnected_pong::{UnconnectedPong, UNCONNECTED_PONG_ID};

/// A simple client for pinging MCPE servers
#[derive(uniffi::Object)]
pub struct Client {
    client_id: [u8; 8],
    client_start_time: Instant,
    runtime: Handle,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum ClientError {
    #[error("Client encountered an IO error: {0}")]
    IoError(String),

    #[error("Client encountered a timeout while waiting for a ping response")]
    Timeout,

    #[error("Unable to ping invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid response from server: {0}")]
    InvalidResponse(String),
}

#[uniffi::export]
impl Client {
    /// Creates a new client bound to a random port
    #[uniffi::constructor]
    pub async fn new() -> Result<Self, ClientError> {
        static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        });

        let client_id = rand::rng().random::<[u8; 8]>();

        Ok(Self {
            client_id,
            client_start_time: Instant::now(),
            runtime: RUNTIME.handle().clone(),
        })
    }

    /// Pings a server and returns the pong response
    pub async fn ping(&self, addr: String) -> Result<Pong, ClientError> {
        let ping_time = elapsed_millis_bytes(self.client_start_time);
        let client_id = self.client_id;

        self.runtime
            .spawn(async move { send_ping(client_id, ping_time, addr).await })
            .await
            .map_err(|e| ClientError::IoError(e.to_string()))?
    }
}

fn elapsed_millis_bytes(start: Instant) -> [u8; 8] {
    // Get elapsed duration since `start`
    let dur = start.elapsed();

    // as_millis() returns a u128, so cast to u64
    let ms: u64 = dur.as_millis() as u64;

    // Convert to bytes (big-endian here; use `to_le_bytes()` for little-endian)
    ms.to_be_bytes()
}

async fn send_ping(
    client_id: [u8; 8],
    ping_time: [u8; 8],
    addr: String,
) -> Result<Pong, ClientError> {
    // Create and send ping packet
    let ping = UnconnectedPing::new(client_id, ping_time);
    let ping_bytes = ping.build();

    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| ClientError::IoError(e.to_string()))?;
    socket
        .set_broadcast(true)
        .map_err(|e| ClientError::IoError(e.to_string()))?;

    let addr = tokio::net::lookup_host(&addr)
        .await
        .map_err(|e| ClientError::InvalidAddress(e.to_string()))?
        .next()
        .ok_or_else(|| ClientError::InvalidAddress("No address found".to_string()))?;

    debug!("Sending ping to {}", addr);

    socket
        .send_to(&ping_bytes, addr)
        .await
        .map_err(|e| ClientError::IoError(e.to_string()))?;

    // Wait for response with timeout
    let mut buf = vec![0; 1024];
    let timeout_duration = Duration::from_secs(5);

    let (len, _) = timeout(timeout_duration, socket.recv_from(&mut buf))
        .await
        .map_err(|_| ClientError::Timeout)?
        .map_err(|e| ClientError::IoError(e.to_string()))?;

    let response = Bytes::from(buf[..len].to_vec());

    // Verify packet ID
    if response.is_empty() || response[0] != UNCONNECTED_PONG_ID {
        return Err(ClientError::InvalidResponse(
            "Invalid response packet ID".to_string(),
        ));
    }

    // Parse pong response
    let pong = UnconnectedPong::from_bytes(response)
        .map_err(|e| ClientError::InvalidResponse(e.to_string()))?;

    Ok(Pong {
        edition: pong.pong.edition,
        motd: pong.pong.motd,
        protocol_version: pong.pong.protocol_version,
        version: pong.pong.version,
        players: pong.pong.players,
        max_players: pong.pong.max_players,
        server_id: pong.pong.server_id,
        sub_motd: pong.pong.sub_motd,
        game_mode: pong.pong.game_mode,
        game_mode_numeric: pong.pong.game_mode_numeric,
        port4: pong.pong.port4,
        port6: pong.pong.port6,
    })
}

/// Response data from a server ping
#[derive(Record)]
pub struct Pong {
    pub edition: String,
    pub motd: String,
    pub protocol_version: String,
    pub version: String,
    pub players: String,
    pub max_players: String,
    pub server_id: String,
    pub sub_motd: String,
    pub game_mode: String,
    pub game_mode_numeric: String,
    pub port4: String,
    pub port6: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let client = Client::new().await.expect("Failed to create client");
        let addr = "127.0.0.1:19132".to_string();

        // This will fail if no server is running, but that's expected
        let result = client.ping(addr).await;
        assert!(result.is_err());
    }
}
