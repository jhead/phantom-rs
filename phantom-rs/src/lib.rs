use log::debug;
use proxy::ProxyInstance;

pub mod actor;
pub mod api;
pub mod proto;
pub mod proxy;

pub use api::{PhantomError, PhantomOpts};

uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
pub struct Phantom {
    instance: ProxyInstance,
}

#[uniffi::export]
impl Phantom {
    #[uniffi::constructor]
    pub fn new(opts: PhantomOpts) -> Result<Self, PhantomError> {
        let instance = ProxyInstance::new(opts)?;
        Ok(Phantom { instance })
    }

    pub async fn start(&self) -> Result<(), PhantomError> {
        if self.instance.is_running() {
            debug!("Phantom instance is already running");
            return Ok(());
        }

        debug!("Starting Phantom instance...");
        self.instance.listen().await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), PhantomError> {
        if !self.instance.is_running() {
            debug!("Phantom instance is not running, nothing to stop");
            return Ok(());
        }

        debug!("Stopping Phantom instance...");
        self.instance.shutdown().await?;
        self.instance.wait().await;
        Ok(())
    }
}
