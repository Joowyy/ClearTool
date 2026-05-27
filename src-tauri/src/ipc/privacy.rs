// ipc/privacy.rs — comandos Tauri para Privacy Hardening.

use crate::core::AppResult;
use crate::domain;
use crate::models::privacy::{PrivacyApplyReport, PrivacyLevel, PrivacyPresetPreview};

#[tauri::command]
pub async fn get_privacy_preset_preview(level: PrivacyLevel) -> AppResult<PrivacyPresetPreview> {
    domain::privacy::preview(level)
}

#[tauri::command]
pub async fn apply_privacy_preset(level: PrivacyLevel, dry_run: bool) -> AppResult<PrivacyApplyReport> {
    domain::privacy::apply(level, dry_run)
}
