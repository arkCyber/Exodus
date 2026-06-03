//! Exodus Browser — application lifecycle monitor, scheduler, and auto-remediation.
//!
//! Detects unhealthy components (window, frontend, Allama, sidecar), logs structured
//! events via [`crate::lifecycle_log`], and runs preset remediation playbooks when
//! health checks fail.
//!
//! # Aerospace-grade invariants (verified in `mod tests`)
//! - **I1** No `panic!` on mutex poison — poisoned locks are recovered via `into_inner`.
//! - **I2** Bounded memory: remediation history ≤ 32, log export ≤ 256.
//! - **I3** Preset playbooks have unique ids; output has unique `cooldown_key`.
//! - **I4** `ShuttingDown` is terminal — no auto-remediation while shutting down.
//! - **I5** Phase recompute: any `Warn` or `Error` component ⇒ `Degraded`; all clear ⇒ `Running`.
//! - **I6** Cooldown 45s per action key; `full_ui_recovery` applies cooldown only after final step.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, RunEvent, Url};
use tokio::time::interval;

use crate::allama_manager::AllamaManager;
use crate::app_window;
use crate::config::ConfigState;
use crate::lifecycle_log::{self, LifecycleLogEntry};
use crate::sidecar::{SidecarManager, probe_models_endpoint};
use crate::startup_log;

/// Seconds between repeated attempts of the same remediation action.
const REMEDIATION_COOLDOWN_SECS: u64 = 45;

/// Max remediation log entries kept in memory.
const MAX_REMEDIATION_HISTORY: usize = 32;

/// Max entries returned by `lifecycle_get_logs`.
const MAX_LOG_EXPORT: usize = 256;

/// High-level application lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecyclePhase {
    Booting,
    Setup,
    Ready,
    Running,
    Background,
    Degraded,
    ShuttingDown,
}

/// Health of a monitored component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentHealth {
    Ok,
    Warn,
    Error,
    Unknown,
}

/// Snapshot of one monitored component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSnapshot {
    pub name: String,
    pub health: ComponentHealth,
    pub message: String,
    pub checked_at: String,
}

/// Record of an automatic or manual remediation attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationRecord {
    /// Preset playbook id (e.g. `show_main_window`).
    pub preset_id: String,
    pub action: String,
    pub success: bool,
    pub detail: String,
    pub at: String,
}

/// Preset remediation scheme exposed to the frontend / diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationPresetDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub component: String,
    pub triggers_on: String,
    pub priority: u8,
    pub cooldown_secs: u64,
}

/// Built-in preset definition (playbook entry).
#[derive(Debug, Clone, Copy)]
struct RemediationPresetDef {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    component: &'static str,
    triggers_on: ComponentHealth,
    cooldown_key: &'static str,
    priority: u8,
}

/// Preset remediation playbooks — ordered by `priority` when multiple match.
const REMEDIATION_PRESETS: &[RemediationPresetDef] = &[
    // Disabled automatic window activation to prevent interference
    // Window will only be activated when explicitly requested by user
    /*
    RemediationPresetDef {
        id: "show_main_window",
        name: "Show Main Window",
        description: "Re-apply macOS dock policy, show and focus the main window.",
        component: "main_window",
        triggers_on: ComponentHealth::Warn,
        cooldown_key: "show_main_window",
        priority: 10,
    },
    */
    RemediationPresetDef {
        id: "reload_frontend",
        name: "Reload Frontend",
        description: "Navigate main webview to dev/prod URL when the frontend is unreachable.",
        component: "frontend",
        triggers_on: ComponentHealth::Warn,
        cooldown_key: "reload_frontend",
        priority: 20,
    },
    RemediationPresetDef {
        id: "reload_on_hidden_window",
        name: "Reload When Window Hidden",
        description: "Reload UI when the window is hidden but the dev server is healthy.",
        component: "main_window",
        triggers_on: ComponentHealth::Warn,
        cooldown_key: "reload_frontend",
        priority: 25,
    },
    RemediationPresetDef {
        id: "restart_allama",
        name: "Restart Allama",
        description: "Restart the Allama HTTP service when it stops responding.",
        component: "allama",
        triggers_on: ComponentHealth::Error,
        cooldown_key: "restart_allama",
        priority: 30,
    },
    RemediationPresetDef {
        id: "restart_sidecar",
        name: "Restart AI Sidecar",
        description: "Restart the AI sidecar when models endpoint is down.",
        component: "sidecar",
        triggers_on: ComponentHealth::Warn,
        cooldown_key: "restart_sidecar",
        priority: 40,
    },
    RemediationPresetDef {
        id: "full_ui_recovery",
        name: "Full UI Recovery",
        description: "Escalation: show window then reload frontend when both UI checks fail.",
        component: "_ui_stack",
        triggers_on: ComponentHealth::Error,
        cooldown_key: "full_ui_recovery",
        priority: 5,
    },
];

/// Public status DTO for the frontend / diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleStatusDto {
    pub phase: LifecyclePhase,
    pub started_at: String,
    pub uptime_secs: u64,
    pub launch_mode: String,
    pub scheduler_active: bool,
    pub tick_count: u64,
    pub auto_fix_enabled: bool,
    pub components: Vec<ComponentSnapshot>,
    pub recent_remediations: Vec<RemediationRecord>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchMode {
    DevBinary,
    BundledApp,
}

impl LaunchMode {
    fn as_str(self) -> &'static str {
        match self {
            LaunchMode::DevBinary => "dev_binary",
            LaunchMode::BundledApp => "bundled_app",
        }
    }
}

struct LifecycleInner {
    phase: LifecyclePhase,
    started: Instant,
    started_at: String,
    launch_mode: LaunchMode,
    scheduler_active: bool,
    tick_count: u64,
    auto_fix_enabled: bool,
    components: HashMap<String, ComponentSnapshot>,
    recent_remediations: VecDeque<RemediationRecord>,
    remediation_cooldown: HashMap<String, Instant>,
    last_error: Option<String>,
}

/// Thread-safe lifecycle manager (managed by Tauri).
pub struct AppLifecycleManager {
    inner: Mutex<LifecycleInner>,
}

impl AppLifecycleManager {
    /// Lock inner state; recover from poison instead of panicking (invariant I1).
    fn lock_inner(&self) -> std::sync::MutexGuard<'_, LifecycleInner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Current lifecycle phase (defaults to `Booting` if lock unavailable).
    fn current_phase(&self) -> LifecyclePhase {
        self.lock_inner().phase
    }

    /// Create manager at the very start of `run()`.
    pub fn new() -> Self {
        let launch_mode = detect_launch_mode();
        startup_log::log_step(&format!(
            "AppLifecycleManager created (launch_mode={})",
            launch_mode.as_str()
        ));
        let started_at = Utc::now().to_rfc3339();
        Self {
            inner: Mutex::new(LifecycleInner {
                phase: LifecyclePhase::Booting,
                started: Instant::now(),
                started_at,
                launch_mode,
                scheduler_active: false,
                tick_count: 0,
                auto_fix_enabled: true,
                components: HashMap::new(),
                recent_remediations: VecDeque::new(),
                remediation_cooldown: HashMap::new(),
                last_error: None,
            }),
        }
    }

    /// Transition lifecycle phase and log.
    pub fn set_phase(&self, phase: LifecyclePhase) {
        let mut g = self.lock_inner();
        let prev = format!("{:?}", g.phase);
        g.phase = phase;
        let next = format!("{:?}", phase);
        lifecycle_log::log_phase(&next, Some(&prev));
        startup_log::log_step(&format!("lifecycle phase → {next}"));
    }

    /// Record component check result.
    pub fn set_component(&self, name: &str, health: ComponentHealth, message: impl Into<String>) {
        let message = message.into();
        if matches!(health, ComponentHealth::Warn | ComponentHealth::Error) {
            lifecycle_log::log_check(name, &format!("{health:?}"), &message);
        }
        let mut g = self.lock_inner();
        g.components.insert(
            name.to_string(),
            ComponentSnapshot {
                name: name.to_string(),
                health,
                message,
                checked_at: Utc::now().to_rfc3339(),
            },
        );
    }

    fn component_health(&self, name: &str) -> ComponentHealth {
        self.lock_inner()
            .components
            .get(name)
            .map(|c| c.health)
            .unwrap_or(ComponentHealth::Unknown)
    }

    fn can_remediate(&self, action_key: &str) -> bool {
        let g = self.lock_inner();
        if !g.auto_fix_enabled {
            return false;
        }
        match g.remediation_cooldown.get(action_key) {
            Some(t) => t.elapsed() >= Duration::from_secs(REMEDIATION_COOLDOWN_SECS),
            None => true,
        }
    }

    fn record_remediation(
        &self,
        preset_id: &str,
        action_key: &str,
        action: &str,
        success: bool,
        detail: impl Into<String>,
        apply_cooldown: bool,
    ) {
        let detail = detail.into();
        let mut g = self.lock_inner();
        if apply_cooldown {
            g.remediation_cooldown
                .insert(action_key.to_string(), Instant::now());
        }
        g.recent_remediations.push_back(RemediationRecord {
            preset_id: preset_id.to_string(),
            action: action.to_string(),
            success,
            detail: detail.clone(),
            at: Utc::now().to_rfc3339(),
        });
        while g.recent_remediations.len() > MAX_REMEDIATION_HISTORY {
            g.recent_remediations.pop_front();
        }
        lifecycle_log::log_remediation(preset_id, success, &detail);
        let level = if success { "ok" } else { "failed" };
        startup_log::log_step(&format!("auto-fix [{preset_id}/{action}] {level}: {detail}"));
    }

    fn recompute_phase(&self) {
        let mut g = self.lock_inner();
        g.phase = recompute_phase_pure(g.phase, g.components.values().map(|c| c.health));
    }

    /// Export current status.
    pub fn status(&self) -> LifecycleStatusDto {
        let g = self.lock_inner();
        let mut components: Vec<_> = g.components.values().cloned().collect();
        components.sort_by(|a, b| a.name.cmp(&b.name));
        LifecycleStatusDto {
            phase: g.phase,
            started_at: g.started_at.clone(),
            uptime_secs: g.started.elapsed().as_secs(),
            launch_mode: g.launch_mode.as_str().to_string(),
            scheduler_active: g.scheduler_active,
            tick_count: g.tick_count,
            auto_fix_enabled: g.auto_fix_enabled,
            components,
            recent_remediations: g.recent_remediations.iter().cloned().collect(),
            last_error: g.last_error.clone(),
        }
    }

    /// Start background scheduler (health checks + auto-remediation).
    pub fn start_scheduler(self: Arc<Self>, app: AppHandle) {
        {
            let mut g = self.lock_inner();
            if g.scheduler_active {
                return;
            }
            g.scheduler_active = true;
        }
        // Re-enabled scheduler with optimized window state checks
        // Window state checks are now efficient and don't cause cursor spinning
        startup_log::log_step("lifecycle scheduler started (5s fast / 30s steady, auto-fix on)");
        tauri::async_runtime::spawn(async move {
            let mut fast = interval(Duration::from_secs(5));
            let mut ticks: u64 = 0;
            for _ in 0..12 {
                fast.tick().await;
                ticks += 1;
                run_scheduled_tick(&app, &self, ticks).await;
            }
            let mut slow = interval(Duration::from_secs(30));
            loop {
                slow.tick().await;
                ticks += 1;
                run_scheduled_tick(&app, &self, ticks).await;
            }
        });
    }

    /// Handle Tauri run events.
    pub fn on_run_event(self: &Arc<Self>, app: &AppHandle, event: &RunEvent) {
        // Log all events for debugging
        match event {
            RunEvent::Ready => startup_log::log_step("RunEvent: Ready"),
            RunEvent::Reopen { .. } => startup_log::log_step("RunEvent: Reopen (dock icon click)"),
            RunEvent::ExitRequested { .. } => startup_log::log_step("RunEvent: ExitRequested"),
            _ => {}
        }

        match event {
            RunEvent::Ready => {
                self.set_phase(LifecyclePhase::Ready);
                startup_log::log_step("RunEvent: Ready — ensure shell URL and window visible");
                crate::app_window::ensure_main_window_ready(app);

                // Disabled window position setting in RunEvent::Ready to prevent interference with lib.rs
                // The window position is now controlled by lib.rs after tray setup
                /*
                startup_log::log_step("RunEvent: Ready - forcing window size and position from config");

                // Force window position and size to ignore saved state
                if let Some(win) = app.get_webview_window("main") {
                    // Try multiple times with delays to ensure the setting sticks
                    let win_clone = win.clone();
                    let app_handle = app.clone();

                    // First attempt immediately
                    match win_clone.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: 1280,
                        height: 720,
                    })) {
                        Ok(_) => startup_log::log_step("RunEvent::Ready: set_size(1280x720) succeeded (Physical)"),
                        Err(e) => startup_log::log_error(&format!("RunEvent::Ready: set_size(1280x720) failed (Physical): {}", e)),
                    }
                    match win_clone.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: 60,
                        y: 60,
                    })) {
                        Ok(_) => startup_log::log_step("RunEvent::Ready: set_position(60, 60) succeeded (Physical)"),
                        Err(e) => startup_log::log_error(&format!("RunEvent::Ready: set_position(60, 60) failed (Physical): {}", e)),
                    }
                    let _ = win_clone.unminimize();

                    // Log the actual window state after setting
                    if let Ok(pos) = win_clone.outer_position() {
                        startup_log::log_step(&format!("RunEvent::Ready: Window position after set: x={}, y={}", pos.x, pos.y));
                    }
                    if let Ok(size) = win_clone.outer_size() {
                        startup_log::log_step(&format!("RunEvent::Ready: Window size after set: width={}, height={}", size.width, size.height));
                    }

                    // Schedule a delayed retry using tokio
                    let handle = app_handle.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        if let Some(win) = handle.get_webview_window("main") {
                            startup_log::log_step("RunEvent::Ready: Delayed retry - forcing window size and position");
                            match win.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                                width: 1280,
                                height: 720,
                            })) {
                                Ok(_) => startup_log::log_step("RunEvent::Ready: Delayed set_size(1280x720) succeeded (Physical)"),
                                Err(e) => startup_log::log_error(&format!("RunEvent::Ready: Delayed set_size(1280x720) failed (Physical): {}", e)),
                            }
                            match win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                                x: 60,
                                y: 60,
                            })) {
                                Ok(_) => startup_log::log_step("RunEvent::Ready: Delayed set_position(60, 60) succeeded (Physical)"),
                                Err(e) => startup_log::log_error(&format!("RunEvent::Ready: Delayed set_position(60, 60) failed (Physical): {}", e)),
                            }
                            let _ = win.unminimize();

                            // Log the actual window state after delayed set
                            if let Ok(pos) = win.outer_position() {
                                startup_log::log_step(&format!("RunEvent::Ready: Delayed Window position after set: x={}, y={}", pos.x, pos.y));
                            }
                            if let Ok(size) = win.outer_size() {
                                startup_log::log_step(&format!("RunEvent::Ready: Delayed Window size after set: width={}, height={}", size.width, size.height));
                            }
                        }
                    });
                }
                */
            }
            #[cfg(target_os = "macos")]
            RunEvent::Reopen { .. } => {
                // User clicked dock icon - show and focus main window
                startup_log::log_step("Dock icon clicked (Reopen event)");
                
                // Log window state before activation
                if let Some(win) = app.get_webview_window("main") {
                    let visible = win.is_visible().unwrap_or(false);
                    let minimized = win.is_minimized().unwrap_or(false);
                    let focused = win.is_focused().unwrap_or(false);
                    startup_log::log_step(&format!(
                        "Window state before Reopen: visible={}, minimized={}, focused={}",
                        visible, minimized, focused
                    ));
                }
                
                crate::app_window::ensure_main_window_visible(app);
                
                // Log window state after activation
                if let Some(win) = app.get_webview_window("main") {
                    let visible = win.is_visible().unwrap_or(false);
                    let minimized = win.is_minimized().unwrap_or(false);
                    let focused = win.is_focused().unwrap_or(false);
                    startup_log::log_step(&format!(
                        "Window state after Reopen: visible={}, minimized={}, focused={}",
                        visible, minimized, focused
                    ));
                }
            }
            #[cfg(target_os = "macos")]
            RunEvent::WindowEvent { label, event, .. } if label == "main" => {
                use tauri::WindowEvent;
                match event {
                    WindowEvent::Focused(true) => {
                        startup_log::log_step("Window gained focus");
                    }
                    WindowEvent::Focused(false) => {
                        startup_log::log_step("Window lost focus");
                    }
                    WindowEvent::CloseRequested { .. } => {
                        startup_log::log_warn("Window close requested");
                    }
                    WindowEvent::Destroyed => {
                        startup_log::log_warn("Window destroyed");
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        startup_log::log_step(&format!("Window scale factor → {scale_factor}"));
                    }
                    WindowEvent::Moved(_) => {
                        startup_log::log_step("WindowEvent::Moved - window position changed");
                        if let Some(win) = app.get_webview_window("main") {
                            crate::app_window::log_window_detailed_state(&win, "AFTER WindowEvent::Moved");
                        }
                    }
                    // Configure window immediately after creation
                    WindowEvent::Resized(_) => {
                        // Do not force Physical resize here — it shrinks the window on Retina and fights setup().
                        if let Some(win) = app.get_webview_window("main") {
                            startup_log::log_step("WindowEvent::Resized — ensure visible");
                            crate::app_window::ensure_main_window_visible(app);
                        }
                    }
                    _ => {}
                }
            }
            RunEvent::ExitRequested { .. } => {
                self.set_phase(LifecyclePhase::ShuttingDown);
            }
            _ => {}
        }
    }
}

fn detect_launch_mode() -> LaunchMode {
    match std::env::current_exe() {
        Ok(path) => {
            let s = path.to_string_lossy();
            if s.contains(".app/Contents/MacOS") {
                LaunchMode::BundledApp
            } else {
                LaunchMode::DevBinary
            }
        }
        Err(_) => LaunchMode::DevBinary,
    }
}

async fn run_scheduled_tick(app: &AppHandle, manager: &Arc<AppLifecycleManager>, tick: u64) {
    {
        let mut g = manager.lock_inner();
        g.tick_count = tick;
    }

    run_health_checks(app, manager).await;

    if manager.current_phase() != LifecyclePhase::ShuttingDown {
        auto_remediate(app, manager).await;
    }
    manager.recompute_phase();

    {
        let mut g = manager.lock_inner();
        if g.launch_mode == LaunchMode::DevBinary {
            g.last_error = Some(
                "Dev binary: Dock may show a generic icon. Use `pnpm tauri build` and open Exodus.app."
                    .into(),
            );
        }
    }

    let status = manager.status();
    let degraded = status.phase == LifecyclePhase::Degraded;
    lifecycle_log::log_tick(tick, &format!("{:?}", status.phase), degraded);
    let _ = app.emit("exodus-lifecycle-tick", &status);
}

/// Deterministic phase transition from component health (invariant I5).
fn recompute_phase_pure(
    current: LifecyclePhase,
    components: impl Iterator<Item = ComponentHealth>,
) -> LifecyclePhase {
    if current == LifecyclePhase::ShuttingDown {
        return current;
    }
    let max_rank = components.map(health_rank).max().unwrap_or(0);
    if max_rank >= health_rank(ComponentHealth::Warn) {
        LifecyclePhase::Degraded
    } else if matches!(current, LifecyclePhase::Degraded | LifecyclePhase::Ready) {
        LifecyclePhase::Running
    } else {
        current
    }
}

/// Map health severity for preset matching (higher = worse).
fn health_rank(health: ComponentHealth) -> u8 {
    match health {
        ComponentHealth::Ok => 0,
        ComponentHealth::Unknown => 1,
        ComponentHealth::Warn => 2,
        ComponentHealth::Error => 3,
    }
}

/// Pure preset selection from component health map (deterministic, no I/O).
fn select_presets_pure(health: &HashMap<String, ComponentHealth>) -> Vec<&'static RemediationPresetDef> {
    let main_h = health
        .get("main_window")
        .copied()
        .unwrap_or(ComponentHealth::Unknown);
    let frontend_h = health
        .get("frontend")
        .copied()
        .unwrap_or(ComponentHealth::Unknown);

    let mut matched: Vec<&RemediationPresetDef> = REMEDIATION_PRESETS
        .iter()
        .filter(|preset| {
            if preset.id == "reload_on_hidden_window" {
                return main_h == ComponentHealth::Warn && frontend_h == ComponentHealth::Ok;
            }
            if preset.id == "full_ui_recovery" {
                return health_rank(main_h) >= health_rank(ComponentHealth::Warn)
                    && health_rank(frontend_h) >= health_rank(ComponentHealth::Warn);
            }
            if preset.component == "_ui_stack" {
                return false;
            }
            let component_h = health
                .get(preset.component)
                .copied()
                .unwrap_or(ComponentHealth::Unknown);
            health_rank(component_h) >= health_rank(preset.triggers_on)
        })
        .collect();

    matched.sort_by_key(|p| p.priority);

    let mut seen_keys = std::collections::HashSet::new();
    matched.retain(|p| seen_keys.insert(p.cooldown_key));

    if matched.iter().any(|p| p.id == "full_ui_recovery") {
        matched.retain(|p| {
            p.id == "full_ui_recovery"
                || !matches!(
                    p.id,
                    "show_main_window" | "reload_frontend" | "reload_on_hidden_window"
                )
        });
    }

    matched
}

/// Select preset playbooks that match current component health.
fn select_presets(manager: &Arc<AppLifecycleManager>) -> Vec<&'static RemediationPresetDef> {
    let health: HashMap<String, ComponentHealth> = manager
        .lock_inner()
        .components
        .iter()
        .map(|(k, v)| (k.clone(), v.health))
        .collect();
    select_presets_pure(&health)
}

fn health_label(health: ComponentHealth) -> &'static str {
    match health {
        ComponentHealth::Ok => "ok",
        ComponentHealth::Warn => "warn",
        ComponentHealth::Error => "error",
        ComponentHealth::Unknown => "unknown",
    }
}

/// List all built-in presets for settings / diagnostics UI.
pub fn list_remediation_presets() -> Vec<RemediationPresetDto> {
    REMEDIATION_PRESETS
        .iter()
        .map(|p| RemediationPresetDto {
            id: p.id.to_string(),
            name: p.name.to_string(),
            description: p.description.to_string(),
            component: p.component.to_string(),
            triggers_on: health_label(p.triggers_on).to_string(),
            priority: p.priority,
            cooldown_secs: REMEDIATION_COOLDOWN_SECS,
        })
        .collect()
}

/// Execute a single preset remediation action.
async fn execute_preset(
    app: &AppHandle,
    manager: &Arc<AppLifecycleManager>,
    preset: &RemediationPresetDef,
) {
    if !manager.can_remediate(preset.cooldown_key) {
        lifecycle_log::log(
            lifecycle_log::LifecycleLogLevel::Debug,
            lifecycle_log::LifecycleLogCategory::Preset,
            &format!("preset={} skipped (cooldown)", preset.id),
            None,
        );
        return;
    }

    match preset.id {
        "show_main_window" => remediate_main_window(app, manager, preset.id).await,
        "reload_frontend" | "reload_on_hidden_window" => {
            remediate_reload_frontend(app, manager, preset.id).await;
        }
        "restart_allama" => remediate_restart_allama(app, manager, preset.id).await,
        "restart_sidecar" => remediate_restart_sidecar(app, manager, preset.id).await,
        "full_ui_recovery" => {}
        _ => {
            lifecycle_log::log(
                lifecycle_log::LifecycleLogLevel::Warn,
                lifecycle_log::LifecycleLogCategory::Preset,
                &format!("unknown preset id={}", preset.id),
                None,
            );
        }
    }
}

/// Run all health probes (no side effects).
async fn run_health_checks(app: &AppHandle, manager: &Arc<AppLifecycleManager>) {
    check_main_window(app, manager);
    check_config(app, manager);
    check_frontend_dev_server_async(app, manager).await;
    check_allama_async(app, manager).await;
    check_sidecar_async(app, manager).await;
}

/// Apply automatic fixes using preset playbooks selected from health snapshots.
async fn auto_remediate(app: &AppHandle, manager: &Arc<AppLifecycleManager>) {
    if manager.current_phase() == LifecyclePhase::ShuttingDown {
        return;
    }
    if !manager.lock_inner().auto_fix_enabled {
        return;
    }

    let presets = select_presets(manager);
    if presets.is_empty() {
        return;
    }

    let ids: Vec<&str> = presets.iter().map(|p| p.id).collect();
    lifecycle_log::log_preset_plan(&ids, "health tick auto-fix");

    for preset in presets {
        if preset.id == "full_ui_recovery" {
            if manager.can_remediate(preset.cooldown_key) {
                remediate_main_window(app, manager, preset.id).await;
                remediate_reload_frontend(app, manager, preset.id).await;
            }
            continue;
        }
        execute_preset(app, manager, preset).await;
    }

    let status = manager.status();
    if let Some(last) = status.recent_remediations.last() {
        let _ = app.emit("exodus-lifecycle-remediation", last);
    }
}

async fn remediate_main_window(
    app: &AppHandle,
    manager: &Arc<AppLifecycleManager>,
    preset_id: &str,
) {
    const KEY: &str = "show_main_window";
    let cooldown_key = if preset_id == "full_ui_recovery" {
        "full_ui_recovery"
    } else {
        KEY
    };
    if !manager.can_remediate(cooldown_key) {
        return;
    }
    #[cfg(target_os = "macos")]
    macos_activate_process();
    app_window::ensure_main_window_ready(app);

    let ok = app.get_webview_window("main").map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false);
    let apply_cooldown = preset_id != "full_ui_recovery";
    manager.record_remediation(
        preset_id,
        cooldown_key,
        "show_main_window",
        ok,
        if ok {
            "window shown and focused"
        } else {
            "main window still missing or hidden"
        },
        apply_cooldown,
    );
    app_window::log_main_window_state(app, "auto-fix show_main_window");
}

async fn remediate_reload_frontend(
    app: &AppHandle,
    manager: &Arc<AppLifecycleManager>,
    preset_id: &str,
) {
    const KEY: &str = "reload_frontend";
    let cooldown_key = if preset_id == "full_ui_recovery" {
        "full_ui_recovery"
    } else {
        KEY
    };
    if !manager.can_remediate(cooldown_key) {
        return;
    }

    let target = app
        .config()
        .build
        .dev_url
        .as_ref()
        .map(|u| u.to_string())
        .unwrap_or_else(|| "http://localhost:1421/".to_string());

    let mut success = false;
    let mut detail = String::new();

    if manager.component_health("frontend") == ComponentHealth::Error {
        detail = format!(
            "Vite not reachable at {target}; start with: pnpm dev (or pnpm tauri dev)"
        );
        let _ = app.emit("exodus-lifecycle-frontend-down", target.clone());
    }

    if let Some(win) = app.get_webview_window("main") {
        match Url::parse(&target) {
            Ok(url) => {
                match win.navigate(url) {
                    Ok(()) => {
                        success = true;
                        if detail.is_empty() {
                            detail = format!("navigated main window to {target}");
                        } else {
                            detail.push_str("; navigated when server returns");
                        }
                    }
                    Err(e) => {
                        detail = format!("navigate failed: {e}");
                    }
                }
            }
            Err(e) => detail = format!("invalid dev url: {e}"),
        }
    } else {
        detail = "main window not found for reload".to_string();
    }

    manager.record_remediation(preset_id, cooldown_key, "reload_frontend", success, detail, true);
}

async fn remediate_restart_allama(
    app: &AppHandle,
    manager: &Arc<AppLifecycleManager>,
    preset_id: &str,
) {
    const KEY: &str = "restart_allama";
    if !manager.can_remediate(KEY) {
        return;
    }

    let Some(allama) = app.try_state::<Arc<AllamaManager>>() else {
        manager.record_remediation(
            preset_id,
            KEY,
            "restart_allama",
            false,
            "AllamaManager not in app state",
            true,
        );
        return;
    };

    let enabled = app
        .try_state::<ConfigState>()
        .and_then(|c| c.lock().ok().map(|cfg| cfg.spawn_allama))
        .unwrap_or(false);

    if !enabled {
        manager.record_remediation(
            preset_id,
            KEY,
            "restart_allama",
            false,
            "spawn_allama disabled in settings",
            true,
        );
        return;
    }

    match allama.restart().await {
        Ok(()) => {
            let online = allama.http_online().await;
            manager.set_component(
                "allama",
                if online {
                    ComponentHealth::Ok
                } else {
                    ComponentHealth::Warn
                },
                format!("restarted; online={online}"),
            );
            manager.record_remediation(
                preset_id,
                KEY,
                "restart_allama",
                online,
                format!("Allama restarted, http_online={online}"),
                true,
            );
            let _ = app.emit("allama-started", allama.status_dto().await);
        }
        Err(e) => {
            manager.record_remediation(preset_id, KEY, "restart_allama", false, e, true);
        }
    }
}

async fn remediate_restart_sidecar(
    app: &AppHandle,
    manager: &Arc<AppLifecycleManager>,
    preset_id: &str,
) {
    const KEY: &str = "restart_sidecar";
    if !manager.can_remediate(KEY) {
        return;
    }

    let Some(sidecar) = app.try_state::<SidecarManager>() else {
        manager.record_remediation(
            preset_id,
            KEY,
            "restart_sidecar",
            false,
            "SidecarManager not in app state",
            true,
        );
        return;
    };

    let (port, enabled) = match app.try_state::<ConfigState>() {
        Some(cfg) => match cfg.lock() {
            Ok(c) => (c.ai_port, c.spawn_sidecar),
            Err(e) => {
                manager.record_remediation(preset_id, KEY, "restart_sidecar", false, e.to_string(), true);
                return;
            }
        },
        None => {
            manager.record_remediation(preset_id, KEY, "restart_sidecar", false, "config not loaded", true);
            return;
        }
    };

    if !enabled {
        manager.record_remediation(
            preset_id,
            KEY,
            "restart_sidecar",
            false,
            "spawn_sidecar disabled",
            true,
        );
        return;
    }

    match sidecar.restart(app, port, true) {
        Ok(()) => {
            let online = probe_models_endpoint(port).await;
            manager.set_component(
                "sidecar",
                if online {
                    ComponentHealth::Ok
                } else {
                    ComponentHealth::Warn
                },
                format!("restarted; endpoint_online={online}"),
            );
            manager.record_remediation(
                preset_id,
                KEY,
                "restart_sidecar",
                online,
                format!("sidecar restarted, online={online}"),
                true,
            );
        }
        Err(e) => manager.record_remediation(preset_id, KEY, "restart_sidecar", false, e, true),
    }
}

fn check_main_window(app: &AppHandle, manager: &Arc<AppLifecycleManager>) {
    match app.get_webview_window("main") {
        Some(win) => {
            let url = win.url().map(|u| u.to_string()).unwrap_or_else(|_| String::new());
            let visible = win.is_visible().unwrap_or(false);
            let focused = win.is_focused().unwrap_or(false);
            let minimized = win.is_minimized().unwrap_or(false);
            let msg = format!("url={url} visible={visible} focused={focused} minimized={minimized}");
            let health = if !visible || minimized {
                ComponentHealth::Warn
            } else {
                ComponentHealth::Ok
            };
            manager.set_component("main_window", health, msg);
        }
        None => manager.set_component(
            "main_window",
            ComponentHealth::Error,
            "main webview window not found",
        ),
    }
}

async fn check_frontend_dev_server_async(app: &AppHandle, manager: &Arc<AppLifecycleManager>) {
    let dev_url = match &app.config().build.dev_url {
        Some(url) => url.to_string(),
        None => {
            manager.set_component(
                "frontend",
                ComponentHealth::Ok,
                "production frontendDist mode",
            );
            return;
        }
    };
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            manager.set_component("frontend", ComponentHealth::Error, e.to_string());
            return;
        }
    };
    match client.get(&dev_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            manager.set_component("frontend", ComponentHealth::Ok, dev_url);
        }
        Ok(resp) => manager.set_component(
            "frontend",
            ComponentHealth::Warn,
            format!("{dev_url} HTTP {}", resp.status()),
        ),
        Err(e) => manager.set_component(
            "frontend",
            ComponentHealth::Error,
            format!("cannot reach {dev_url}: {e}"),
        ),
    }
}

async fn check_allama_async(app: &AppHandle, manager: &Arc<AppLifecycleManager>) {
    let Some(allama) = app.try_state::<Arc<AllamaManager>>() else {
        manager.set_component("allama", ComponentHealth::Unknown, "not initialized");
        return;
    };
    let enabled = app
        .try_state::<ConfigState>()
        .and_then(|c| c.lock().ok().map(|cfg| cfg.spawn_allama))
        .unwrap_or(false);
    if !enabled {
        manager.set_component("allama", ComponentHealth::Ok, "auto-start disabled");
        return;
    }
    let online = allama.http_online().await;
    let dto = allama.status_dto().await;
    if online {
        manager.set_component(
            "allama",
            ComponentHealth::Ok,
            format!("online port={} mode={}", dto.port, dto.state),
        );
    } else {
        manager.set_component(
            "allama",
            ComponentHealth::Error,
            format!("offline port={} detail={}", dto.port, dto.detail),
        );
    }
}

async fn check_sidecar_async(app: &AppHandle, manager: &Arc<AppLifecycleManager>) {
    let Some(sidecar) = app.try_state::<SidecarManager>() else {
        manager.set_component("sidecar", ComponentHealth::Unknown, "not initialized");
        return;
    };
    let (port, enabled) = match app.try_state::<ConfigState>() {
        Some(cfg) => match cfg.lock() {
            Ok(c) => (c.ai_port, c.spawn_sidecar),
            Err(e) => {
                manager.set_component("sidecar", ComponentHealth::Error, e.to_string());
                return;
            }
        },
        None => {
            manager.set_component("sidecar", ComponentHealth::Unknown, "config not loaded");
            return;
        }
    };
    if !enabled {
        manager.set_component("sidecar", ComponentHealth::Ok, "auto-start disabled");
        return;
    }
    let dto = sidecar.status_dto(port).await;
    if dto.endpoint_online {
        manager.set_component(
            "sidecar",
            ComponentHealth::Ok,
            format!("{} — {}", dto.state, dto.detail),
        );
    } else {
        manager.set_component(
            "sidecar",
            ComponentHealth::Error,
            format!("{} — {}", dto.state, dto.detail),
        );
    }
}

fn check_config(app: &AppHandle, manager: &Arc<AppLifecycleManager>) {
    match app.try_state::<ConfigState>() {
        Some(cfg) => match cfg.lock() {
            Ok(c) => manager.set_component(
                "config",
                ComponentHealth::Ok,
                format!(
                    "allama={} sidecar={} port={}",
                    c.spawn_allama, c.spawn_sidecar, c.ai_port
                ),
            ),
            Err(e) => manager.set_component("config", ComponentHealth::Error, e.to_string()),
        },
        None => manager.set_component("config", ComponentHealth::Unknown, "not loaded yet"),
    }
}

#[cfg(target_os = "macos")]
fn macos_activate_process() {
    let pid = std::process::id();
    let script = format!(
        "tell application \"System Events\" to set frontmost of first process whose unix id is {pid} to true"
    );
    let _ = std::process::Command::new("osascript").args(["-e", &script]).output();
}

#[cfg(not(target_os = "macos"))]
fn macos_activate_process() {}

// --- Tauri commands ---

/// Return full lifecycle status (phase, components, remediations).
#[tauri::command]
pub fn lifecycle_get_status(
    manager: tauri::State<'_, Arc<AppLifecycleManager>>,
) -> Result<LifecycleStatusDto, String> {
    Ok(manager.status())
}

/// Force main window visible and refresh dock policy.
#[tauri::command]
pub async fn lifecycle_show_main_window(
    app: AppHandle,
    manager: tauri::State<'_, Arc<AppLifecycleManager>>,
) -> Result<LifecycleStatusDto, String> {
    startup_log::log_step("lifecycle_show_main_window invoked (manual)");
    remediate_main_window(&app, &manager, "show_main_window").await;
    Ok(manager.status())
}

/// Run health checks + auto-remediation immediately.
#[tauri::command]
pub async fn lifecycle_run_health_tick(
    app: AppHandle,
    manager: tauri::State<'_, Arc<AppLifecycleManager>>,
) -> Result<LifecycleStatusDto, String> {
    run_scheduled_tick(&app, &manager, 0).await;
    Ok(manager.status())
}

/// Enable or disable automatic remediation (scheduler still runs checks).
#[tauri::command]
pub fn lifecycle_set_auto_fix(
    enabled: bool,
    manager: tauri::State<'_, Arc<AppLifecycleManager>>,
) -> Result<bool, String> {
    manager.lock_inner().auto_fix_enabled = enabled;
    startup_log::log_step(&format!("lifecycle auto_fix_enabled={enabled}"));
    Ok(enabled)
}

/// Return recent structured lifecycle log entries for analysis.
#[tauri::command]
pub fn lifecycle_get_logs(limit: Option<usize>) -> Vec<LifecycleLogEntry> {
    let limit = limit.unwrap_or(64);
    let limit = limit.clamp(1, MAX_LOG_EXPORT);
    lifecycle_log::recent_entries(limit)
}

/// List built-in remediation preset playbooks.
#[tauri::command]
pub fn lifecycle_list_presets() -> Vec<RemediationPresetDto> {
    list_remediation_presets()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_phase_serde_snake_case() {
        let j = serde_json::to_string(&LifecyclePhase::Running).expect("serialize");
        assert_eq!(j, "\"running\"");
    }

    #[test]
    fn manager_starts_in_booting_with_auto_fix() {
        let m = AppLifecycleManager::new();
        let s = m.status();
        assert_eq!(s.phase, LifecyclePhase::Booting);
        assert!(s.auto_fix_enabled);
    }

    #[test]
    fn remediation_cooldown_blocks_rapid_retry() {
        let m = AppLifecycleManager::new();
        m.record_remediation("test_preset", "test_action", "test", true, "first", true);
        assert!(!m.can_remediate("test_action"));
    }

    #[test]
    fn select_presets_matches_frontend_error() {
        let m = Arc::new(AppLifecycleManager::new());
        m.set_component("frontend", ComponentHealth::Error, "down");
        let presets = select_presets(&m);
        assert!(presets.iter().any(|p| p.id == "reload_frontend"));
    }

    #[test]
    fn list_presets_includes_show_main_window() {
        let list = list_remediation_presets();
        assert!(list.iter().any(|p| p.id == "show_main_window"));
    }

    #[test]
    fn recompute_phase_marks_degraded_on_error() {
        let m = AppLifecycleManager::new();
        m.set_component("frontend", ComponentHealth::Error, "down");
        m.recompute_phase();
        assert_eq!(m.status().phase, LifecyclePhase::Degraded);
    }

    #[test]
    fn status_started_at_is_stable() {
        let m = AppLifecycleManager::new();
        let s1 = m.status().started_at;
        let s2 = m.status().started_at;
        assert_eq!(s1, s2);
        assert!(!s1.is_empty());
    }

    #[test]
    fn select_presets_full_ui_supersedes_ui_presets() {
        let m = Arc::new(AppLifecycleManager::new());
        m.set_component("main_window", ComponentHealth::Warn, "hidden");
        m.set_component("frontend", ComponentHealth::Warn, "slow");
        let presets = select_presets(&m);
        assert!(presets.iter().any(|p| p.id == "full_ui_recovery"));
        assert!(!presets.iter().any(|p| p.id == "show_main_window"));
        assert!(!presets.iter().any(|p| p.id == "reload_frontend"));
    }

    #[test]
    fn select_presets_reload_on_hidden_requires_frontend_ok() {
        let m = Arc::new(AppLifecycleManager::new());
        m.set_component("main_window", ComponentHealth::Warn, "hidden");
        m.set_component("frontend", ComponentHealth::Error, "down");
        let presets = select_presets(&m);
        assert!(!presets.iter().any(|p| p.id == "reload_on_hidden_window"));
    }

    #[test]
    fn select_presets_dedupes_shared_cooldown_key() {
        let m = Arc::new(AppLifecycleManager::new());
        m.set_component("main_window", ComponentHealth::Warn, "hidden");
        m.set_component("frontend", ComponentHealth::Ok, "up");
        let presets = select_presets(&m);
        let reload_count = presets
            .iter()
            .filter(|p| p.cooldown_key == "reload_frontend")
            .count();
        assert_eq!(reload_count, 1);
    }

    #[test]
    fn list_presets_has_six_entries() {
        assert_eq!(list_remediation_presets().len(), REMEDIATION_PRESETS.len());
        assert_eq!(list_remediation_presets().len(), 6);
    }

    #[test]
    fn preset_triggers_on_uses_snake_case() {
        let show = list_remediation_presets()
            .into_iter()
            .find(|p| p.id == "show_main_window")
            .expect("show_main_window preset");
        assert_eq!(show.triggers_on, "warn");
    }

    #[test]
    fn health_rank_orders_severity() {
        assert!(health_rank(ComponentHealth::Error) > health_rank(ComponentHealth::Warn));
        assert!(health_rank(ComponentHealth::Warn) > health_rank(ComponentHealth::Ok));
    }

    #[test]
    fn full_ui_recovery_only_applies_cooldown_on_last_step() {
        let m = AppLifecycleManager::new();
        m.record_remediation(
            "full_ui_recovery",
            "full_ui_recovery",
            "show_main_window",
            true,
            "step1",
            false,
        );
        assert!(m.can_remediate("full_ui_recovery"));
        m.record_remediation(
            "full_ui_recovery",
            "full_ui_recovery",
            "reload_frontend",
            true,
            "step2",
            true,
        );
        assert!(!m.can_remediate("full_ui_recovery"));
    }

    // --- Aerospace-grade verification matrix ---

    #[test]
    fn preset_table_unique_ids_and_valid_priorities() {
        let mut ids = std::collections::HashSet::new();
        for p in REMEDIATION_PRESETS {
            assert!(ids.insert(p.id), "duplicate preset id: {}", p.id);
            assert!(!p.cooldown_key.is_empty());
            assert!(p.priority > 0 && p.priority <= 100);
            assert!(!p.name.is_empty());
            assert!(!p.description.is_empty());
        }
        assert_eq!(ids.len(), 6);
    }

    #[test]
    fn select_presets_output_sorted_by_priority() {
        let mut health = HashMap::new();
        health.insert("main_window".into(), ComponentHealth::Warn);
        health.insert("frontend".into(), ComponentHealth::Error);
        health.insert("allama".into(), ComponentHealth::Error);
        let presets = select_presets_pure(&health);
        let mut priorities: Vec<u8> = presets.iter().map(|p| p.priority).collect();
        let mut sorted = priorities.clone();
        sorted.sort_unstable();
        assert_eq!(priorities, sorted);
    }

    #[test]
    fn select_presets_never_duplicate_cooldown_exhaustive_ui_grid() {
        let grid = [
            ComponentHealth::Ok,
            ComponentHealth::Warn,
            ComponentHealth::Error,
            ComponentHealth::Unknown,
        ];
        for &main_h in &grid {
            for &front_h in &grid {
                let mut health = HashMap::new();
                health.insert("main_window".into(), main_h);
                health.insert("frontend".into(), front_h);
                let presets = select_presets_pure(&health);
                let mut keys = std::collections::HashSet::new();
                for p in &presets {
                    assert!(keys.insert(p.cooldown_key), "dup key for {:?}", p.id);
                }
            }
        }
    }

    #[test]
    fn recompute_phase_pure_warn_is_degraded() {
        let next = recompute_phase_pure(
            LifecyclePhase::Running,
            [ComponentHealth::Warn].into_iter(),
        );
        assert_eq!(next, LifecyclePhase::Degraded);
    }

    #[test]
    fn recompute_phase_pure_all_ok_recovers_from_degraded() {
        let next = recompute_phase_pure(
            LifecyclePhase::Degraded,
            [ComponentHealth::Ok, ComponentHealth::Unknown].into_iter(),
        );
        assert_eq!(next, LifecyclePhase::Running);
    }

    #[test]
    fn recompute_phase_pure_shutting_down_is_immutable() {
        let next = recompute_phase_pure(
            LifecyclePhase::ShuttingDown,
            [ComponentHealth::Error].into_iter(),
        );
        assert_eq!(next, LifecyclePhase::ShuttingDown);
    }

    #[test]
    fn recompute_phase_warn_keeps_degraded_via_manager() {
        let m = AppLifecycleManager::new();
        m.set_phase(LifecyclePhase::Running);
        m.set_component("config", ComponentHealth::Warn, "slow disk");
        m.recompute_phase();
        assert_eq!(m.status().phase, LifecyclePhase::Degraded);
    }

    #[test]
    fn remediation_history_bounded_invariant_i2() {
        let m = AppLifecycleManager::new();
        for i in 0..(MAX_REMEDIATION_HISTORY + 8) {
            m.record_remediation("p", &format!("k{i}"), "a", true, "d", true);
        }
        assert!(m.status().recent_remediations.len() <= MAX_REMEDIATION_HISTORY);
    }

    #[test]
    fn auto_fix_disabled_blocks_remediation() {
        let m = AppLifecycleManager::new();
        m.lock_inner().auto_fix_enabled = false;
        assert!(!m.can_remediate("show_main_window"));
    }

    #[test]
    fn component_health_serde_roundtrip() {
        for h in [
            ComponentHealth::Ok,
            ComponentHealth::Warn,
            ComponentHealth::Error,
            ComponentHealth::Unknown,
        ] {
            let j = serde_json::to_string(&h).unwrap();
            let back: ComponentHealth = serde_json::from_str(&j).unwrap();
            assert_eq!(h, back);
        }
    }

    #[test]
    fn remediation_record_roundtrip_json() {
        let r = RemediationRecord {
            preset_id: "reload_frontend".into(),
            action: "reload_frontend".into(),
            success: true,
            detail: "ok".into(),
            at: Utc::now().to_rfc3339(),
        };
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("preset_id"));
        assert!(j.contains("reload_frontend"));
    }

    #[test]
    fn select_presets_pure_allama_error_triggers_restart() {
        let mut health = HashMap::new();
        health.insert("allama".into(), ComponentHealth::Error);
        let presets = select_presets_pure(&health);
        assert!(presets.iter().any(|p| p.id == "restart_allama"));
    }

    #[test]
    fn health_rank_strict_order() {
        assert!(health_rank(ComponentHealth::Error) > health_rank(ComponentHealth::Warn));
        assert!(health_rank(ComponentHealth::Warn) > health_rank(ComponentHealth::Unknown));
        assert!(health_rank(ComponentHealth::Unknown) > health_rank(ComponentHealth::Ok));
    }
}
