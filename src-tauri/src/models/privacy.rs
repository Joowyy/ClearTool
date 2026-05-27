// models/privacy.rs — DTOs del módulo Privacy Hardening.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrivacyLevel {
    Balanced,
    Strict,
    Paranoid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyPresetPreview {
    pub level: PrivacyLevel,
    pub registry_tweak_ids: Vec<String>,
    pub service_changes: Vec<ServiceChange>,
    pub estimated_changes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceChange {
    pub service_name: String,
    pub target_start_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyApplyReport {
    pub level: PrivacyLevel,
    pub dry_run: bool,
    pub registry_applied: u32,
    pub registry_failed: u32,
    pub services_applied: u32,
    pub services_failed: u32,
    pub restore_point_seq: Option<u32>,
    pub audit_run_id: String,
    pub errors: Vec<String>,
}
