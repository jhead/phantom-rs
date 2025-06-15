mod logger;

use log::debug;
use logger::{PhantomLogger, PhantomLoggerConfig};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::runtime::{Handle, Runtime};

use crate::proxy::ProxyInstance;

#[derive(uniffi::Object)]
pub struct Phantom {
    instance: Arc<ProxyInstance>,
    rt: Handle,
}

pub fn new_with_current_runtime(opts: PhantomOpts) -> Result<Phantom, PhantomError> {
    let rt = tokio::runtime::Handle::current();
    new_with_runtime(opts, &rt)
}

pub fn new_with_runtime(opts: PhantomOpts, rt: &Handle) -> Result<Phantom, PhantomError> {
    let instance = Arc::new(ProxyInstance::new(opts)?);
    Ok(Phantom {
        instance,
        rt: rt.clone(),
    })
}

#[uniffi::export]
impl Phantom {
    #[uniffi::constructor]
    pub fn new(opts: PhantomOpts) -> Result<Self, PhantomError> {
        static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        });

        new_with_runtime(opts, RUNTIME.handle())
    }

    pub async fn start(&self) -> Result<(), PhantomError> {
        if self.instance.is_running() {
            debug!("Phantom instance is already running");
            return Ok(());
        }

        debug!("Starting Phantom instance...");

        let instance = self.instance.clone();

        self.rt
            .spawn(async move {
                instance.listen().await?;
                instance.join().await;
                Ok(())
            })
            .await
            .map_err(unknown_error)?
    }

    pub async fn stop(&self) -> Result<(), PhantomError> {
        if !self.instance.is_running() {
            debug!("Phantom instance is not running, nothing to stop");
            return Ok(());
        }

        debug!("Stopping Phantom instance...");

        let instance = self.instance.clone();

        self.rt
            .spawn(async move {
                instance.shutdown().await?;
                Ok(())
            })
            .await
            .map_err(unknown_error)?
    }

    pub fn set_logger(&self, logger: Box<dyn PhantomLogger>) -> Result<(), PhantomError> {
        let config = PhantomLoggerConfig::new(logger);

        log::set_boxed_logger(Box::new(config))
            .map_err(|e| PhantomError::LoggerSetupFailed(e.to_string()))?;

        log::set_max_level(log::LevelFilter::Debug);

        Ok(())
    }
}

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

    #[error("Unable to resolve remote address: {0}")]
    InvalidAddress(String),

    #[error("Phantom is already running")]
    AlreadyRunning,

    #[error("Unable to configure Phantom logger: {0}")]
    LoggerSetupFailed(String),
}

pub fn unknown_error(error: impl std::error::Error) -> PhantomError {
    PhantomError::UnknownError(error.to_string())
}
