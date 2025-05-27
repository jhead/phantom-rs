#[derive(Clone, Debug, uniffi::Record)]
pub struct PhantomOpts {
    pub server: String,
    pub bind: String,
    pub bind_port: u16,
    pub timeout: u64,
    pub debug: bool,
    pub ipv6: bool,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PhantomError {
    #[error("Phantom encountered an error: {0}")]
    UnknownError(String),
    #[error("Failed to bind to address: {0}")]
    FailedToBind(String),
    #[error("Phantom failed to start: {0}")]
    FailedToStart(String),
    #[error("Phantom encountered an IO error: {0}")]
    IoError(String),
    #[error("Uneable to resolve remote address: {0}")]
    InvalidAddress(String),
    #[error("Phantom is already running")]
    AlreadyRunning,
}
