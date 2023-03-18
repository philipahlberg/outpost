#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::is_process_running;

#[cfg(target_os = "macos")]
pub use macos::is_process_running;
