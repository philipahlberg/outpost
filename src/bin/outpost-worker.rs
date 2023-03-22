use std::{path::PathBuf, time::Duration};

use clap::Parser;
use outpost::{config::Credentials, worker};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "outpost-worker", version)]
enum Cli {
    Poll {
        #[arg(long)]
        on_update: PathBuf,

        #[arg(long)]
        iterations: Option<usize>,

        #[arg(long)]
        interval: Option<u64>,
    },
}

fn main() {
    setup_logging();

    match Cli::parse() {
        Cli::Poll {
            on_update,
            iterations,
            interval,
        } => {
            let credentials = Credentials::from_env().expect("invalid credentials");
            let interval = Duration::from_secs(interval.unwrap_or(60));
            worker::poll(on_update, interval, iterations, credentials)
                .expect("failed to run `poll`");
        }
    }

    tracing::info!("Process exited.");
}

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "outpost=debug".into());

    let formatter = tracing_subscriber::fmt::layer().json();

    tracing_subscriber::registry()
        .with(filter)
        .with(formatter)
        .init()
}
