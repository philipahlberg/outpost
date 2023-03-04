use std::fs::File;

use clap::Parser;
use daemonize::Daemonize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "outpost", version)]
enum Command {
    Setup {
        name: String,
    },
    Start {
        name: String,

        /// The file to write the daemon PID into.
        #[arg(long = "pid-file")]
        pid_file: Option<String>,

        /// The file to write all stdout into.
        #[arg(long)]
        stdout: Option<String>,

        /// The file to write all stderr into.
        #[arg(long)]
        stderr: Option<String>,

        /// The umask to apply to the process.
        #[arg(long)]
        umask: Option<u32>,
    },
    Stop {
        name: String,
    },
}

fn main() {
    setup_logging();
    match Command::parse() {
        Command::Setup { name } => {
            tracing::info!("setup {}", name);
        }
        Command::Start {
            name,
            pid_file,
            stdout,
            stderr,
            umask,
        } => {
            tracing::info!("start {}", name);

            let stdout = stdout.unwrap_or("/tmp/outpost.out".to_string());
            let stderr = stderr.unwrap_or("/tmp/outpost.err".to_string());
            let pid_file = pid_file.unwrap_or("/tmp/outpost.pid".to_string());

            let stdout = File::create(stdout).unwrap();
            let stderr = File::create(stderr).unwrap();

            let daemonize = Daemonize::new()
                .pid_file(pid_file)
                .working_directory("/tmp")
                .umask(umask.unwrap_or(0o027))
                .stdout(stdout)
                .stderr(stderr);

            tracing::debug!("Starting daemon.");

            match daemonize.start() {
                Ok(()) => {
                    tracing::debug!("Running in separate process.");
                }
                Err(e) => eprintln!("Error, {}", e),
            }
        }
        Command::Stop { name } => {
            tracing::info!("stop {}", name);
        }
    }
}

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "outpost=debug".into());

    let formatter = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(filter)
        .with(formatter)
        .init()
}
