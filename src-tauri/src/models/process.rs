use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub display_name: Option<String>,
    pub exe_path: Option<String>,
    pub session_id: u32,
    pub start_time: Option<String>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub thread_count: u32,
    pub handle_count: u32,
    pub command_line: Option<String>,
    pub user_sid: Option<String>,
    pub user_name: Option<String>,
    pub is_uwp: bool,
    pub uwp_package_family: Option<String>,
    pub category: ProcessCategory,
    pub is_protected: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessCategory {
    System,
    Service,
    Browser,
    Communication,
    Media,
    Development,
    Background,
    UserApp,
    Unknown,
}

impl Default for ProcessCategory {
    fn default() -> Self {
        ProcessCategory::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseReport {
    pub closed_count: u32,
    pub failed_count: u32,
    pub closed_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockingProcess {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
}
