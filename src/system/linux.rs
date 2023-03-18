use procfs::process::Process;

pub fn is_process_running(id: u32) -> bool {
    Process::new(id).is_alive()
}
