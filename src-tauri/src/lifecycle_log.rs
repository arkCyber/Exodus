//! Exodus Browser — structured lifecycle logging for post-mortem analysis.
//!
//! Writes timestamped lines to stderr (tracing), `startup.log`, and `lifecycle.log`
//! under `{app_data}/logs/`. Keeps a bounded in-memory ring for `lifecycle_get_logs`.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Maximum in-memory log entries retained for API export.
const MAX_MEMORY_ENTRIES: usize = 256;

/// Log severity for lifecycle events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Category groups events for filtering during analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleLogCategory {
    Phase,
    Check,
    Tick,
    Preset,
    Remediation,
    Scheduler,
    System,
}

/// One structured lifecycle log entry (JSON-serializable for frontend export).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleLogEntry {
    pub at: String,
    pub level: LifecycleLogLevel,
    pub category: LifecycleLogCategory,
    pub message: String,
    pub detail: Option<String>,
}

static LOG_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
static MEMORY: Mutex<VecDeque<LifecycleLogEntry>> = Mutex::new(VecDeque::new());

/// Initialize lifecycle file logging (call after `startup_log::set_file_log_dir`).
pub fn init_lifecycle_log_dir(app_data_dir: &Path) {
    let logs = app_data_dir.join("logs");
    let _ = std::fs::create_dir_all(&logs);
    if let Ok(mut guard) = LOG_DIR.lock() {
        *guard = Some(logs);
    }
    log(
        LifecycleLogLevel::Info,
        LifecycleLogCategory::System,
        "lifecycle log initialized",
        None,
    );
}

fn append_lifecycle_file(line: &str) {
    let dir = LOG_DIR.lock().ok().and_then(|g| g.clone());
    let Some(dir) = dir else {
        return;
    };
    let path = dir.join("lifecycle.log");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

fn push_memory(entry: LifecycleLogEntry) {
    if let Ok(mut buf) = MEMORY.lock() {
        buf.push_back(entry);
        while buf.len() > MAX_MEMORY_ENTRIES {
            buf.pop_front();
        }
    }
}

fn level_str(level: LifecycleLogLevel) -> &'static str {
    match level {
        LifecycleLogLevel::Debug => "DEBUG",
        LifecycleLogLevel::Info => "INFO",
        LifecycleLogLevel::Warn => "WARN",
        LifecycleLogLevel::Error => "ERROR",
    }
}

fn category_str(category: LifecycleLogCategory) -> &'static str {
    match category {
        LifecycleLogCategory::Phase => "phase",
        LifecycleLogCategory::Check => "check",
        LifecycleLogCategory::Tick => "tick",
        LifecycleLogCategory::Preset => "preset",
        LifecycleLogCategory::Remediation => "remediation",
        LifecycleLogCategory::Scheduler => "scheduler",
        LifecycleLogCategory::System => "system",
    }
}

/// Core log function — records to memory, lifecycle.log, tracing, and startup.log.
pub fn log(
    level: LifecycleLogLevel,
    category: LifecycleLogCategory,
    message: &str,
    detail: Option<&str>,
) {
    let at = Utc::now().to_rfc3339();
    let entry = LifecycleLogEntry {
        at: at.clone(),
        level,
        category,
        message: message.to_string(),
        detail: detail.map(str::to_string),
    };

    let line = match &entry.detail {
        Some(d) => format!(
            "[{at}][exodus_lifecycle][{}][{}] {message} | {d}",
            level_str(level),
            category_str(category)
        ),
        None => format!(
            "[{at}][exodus_lifecycle][{}][{}] {message}",
            level_str(level),
            category_str(category)
        ),
    };

    push_memory(entry);

    match level {
        LifecycleLogLevel::Debug => tracing::debug!(target: "exodus_lifecycle", "{message}"),
        LifecycleLogLevel::Info => tracing::info!(target: "exodus_lifecycle", "{message}"),
        LifecycleLogLevel::Warn => tracing::warn!(target: "exodus_lifecycle", "{message}"),
        LifecycleLogLevel::Error => tracing::error!(target: "exodus_lifecycle", "{message}"),
    }

    append_lifecycle_file(&line);
    crate::startup_log::log_step(&format!(
        "lifecycle[{}/{}] {message}{}",
        level_str(level),
        category_str(category),
        detail.map(|d| format!(" ({d})")).unwrap_or_default()
    ));
}

/// Log lifecycle phase transition.
pub fn log_phase(phase: &str, previous: Option<&str>) {
    let detail = previous.map(|p| format!("from={p}"));
    log(
        LifecycleLogLevel::Info,
        LifecycleLogCategory::Phase,
        &format!("phase → {phase}"),
        detail.as_deref(),
    );
}

/// Log a component health check result.
pub fn log_check(component: &str, health: &str, message: &str) {
    log(
        LifecycleLogLevel::Info,
        LifecycleLogCategory::Check,
        &format!("{component} health={health}"),
        Some(message),
    );
}

/// Log scheduler tick summary.
pub fn log_tick(tick: u64, phase: &str, degraded: bool) {
    log(
        LifecycleLogLevel::Debug,
        LifecycleLogCategory::Tick,
        &format!("tick={tick} phase={phase} degraded={degraded}"),
        None,
    );
}

/// Log planned preset remediations before execution.
pub fn log_preset_plan(preset_ids: &[&str], reason: &str) {
    log(
        LifecycleLogLevel::Info,
        LifecycleLogCategory::Preset,
        &format!("playbook plan: [{}]", preset_ids.join(", ")),
        Some(reason),
    );
}

/// Log result of a preset remediation action.
pub fn log_remediation(preset_id: &str, success: bool, detail: &str) {
    log(
        if success {
            LifecycleLogLevel::Info
        } else {
            LifecycleLogLevel::Warn
        },
        LifecycleLogCategory::Remediation,
        &format!("preset={preset_id} success={success}"),
        Some(detail),
    );
}

/// Return recent log entries (newest last), optionally limited (0 ⇒ empty).
pub fn recent_entries(limit: usize) -> Vec<LifecycleLogEntry> {
    if limit == 0 {
        return Vec::new();
    }
    let buf = MEMORY.lock().ok();
    let Some(buf) = buf else {
        return Vec::new();
    };
    let n = limit.min(buf.len()).min(MAX_MEMORY_ENTRIES);
    buf.iter().skip(buf.len().saturating_sub(n)).cloned().collect()
}

#[cfg(test)]
pub fn clear_memory_for_tests() {
    if let Ok(mut buf) = MEMORY.lock() {
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_entry_serializes() {
        let e = LifecycleLogEntry {
            at: "2026-01-01T00:00:00Z".into(),
            level: LifecycleLogLevel::Info,
            category: LifecycleLogCategory::Check,
            message: "test".into(),
            detail: None,
        };
        let j = serde_json::to_string(&e).expect("serialize");
        assert!(j.contains("check"));
        assert!(j.contains("\"level\":\"info\""));
    }

    #[test]
    fn recent_entries_returns_last_n() {
        clear_memory_for_tests();
        log(LifecycleLogLevel::Info, LifecycleLogCategory::System, "only-entry", None);
        let entries = recent_entries(32);
        assert!(
            entries.iter().any(|e| e.message == "only-entry"),
            "expected only-entry in {:?}",
            entries.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn preset_plan_log_stored() {
        clear_memory_for_tests();
        // Must clear: other tests may fill the ring buffer when run in parallel.
        log_preset_plan(&["show_main_window", "reload_frontend"], "unit test");
        let entries = recent_entries(8);
        assert!(entries.iter().any(|e| e.message.contains("playbook plan")));
    }

    #[test]
    fn recent_entries_zero_limit_returns_empty() {
        clear_memory_for_tests();
        log(LifecycleLogLevel::Info, LifecycleLogCategory::System, "x", None);
        assert!(recent_entries(0).is_empty());
    }

    #[test]
    fn memory_ring_bounded_at_max_entries() {
        clear_memory_for_tests();
        // Isolate from parallel tests sharing static MEMORY.
        for i in 0..(MAX_MEMORY_ENTRIES + 16) {
            log(
                LifecycleLogLevel::Info,
                LifecycleLogCategory::System,
                &format!("line-{i}"),
                None,
            );
        }
        let all = recent_entries(MAX_MEMORY_ENTRIES);
        assert_eq!(all.len(), MAX_MEMORY_ENTRIES);
        let last = all.last().expect("ring has entries");
        assert!(
            last.message.starts_with("line-"),
            "expected ring tail line-* got {}",
            last.message
        );
    }
}
