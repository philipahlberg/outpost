use crate::{
    database::{Process, PROCESSES},
    system::is_process_running,
};

#[derive(Debug)]
pub enum RmError {
    KeyNotPresent,
    HomeDirectoryMissing,
    ProcessRunning,
    Database(sled::Error),
}

pub fn rm(key: String) -> Result<(), RmError> {
    let outpost_dir = home::home_dir()
        .ok_or(RmError::HomeDirectoryMissing)?
        .join(".outpost");

    let database_dir = outpost_dir.join("database");

    let processes = sled::open(database_dir)
        .map_err(RmError::Database)?
        .open_tree(PROCESSES)
        .map_err(RmError::Database)?;

    let process = processes
        .get(key.as_bytes())
        .map_err(RmError::Database)?
        .ok_or(RmError::KeyNotPresent)?;

    let process: Process = serde_json::from_slice(&process).expect("invalid JSON");

    if let Some(id) = process.process_id() {
        if is_process_running(id) {
            return Err(RmError::ProcessRunning);
        }
    }

    processes
        .remove(key.as_bytes())
        .map_err(RmError::Database)?
        .ok_or(RmError::KeyNotPresent)?;

    Ok(())
}
