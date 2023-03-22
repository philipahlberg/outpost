use std::{fs::File, io, process::Command};

use crate::{
    config::Credentials,
    database::{v1, Process, PROCESSES},
};

#[derive(Debug)]
pub enum StartError {
    HomeDirectoryMissing,
    CurrentDirectory(io::Error),
    Database(sled::Error),
    ExistingEntry(Process),
    Stdout(io::Error),
    Stderr(io::Error),
    Spawn(io::Error),
}

#[cfg(debug_assertions)]
const OUTPOST_WORKER: &str = "./target/debug/outpost-worker";

#[cfg(not(debug_assertions))]
const OUTPOST_WORKER: &str = "outpost-worker";

pub fn start(
    stdout: String,
    stderr: String,
    on_update: String,
    updates: String,
    iterations: Option<usize>,
    interval: Option<u64>,
    credentials: Option<Credentials>,
) -> Result<(), StartError> {
    let outpost_dir = home::home_dir()
        .ok_or(StartError::HomeDirectoryMissing)?
        .join(".outpost");

    let database_dir = outpost_dir.join("database");

    let current_dir = std::env::current_dir()
        .map_err(StartError::CurrentDirectory)?
        .display()
        .to_string();

    let processes = sled::open(&database_dir)
        .map_err(StartError::Database)?
        .open_tree(PROCESSES)
        .map_err(StartError::Database)?;

    let entry = processes
        .get(current_dir.as_bytes())
        .map_err(StartError::Database)?
        .map(|value| serde_json::from_slice(value.as_ref()).expect("valid json"));

    drop(processes);

    if let Some(process) = entry {
        return Err(StartError::ExistingEntry(process));
    }

    tracing::debug!("Starting worker process.");

    let worker = {
        let stdout = File::create(&stdout).map_err(StartError::Stdout)?;
        let stderr = File::create(&stderr).map_err(StartError::Stderr)?;

        let mut command = Command::new(OUTPOST_WORKER);

        command
            .args([
                "poll",
                "--on-update",
                on_update.as_str(),
                "--updates",
                updates.as_str(),
            ])
            .stdout(stdout)
            .stderr(stderr);

        if let Some(n) = iterations {
            command.args(["--iterations", n.to_string().as_str()]);
        }

        if let Some(interval) = interval {
            command.args(["--interval", interval.to_string().as_str()]);
        }

        if let Some(c) = credentials {
            command.env("GIT_USERNAME", c.username);
            command.env("GIT_PASSWORD", c.password);
        }

        command.spawn().map_err(StartError::Spawn)?
    };

    tracing::debug!("Worker process started (ID: {}).", worker.id());

    let process = Process::V1(v1::Process {
        directory: current_dir.to_string(),
        stdout,
        stderr,
        process_id: Some(worker.id()),
    });

    let processes = sled::open(&database_dir)
        .map_err(StartError::Database)?
        .open_tree(PROCESSES)
        .map_err(StartError::Database)?;

    processes
        .insert(
            current_dir.as_bytes(),
            serde_json::to_vec(&process).expect("failed to serialize process"),
        )
        .expect("failed to insert process");

    tracing::debug!("Worker process registered.");

    Ok(())
}
