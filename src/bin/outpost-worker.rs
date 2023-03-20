use clap::Parser;
use outpost::{config::Credentials, worker};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "outpost-worker", version)]
enum Cli {
    Poll {
        #[arg(long)]
        on_update: String,

        #[arg(long)]
        resume: bool,
    },
}

fn main() {
    setup_logging();

    match Cli::parse() {
        Cli::Poll { on_update, resume } => {
            let (program, arguments) = match shlex::split(&on_update) {
                Some(on_update) => {
                    let mut iter = on_update.into_iter();
                    let program = iter
                        .next()
                        .expect("`--on-update` does not specify a command");
                    let arguments: Vec<_> = iter.collect();
                    (program, arguments)
                }
                None => {
                    tracing::error!(
                        "`{}` is not a valid command (passed to `--on-update`)",
                        on_update
                    );
                    return;
                }
            };
            let credentials = Credentials::from_env().expect("invalid credentials");

            worker::poll(program, arguments, resume, credentials).expect("failed to run `poll`");
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
