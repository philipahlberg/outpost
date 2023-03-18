use libproc::libproc::task_info::TaskAllInfo;

pub fn is_process_running(id: u32) -> bool {
    libproc::libproc::proc_pid::pidinfo::<TaskAllInfo>(id as i32, 0).is_ok()
}
