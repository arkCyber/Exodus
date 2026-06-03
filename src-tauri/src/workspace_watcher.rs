//! ExodusWorkSpace directory watcher — auto-publish new files in `shared/`.

use crate::exodus_workspace::ExodusWorkspace;
use crate::microservice::file_transfer_commands::publish_file_to_workspace;
use crate::p2p_cdn::P2pCdnState;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;

/// Handle to stop the workspace file watcher.
pub struct WorkspaceWatcherHandle {
    stop: Arc<AtomicBool>,
}

impl WorkspaceWatcherHandle {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// Start watching `shared/`; debounced auto-publish via CDN.
pub fn start_workspace_watch(
    app: AppHandle,
    workspace: Arc<ExodusWorkspace>,
    cdn: Arc<P2pCdnState>,
) -> Result<WorkspaceWatcherHandle, String> {
    let shared = workspace.shared_dir();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&stop);

    std::thread::spawn(move || {
        if let Err(e) = run_notify_loop(app, workspace, cdn, shared, stop_flag) {
            tracing::warn!("workspace watcher stopped: {e}");
        }
    });

    Ok(WorkspaceWatcherHandle { stop })
}

fn run_notify_loop(
    app: AppHandle,
    workspace: Arc<ExodusWorkspace>,
    cdn: Arc<P2pCdnState>,
    shared: PathBuf,
    stop: Arc<AtomicBool>,
) -> Result<(), String> {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default(),
    )
    .map_err(|e| e.to_string())?;

    watcher
        .watch(&shared, RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    while !stop.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(event)) => {
                if let Some(path) = event.paths.first() {
                    if path.is_file() {
                        std::thread::sleep(Duration::from_secs(2));
                        let app_clone = app.clone();
                        let ws = Arc::clone(&workspace);
                        let cdn_clone = Arc::clone(&cdn);
                        let path = path.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) =
                                publish_file_to_workspace(&app_clone, &ws, &cdn_clone, &path).await
                            {
                                tracing::debug!("auto-publish skipped: {e}");
                            }
                        });
                    }
                }
            }
            Ok(Err(e)) => tracing::warn!("watch error: {e}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}
