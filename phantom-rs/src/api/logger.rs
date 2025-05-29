#[uniffi::export(callback_interface)]
pub trait PhantomLogger: Send + Sync {
    fn log_string(&self, str: String);
}

pub struct PhantomLoggerConfig {
    logger: Box<dyn PhantomLogger>,
}

impl PhantomLoggerConfig {
    pub fn new(logger: Box<dyn PhantomLogger>) -> Self {
        PhantomLoggerConfig { logger }
    }
}

impl log::Log for PhantomLoggerConfig {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let message = format!("[{}] {}", record.level(), record.args());
        self.logger.log_string(message);
    }

    fn flush(&self) {}
}
