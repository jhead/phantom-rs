use std::sync::Arc;

use clap::{command, Parser};
use log::{error, info};
use phantom_rs::PhantomOpts;
use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Bedrock/MCPE server IP address and port (ex: 1.2.3.4:19132)
    #[arg(short, long)]
    server: String,

    /// IP address to listen on. Defaults to all interfaces.
    #[arg(long, default_value = "0.0.0.0")]
    bind: String,

    /// Port to listen on. Defaults to 0, which selects a random port.
    /// Note that phantom always binds to port 19132 as well, so both ports need to be open.
    #[arg(long, default_value_t = 0)]
    bind_port: u16,

    // TODO: implement timeouts
    /// Seconds to wait before cleaning up a disconnected client
    #[arg(long, default_value_t = 60)]
    timeout: u64,

    /// Enables debug logging
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Enables IPv6 support on port 19133 (experimental)
    #[arg(short = '6', long, default_value_t = false)]
    ipv6: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("Args: {:?}", args);

    let opts = PhantomOpts {
        server: args.server.clone(),
        bind: args.bind.clone(),
        bind_port: args.bind_port,
        timeout: args.timeout,
        debug: args.debug,
        ipv6: args.ipv6,
    };

    let log_level = if opts.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let _ = TermLogger::init(
        log_level,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Always,
    );

    info!("Starting Phantom with options: {:?}", opts);
    let phantom = Arc::new(
        phantom_rs::new_with_current_runtime(opts).expect("Failed to create Phantom instance"),
    );

    // Catch ctrl-c to stop Phantom gracefully
    let phantom_for_shutdown = phantom.clone();
    tokio::spawn(async move {
        loop {
            let _ = tokio::signal::ctrl_c().await;
            info!("Ctrl-C received, stopping Phantom...");
            phantom_for_shutdown
                .stop()
                .await
                .expect("Failed to stop Phantom");
        }
    });

    if let Err(e) = phantom.start().await {
        error!("Failed to start Phantom: {}", e);
        return;
    }

    info!("Phantom shut down");
}
