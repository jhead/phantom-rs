use std::sync::Arc;

use log::debug;
use once_cell::sync::Lazy;
use proxy::ProxyInstance;

pub mod actor;
pub mod api;
pub mod proto;
pub mod proxy;

pub use api::{PhantomError, PhantomLogger, PhantomLoggerConfig, PhantomOpts};
use tokio::runtime::{Handle, Runtime};

uniffi::setup_scaffolding!();

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
            .spawn(async move { instance.listen().await })
            .await
            .map_err(PhantomError::from_error)?
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
                instance.wait().await;

                Ok(())
            })
            .await
            .map_err(PhantomError::from_error)?
    }

    pub fn set_logger(&self, logger: Box<dyn PhantomLogger>) -> Result<(), PhantomError> {
        let config = PhantomLoggerConfig::new(logger);

        log::set_boxed_logger(Box::new(config))
            .map_err(|e| PhantomError::LoggerSetupFailed(e.to_string()))?;

        log::set_max_level(log::LevelFilter::Debug);

        Ok(())
    }
}
