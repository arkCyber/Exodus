//! Exodus Browser — timestamped startup logging (stderr + optional app_data file).
//!
//! Use `log_step` at each major init phase so `tauri dev` / Console.app show where boot stalls.

use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static FILE_LOG_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Initialize `tracing` subscriber (call once from `run()` before `Builder`).
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_target(true)
        .try_init();
    log_step("tracing subscriber initialized");
}

/// Enable append-only `startup.log` under `{app_data}/logs/`.
pub fn set_file_log_dir(app_data_dir: &Path) {
    let logs = app_data_dir.join("logs");
    let _ = std::fs::create_dir_all(&logs);
    let log_path = logs.display().to_string();
    if let Ok(mut guard) = FILE_LOG_DIR.lock() {
        *guard = Some(logs);
    }
    log_step(&format!("file log dir: {log_path}"));
}

fn append_file(line: &str) {
    let dir = FILE_LOG_DIR.lock().ok().and_then(|g| g.clone());
    let Some(dir) = dir else { return };
    let path = dir.join("startup.log");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

fn format_line(level: &str, message: &str) -> String {
    format!("[{}][exodus_startup][{level}] {message}", Utc::now().to_rfc3339())
}

/// Info-level startup milestone.
pub fn log_step(message: &str) {
    let line = format_line("INFO", message);
    tracing::info!(target: "exodus_startup", "{message}");
    append_file(&line);
}

/// Warning during startup (non-fatal).
pub fn log_warn(message: &str) {
    let line = format_line("WARN", message);
    tracing::warn!(target: "exodus_startup", "{message}");
    append_file(&line);
}

/// Error during startup.
pub fn log_error(message: &str) {
    let line = format_line("ERROR", message);
    tracing::error!(target: "exodus_startup", "{message}");
    append_file(&line);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_line_includes_level_and_message() {
        let s = format_line("INFO", "hello");
        assert!(s.contains("exodus_startup"));
        assert!(s.contains("hello"));
    }
}
