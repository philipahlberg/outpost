use procfs::process::Process;

pub fn is_process_running(id: u32) -> bool {
    Process::new(id as i32)
        .map(|p| p.is_alive())
        .unwrap_or(false)
}
