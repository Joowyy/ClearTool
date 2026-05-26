use crate::core::AppResult;
use crate::models::process::{LockingProcess, ReleaseReport};
use crate::platform::processes;

#[tauri::command]
pub async fn list_processes() -> AppResult<Vec<crate::models::process::ProcessInfo>> {
    processes::list_processes_extended()
}

#[tauri::command]
pub async fn kill_process(pid: u32) -> AppResult<()> {
    processes::kill_process(pid)
}

#[tauri::command]
pub async fn kill_process_tree(pid: u32) -> AppResult<()> {
    processes::kill_process_tree(pid)
}

#[tauri::command]
pub async fn suspend_process(pid: u32) -> AppResult<()> {
    processes::suspend_process(pid)
}

#[tauri::command]
pub async fn resume_process(pid: u32) -> AppResult<()> {
    processes::resume_process(pid)
}

#[tauri::command]
pub async fn close_gracefully(pid: u32, timeout_ms: u32) -> AppResult<bool> {
    processes::close_gracefully(pid, timeout_ms).await
}

#[tauri::command]
pub async fn who_locks_path(path: String) -> AppResult<Vec<LockingProcess>> {
    processes::who_locks_path(std::path::Path::new(&path))
}

#[tauri::command]
pub async fn release_caches() -> AppResult<ReleaseReport> {
    processes::release_common_apps_for_cleanup().await
}
