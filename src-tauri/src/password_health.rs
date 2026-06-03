//! Password Health Monitoring Dashboard for Exodus Browser
//! Provides password strength analysis, duplicate detection, and breach monitoring

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Password health score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasswordHealthScore {
    Excellent,
    Good,
    Fair,
    Weak,
    Critical,
}

/// Password health entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHealthEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub username: String,
    pub strength_score: PasswordHealthScore,
    pub strength_percentage: u8,
    pub is_compromised: bool,
    pub is_duplicate: bool,
    pub age_days: u32,
    pub last_changed: String,
}

/// Password health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHealthSummary {
    pub total_passwords: usize,
    pub weak_passwords: usize,
    pub duplicate_passwords: usize,
    pub compromised_passwords: usize,
    pub old_passwords: usize,
    pub overall_health_score: PasswordHealthScore,
    pub health_percentage: u8,
}

/// Password Health Manager
pub struct PasswordHealthManager {
    passwords: Arc<Mutex<Vec<PasswordHealthEntry>>>,
}

impl PasswordHealthManager {
    pub fn new() -> Self {
        Self {
            passwords: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Add password entry
    pub fn add_password(&self, entry: PasswordHealthEntry, app: AppHandle) {
        if let Ok(mut passwords) = self.passwords.lock() {
            passwords.push(entry.clone());
            let _ = app.emit("exodus-password-health-updated", self.get_summary());
        }
    }

    /// Remove password entry
    pub fn remove_password(&self, id: String, app: AppHandle) {
        if let Ok(mut passwords) = self.passwords.lock() {
            passwords.retain(|p| p.id != id);
            let _ = app.emit("exodus-password-health-updated", self.get_summary());
        }
    }

    /// Update password strength
    pub fn update_strength(&self, id: String, score: PasswordHealthScore, percentage: u8, app: AppHandle) {
        if let Ok(mut passwords) = self.passwords.lock() {
            if let Some(entry) = passwords.iter_mut().find(|p| p.id == id) {
                entry.strength_score = score.clone();
                entry.strength_percentage = percentage;
            }
            let _ = app.emit("exodus-password-health-updated", self.get_summary());
        }
    }

    /// Mark password as compromised
    pub fn mark_compromised(&self, id: String, compromised: bool, app: AppHandle) {
        if let Ok(mut passwords) = self.passwords.lock() {
            if let Some(entry) = passwords.iter_mut().find(|p| p.id == id) {
                entry.is_compromised = compromised;
            }
            let _ = app.emit("exodus-password-health-updated", self.get_summary());
        }
    }

    /// Mark password as duplicate
    pub fn mark_duplicate(&self, id: String, duplicate: bool, app: AppHandle) {
        if let Ok(mut passwords) = self.passwords.lock() {
            if let Some(entry) = passwords.iter_mut().find(|p| p.id == id) {
                entry.is_duplicate = duplicate;
            }
            let _ = app.emit("exodus-password-health-updated", self.get_summary());
        }
    }

    /// Get all password entries
    pub fn get_passwords(&self) -> Vec<PasswordHealthEntry> {
        self.passwords.lock()
            .map(|passwords| passwords.clone())
            .unwrap_or_default()
    }

    /// Get weak passwords
    pub fn get_weak_passwords(&self) -> Vec<PasswordHealthEntry> {
        self.passwords.lock()
            .map(|passwords| {
                passwords.iter()
                    .filter(|p| matches!(p.strength_score, PasswordHealthScore::Weak | PasswordHealthScore::Critical))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get duplicate passwords
    pub fn get_duplicate_passwords(&self) -> Vec<PasswordHealthEntry> {
        self.passwords.lock()
            .map(|passwords| {
                passwords.iter()
                    .filter(|p| p.is_duplicate)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get compromised passwords
    pub fn get_compromised_passwords(&self) -> Vec<PasswordHealthEntry> {
        self.passwords.lock()
            .map(|passwords| {
                passwords.iter()
                    .filter(|p| p.is_compromised)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get old passwords (older than 90 days)
    pub fn get_old_passwords(&self) -> Vec<PasswordHealthEntry> {
        self.passwords.lock()
            .map(|passwords| {
                passwords.iter()
                    .filter(|p| p.age_days > 90)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get health summary
    pub fn get_summary(&self) -> PasswordHealthSummary {
        let passwords = self.passwords.lock();
        let total = passwords.as_ref().map(|p| p.len()).unwrap_or(0);
        let weak = passwords.as_ref().map(|p| p.iter().filter(|p| matches!(p.strength_score, PasswordHealthScore::Weak | PasswordHealthScore::Critical)).count()).unwrap_or(0);
        let duplicate = passwords.as_ref().map(|p| p.iter().filter(|p| p.is_duplicate).count()).unwrap_or(0);
        let compromised = passwords.as_ref().map(|p| p.iter().filter(|p| p.is_compromised).count()).unwrap_or(0);
        let old = passwords.as_ref().map(|p| p.iter().filter(|p| p.age_days > 90).count()).unwrap_or(0);
        
        let overall_score = if total == 0 {
            PasswordHealthScore::Excellent
        } else {
            let issues = weak + duplicate + compromised + old;
            let issue_ratio = issues as f64 / total as f64;
            if issue_ratio < 0.1 {
                PasswordHealthScore::Excellent
            } else if issue_ratio < 0.25 {
                PasswordHealthScore::Good
            } else if issue_ratio < 0.5 {
                PasswordHealthScore::Fair
            } else if issue_ratio < 0.75 {
                PasswordHealthScore::Weak
            } else {
                PasswordHealthScore::Critical
            }
        };

        let health_percentage = if total == 0 {
            100
        } else {
            let issues = weak + duplicate + compromised + old;
            let healthy = total - issues;
            ((healthy as f64 / total as f64) * 100.0) as u8
        };

        PasswordHealthSummary {
            total_passwords: total,
            weak_passwords: weak,
            duplicate_passwords: duplicate,
            compromised_passwords: compromised,
            old_passwords: old,
            overall_health_score: overall_score,
            health_percentage,
        }
    }

    /// Run health check
    pub fn run_health_check(&self, app: AppHandle) -> PasswordHealthSummary {
        let summary = self.get_summary();
        let _ = app.emit("exodus-password-health-check-completed", summary.clone());
        summary
    }
}

impl Default for PasswordHealthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to add password entry
#[tauri::command]
pub fn add_password_health_entry(
    title: String,
    url: String,
    username: String,
    strength_score: String,
    strength_percentage: u8,
    app: AppHandle,
    manager: State<'_, Arc<PasswordHealthManager>>,
) -> String {
    let score = match strength_score.as_str() {
        "excellent" => PasswordHealthScore::Excellent,
        "good" => PasswordHealthScore::Good,
        "fair" => PasswordHealthScore::Fair,
        "weak" => PasswordHealthScore::Weak,
        "critical" => PasswordHealthScore::Critical,
        _ => PasswordHealthScore::Fair,
    };
    
    let entry = PasswordHealthEntry {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        url,
        username,
        strength_score: score,
        strength_percentage,
        is_compromised: false,
        is_duplicate: false,
        age_days: 0,
        last_changed: chrono::Utc::now().to_rfc3339(),
    };
    
    manager.add_password(entry, app);
    uuid::Uuid::new_v4().to_string()
}

/// Tauri command to remove password entry
#[tauri::command]
pub fn remove_password_health_entry(
    id: String,
    app: AppHandle,
    manager: State<'_, Arc<PasswordHealthManager>>,
) {
    manager.remove_password(id, app);
}

/// Tauri command to update password strength
#[tauri::command]
pub fn update_password_strength(
    id: String,
    strength_score: String,
    strength_percentage: u8,
    app: AppHandle,
    manager: State<'_, Arc<PasswordHealthManager>>,
) {
    let score = match strength_score.as_str() {
        "excellent" => PasswordHealthScore::Excellent,
        "good" => PasswordHealthScore::Good,
        "fair" => PasswordHealthScore::Fair,
        "weak" => PasswordHealthScore::Weak,
        "critical" => PasswordHealthScore::Critical,
        _ => PasswordHealthScore::Fair,
    };
    manager.update_strength(id, score, strength_percentage, app);
}

/// Tauri command to mark password as compromised
#[tauri::command]
pub fn mark_password_compromised(
    id: String,
    compromised: bool,
    app: AppHandle,
    manager: State<'_, Arc<PasswordHealthManager>>,
) {
    manager.mark_compromised(id, compromised, app);
}

/// Tauri command to mark password as duplicate
#[tauri::command]
pub fn mark_password_duplicate(
    id: String,
    duplicate: bool,
    app: AppHandle,
    manager: State<'_, Arc<PasswordHealthManager>>,
) {
    manager.mark_duplicate(id, duplicate, app);
}

/// Tauri command to get all password entries
#[tauri::command]
pub fn get_password_health_entries(
    manager: State<'_, Arc<PasswordHealthManager>>,
) -> Vec<PasswordHealthEntry> {
    manager.get_passwords()
}

/// Tauri command to get weak passwords
#[tauri::command]
pub fn get_password_health_weak_passwords(
    manager: State<'_, Arc<PasswordHealthManager>>,
) -> Vec<PasswordHealthEntry> {
    manager.get_weak_passwords()
}

/// Tauri command to get duplicate passwords
#[tauri::command]
pub fn get_password_health_duplicate_passwords(
    manager: State<'_, Arc<PasswordHealthManager>>,
) -> Vec<PasswordHealthEntry> {
    manager.get_duplicate_passwords()
}

/// Tauri command to get compromised passwords
#[tauri::command]
pub fn get_password_health_compromised_passwords(
    manager: State<'_, Arc<PasswordHealthManager>>,
) -> Vec<PasswordHealthEntry> {
    manager.get_compromised_passwords()
}

/// Tauri command to get old passwords
#[tauri::command]
pub fn get_password_health_old_passwords(
    manager: State<'_, Arc<PasswordHealthManager>>,
) -> Vec<PasswordHealthEntry> {
    manager.get_old_passwords()
}

/// Tauri command to get health summary
#[tauri::command]
pub fn get_password_health_summary(
    manager: State<'_, Arc<PasswordHealthManager>>,
) -> PasswordHealthSummary {
    manager.get_summary()
}

/// Tauri command to run health check
#[tauri::command]
pub fn run_password_health_check(
    app: AppHandle,
    manager: State<'_, Arc<PasswordHealthManager>>,
) -> PasswordHealthSummary {
    manager.run_health_check(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_health_manager_creation() {
        let manager = PasswordHealthManager::new();
        let summary = manager.get_summary();
        assert_eq!(summary.total_passwords, 0);
    }

    #[test]
    fn test_summary() {
        let manager = PasswordHealthManager::new();
        
        let summary = manager.get_summary();
        assert_eq!(summary.total_passwords, 0);
        assert_eq!(summary.health_percentage, 100);
    }
}
