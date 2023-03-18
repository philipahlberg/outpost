use crate::database::{Process, PROCESSES};

#[derive(Debug)]
pub enum LsError {
    HomeDirectoryMissing,
    Database(sled::Error),
}

pub fn ls(path: &str) -> Result<(), LsError> {
    let outpost_dir = home::home_dir()
        .ok_or(LsError::HomeDirectoryMissing)?
        .join(".outpost");

    let database_dir = outpost_dir.join("database");

    let processes = sled::open(&database_dir)
        .map_err(LsError::Database)?
        .open_tree(PROCESSES)
        .map_err(LsError::Database)?;

    let values: Vec<_> = processes
        .scan_prefix(path)
        .map(|b| {
            let (key, value) = b.expect("invalid entry");
            let key = String::from_utf8(key.to_vec()).expect("valid utf8");
            let value: Process = serde_json::from_slice(value.as_ref()).expect("valid json");
            (key, value)
        })
        .collect();

    for (key, process) in values {
        println!("{key}: {process:?}");
    }

    Ok(())
}
