use std::path::PathBuf;

use clap::Parser;
use outpost::config::Config;
use outpost::{cli, config::Credentials};
use time::macros::format_description;
use tracing_subscriber::{
    fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[derive(Parser)]
#[command(name = "outpost", version)]
enum Command {
    Start {
        /// The path to the configuration file.
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Stop {},
    Rm {
        /// The key to remove.
        key: PathBuf,
    },
    Ls {
        path: Option<PathBuf>,
    },
}

fn main() {
    setup_logging();
    match Command::parse() {
        Command::Start { config } => {
            let config = config.expect("must provide config");
            let config = Config::from_path(&config).expect("failed to read config");
            let credentials = Credentials::from_env().expect("invalid credentials");
            let stdout = config
                .stdout
                .unwrap_or(PathBuf::from("/tmp/outpost.out"))
                .display()
                .to_string();
            let stderr = config
                .stderr
                .unwrap_or(PathBuf::from("/tmp/outpost.err"))
                .display()
                .to_string();
            let on_update = config.on_update.display().to_string();
            cli::start(stdout, stderr, on_update, credentials).expect("`start` failed");
        }
        Command::Stop {} => {
            cli::stop();
        }
        Command::Rm { key } => {
            let key = key
                .canonicalize()
                .expect("failed to canonicalize path")
                .display()
                .to_string();
            cli::rm(key).expect("`rm` failed");
        }
        Command::Ls { path } => {
            let path = path
                .unwrap_or_else(|| {
                    std::env::current_dir().expect("failed to find current directory")
                })
                .canonicalize()
                .expect("failed to canonicalize path")
                .display()
                .to_string();
            cli::ls(path.as_str()).expect("`ls` failed");
        }
    }
}

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "outpost=debug".into());

    let formatter = tracing_subscriber::fmt::layer().with_timer(UtcTime::new(format_description!(
        "[hour]:[minute]:[second]"
    )));

    tracing_subscriber::registry()
        .with(filter)
        .with(formatter)
        .init()
}
