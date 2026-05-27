// domain/throughput_stats.rs — buffer circular de muestras de throughput.
// Persiste las últimas 20 ejecuciones en %APPDATA%\ClearTool\throughput-stats.json.

use crate::core::AppResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThroughputStats {
    pub samples: Vec<u64>,
    pub mean_bytes_per_sec: u64,
    pub p95_bytes_per_sec: u64,
}

const MAX_SAMPLES: usize = 20;
pub const DEFAULT_BPS: u64 = 100 * 1024 * 1024;

fn stats_path() -> PathBuf {
    let mut p = dirs::data_dir().unwrap_or_else(|| {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    });
    p.push("ClearTool");
    let _ = fs::create_dir_all(&p);
    p.push("throughput-stats.json");
    p
}

pub fn load() -> ThroughputStats {
    let path = stats_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn push_sample(bps: u64) -> AppResult<()> {
    if bps == 0 {
        return Ok(());
    }
    let mut stats = load();
    stats.samples.push(bps);
    if stats.samples.len() > MAX_SAMPLES {
        let drain_count = stats.samples.len() - MAX_SAMPLES;
        stats.samples.drain(0..drain_count);
    }
    recompute(&mut stats);
    let path = stats_path();
    fs::write(&path, serde_json::to_string_pretty(&stats)?)?;
    Ok(())
}

fn recompute(stats: &mut ThroughputStats) {
    if stats.samples.is_empty() {
        stats.mean_bytes_per_sec = DEFAULT_BPS;
        stats.p95_bytes_per_sec = DEFAULT_BPS;
        return;
    }
    let sum: u64 = stats.samples.iter().sum();
    stats.mean_bytes_per_sec = sum / stats.samples.len() as u64;
    let mut sorted = stats.samples.clone();
    sorted.sort_unstable();
    let p95_idx = ((sorted.len() as f32) * 0.95).floor() as usize;
    stats.p95_bytes_per_sec = sorted[p95_idx.min(sorted.len() - 1)];
}

pub fn current() -> ThroughputStats {
    let mut s = load();
    if s.samples.is_empty() {
        s.mean_bytes_per_sec = DEFAULT_BPS;
        s.p95_bytes_per_sec = DEFAULT_BPS;
    }
    s
}
