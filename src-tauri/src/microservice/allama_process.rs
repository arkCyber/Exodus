//! Exodus Browser — discover and spawn the native Allama `serve` binary.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Child;

/// Default Allama HTTP port (Ollama replacement).
pub const ALLAMA_DEFAULT_PORT: u16 = 11435;

/// Resolve candidate paths for the `allama` CLI binary.
pub fn discover_allama_binary() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ALLAMA_BINARY") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }

    let mut candidates: Vec<PathBuf> = vec![
        PathBuf::from("allama/target/release/allama"),
        PathBuf::from("../allama/target/release/allama"),
        PathBuf::from("../../allama/target/release/allama"),
        PathBuf::from("allama/build/bin/allama"),
        PathBuf::from("allama/build/bin/llama-server"),
    ];

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("allama/target/release/allama"));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join("allama/target/release/allama"));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(mut dir) = exe.parent().map(Path::to_path_buf) {
            for _ in 0..6 {
                candidates.push(dir.join("allama/target/release/allama"));
                if !dir.pop() {
                    break;
                }
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        candidates.push(
            PathBuf::from(&home)
                .join("Allama")
                .join("allama")
                .join("target/release/allama"),
        );
    }

    for path in candidates {
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

/// Whether the binary's `serve` subcommand accepts `--models` (CMake/llama-server style).
fn binary_serve_accepts_models_flag(binary: &Path) -> bool {
    let output = std::process::Command::new(binary)
        .arg("serve")
        .arg("--help")
        .output()
        .ok();
    output
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("--models"))
        .unwrap_or(false)
}

/// Rust allama reads `~/.allama/models`. Symlink Exodus GGUF model dirs when absent there.
pub fn link_exodus_models_into_allama_home(exodus_models: &Path) -> Result<(), String> {
    if !dir_has_gguf(exodus_models) {
        return Ok(());
    }
    let home = std::env::var("HOME").map_err(|e| format!("HOME not set: {e}"))?;
    let native = PathBuf::from(home).join(".allama").join("models");
    std::fs::create_dir_all(&native).map_err(|e| format!("create {}: {e}", native.display()))?;

    for entry in std::fs::read_dir(exodus_models).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let ft = entry.file_type().map_err(|e| e.to_string())?;
        if ft.is_dir() {
            let dest = native.join(entry.file_name());
            if dest.exists() {
                continue;
            }
            #[cfg(unix)]
            std::os::unix::fs::symlink(&path, &dest).map_err(|e| {
                format!(
                    "symlink {} -> {}: {e}",
                    path.display(),
                    dest.display()
                )
            })?;
            #[cfg(not(unix))]
            {
                let _ = path;
                let _ = dest;
            }
        } else if ft.is_file() {
            if path.extension().and_then(|s| s.to_str()) != Some("gguf") {
                continue;
            }
            let dest = native.join(entry.file_name());
            if dest.exists() {
                continue;
            }
            #[cfg(unix)]
            std::os::unix::fs::symlink(&path, &dest).map_err(|e| {
                format!(
                    "symlink {} -> {}: {e}",
                    path.display(),
                    dest.display()
                )
            })?;
        }
    }
    Ok(())
}

/// Resolve bundled `allama/models` next to the repo or executable.
pub fn discover_bundle_models_dir() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("allama/models"),
        PathBuf::from("../allama/models"),
        PathBuf::from("../../allama/models"),
    ];
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("allama/models"));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join("allama/models"));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(mut dir) = exe.parent().map(Path::to_path_buf) {
            for _ in 0..6 {
                candidates.push(dir.join("allama/models"));
                if !dir.pop() {
                    break;
                }
            }
        }
    }
    candidates.into_iter().find(|p| p.is_dir())
}

/// Prefer `app_data_dir/allama/models` (always returned when `app_data_dir` is set).
/// Bundled `allama/models` is discovered separately via [`discover_bundle_models_dir`].
pub fn resolve_models_dir(app_data_dir: Option<&Path>) -> PathBuf {
    if let Some(app_data) = app_data_dir {
        let app_models = app_data.join("allama").join("models");
        if std::fs::create_dir_all(&app_models).is_err() {
            eprintln!(
                "[Allama] failed to create models dir: {}",
                app_models.display()
            );
        }
        if !dir_has_gguf(&app_models) {
            if let Some(bundle) = discover_bundle_models_dir() {
                if dir_has_gguf(&bundle) {
                    println!(
                        "[Allama] app models dir empty; bundled GGUF at {} (copy into {})",
                        bundle.display(),
                        app_models.display()
                    );
                }
            }
        }
        return app_models;
    }
    discover_bundle_models_dir().unwrap_or_else(|| PathBuf::from("allama/models"))
}

fn dir_has_gguf(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    walkdir::WalkDir::new(dir)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("gguf"))
}

/// Spawn `allama serve` (or compatible binary) listening on `port`.
pub async fn spawn_allama_serve(
    binary: &Path,
    port: u16,
    models_dir: &Path,
) -> Result<Child, String> {
    let mut cmd = tokio::process::Command::new(binary);
    cmd.arg("serve")
        .arg("--port")
        .arg(port.to_string())
        .arg("--host")
        .arg("127.0.0.1");

    if models_dir.is_dir() {
        if binary_serve_accepts_models_flag(binary) {
            cmd.arg("--models").arg(models_dir);
        } else {
            link_exodus_models_into_allama_home(models_dir)?;
            cmd.env(
                "ALLAMA_INFERENCE_MODELS_DIR",
                models_dir.to_string_lossy().to_string(),
            );
        }
    }

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    cmd.spawn()
        .map_err(|e| format!("Failed to spawn Allama at {}: {}", binary.display(), e))
}

/// Probe whether Allama HTTP API responds on the given port.
pub async fn probe_allama_http(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/api/tags", port);
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    client.get(&url).send().await.map(|r| r.status().is_success()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rust_allama_serve_rejects_models_flag() {
        let binary = discover_allama_binary();
        let Some(bin) = binary else {
            eprintln!("skip: no allama binary for serve --help probe");
            return;
        };
        assert!(
            !binary_serve_accepts_models_flag(&bin),
            "Rust allama should not accept --models on serve"
        );
    }

    #[test]
    fn link_exodus_models_skips_when_no_gguf() {
        let tmp = std::env::temp_dir().join(format!("exodus-allama-link-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        link_exodus_models_into_allama_home(&tmp).expect("empty dir ok");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_models_dir_creates_app_data_path() {
        let tmp = std::env::temp_dir().join(format!("exodus-allama-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let resolved = resolve_models_dir(Some(&tmp));
        assert!(resolved.ends_with("allama/models"));
        assert!(resolved.starts_with(&tmp));
        assert!(resolved.is_dir());
        let _ = fs::remove_dir_all(&tmp);
    }
}
