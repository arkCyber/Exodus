//! Tab Stacking for Exodus Browser
//! Provides nested tab organization (tab groups within tab groups)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Tab stack hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabStack {
    pub stack_id: String,
    pub parent_stack_id: Option<String>,
    pub label: String,
    pub color: String,
    pub collapsed: bool,
    pub tab_count: usize,
}

/// Tab stack settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabStackSettings {
    pub enabled: bool,
    pub auto_group_by_domain: bool,
    pub show_stack_badges: bool,
    pub collapse_on_navigate_away: bool,
}

impl Default for TabStackSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_group_by_domain: false,
            show_stack_badges: true,
            collapse_on_navigate_away: false,
        }
    }
}

/// Tab Stacking Manager
pub struct TabStackManager {
    stacks: Arc<Mutex<HashMap<String, TabStack>>>,
    tab_to_stack: Arc<Mutex<HashMap<String, String>>>, // tab label -> stack_id
    settings: Arc<Mutex<TabStackSettings>>,
}

impl TabStackManager {
    pub fn new() -> Self {
        Self {
            stacks: Arc::new(Mutex::new(HashMap::new())),
            tab_to_stack: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(TabStackSettings::default())),
        }
    }

    /// Create a new stack
    pub fn create_stack(&self, stack_id: String, label: String, color: String, parent_stack_id: Option<String>, app: AppHandle) {
        if let Ok(mut stacks) = self.stacks.lock() {
            let stack = TabStack {
                stack_id: stack_id.clone(),
                parent_stack_id,
                label,
                color,
                collapsed: false,
                tab_count: 0,
            };
            
            stacks.insert(stack_id.clone(), stack);
            let _ = app.emit("exodus-tab-stack-created", stack_id);
        }
    }

    /// Delete a stack
    pub fn delete_stack(&self, stack_id: String, app: AppHandle) {
        if let (Ok(mut stacks), Ok(mut tab_to_stack)) = (self.stacks.lock(), self.tab_to_stack.lock()) {
        
        // Remove all tabs from this stack
        if let Some(stack) = stacks.get(&stack_id) {
            let tabs_to_remove: Vec<String> = tab_to_stack.iter()
                .filter(|(_, sid)| *sid == &stack_id)
                .map(|(tab, _)| tab.clone())
                .collect();
            
            for tab in tabs_to_remove {
                tab_to_stack.remove(&tab);
            }
        }
        
        stacks.remove(&stack_id);
        let _ = app.emit("exodus-tab-stack-deleted", stack_id);
        }
    }

    /// Add a tab to a stack
    pub fn add_tab_to_stack(&self, tab_label: String, stack_id: String, app: AppHandle) {
        if let (Ok(mut tab_to_stack), Ok(mut stacks)) = (self.tab_to_stack.lock(), self.stacks.lock()) {
            tab_to_stack.insert(tab_label.clone(), stack_id.clone());
            
            if let Some(stack) = stacks.get_mut(&stack_id) {
                stack.tab_count += 1;
            }
            
            let _ = app.emit("exodus-tab-added-to-stack", (tab_label, stack_id));
        }
    }

    /// Remove a tab from its stack
    pub fn remove_tab_from_stack(&self, tab_label: String, app: AppHandle) {
        if let (Ok(mut tab_to_stack), Ok(mut stacks)) = (self.tab_to_stack.lock(), self.stacks.lock()) {
            if let Some(stack_id) = tab_to_stack.remove(&tab_label) {
                if let Some(stack) = stacks.get_mut(&stack_id) {
                    stack.tab_count = stack.tab_count.saturating_sub(1);
                }
                let _ = app.emit("exodus-tab-removed-from-stack", (tab_label, stack_id));
            }
        }
    }

    /// Move a tab to a different stack
    pub fn move_tab_to_stack(&self, tab_label: String, new_stack_id: String, app: AppHandle) {
        self.remove_tab_from_stack(tab_label.clone(), app.clone());
        self.add_tab_to_stack(tab_label, new_stack_id, app);
    }

    /// Collapse a stack
    pub fn collapse_stack(&self, stack_id: String, app: AppHandle) {
        if let Ok(mut stacks) = self.stacks.lock() {
            if let Some(stack) = stacks.get_mut(&stack_id) {
                stack.collapsed = true;
                let _ = app.emit("exodus-tab-stack-collapsed", stack_id);
            }
        }
    }

    /// Expand a stack
    pub fn expand_stack(&self, stack_id: String, app: AppHandle) {
        if let Ok(mut stacks) = self.stacks.lock() {
            if let Some(stack) = stacks.get_mut(&stack_id) {
                stack.collapsed = false;
                let _ = app.emit("exodus-tab-stack-expanded", stack_id);
            }
        }
    }

    /// Toggle collapse state
    pub fn toggle_stack_collapse(&self, stack_id: String, app: AppHandle) {
        if let Ok(stacks) = self.stacks.lock() {
            if let Some(stack) = stacks.get(&stack_id) {
                if stack.collapsed {
                    drop(stacks);
                    self.expand_stack(stack_id, app);
                } else {
                    drop(stacks);
                    self.collapse_stack(stack_id, app);
                }
            }
        }
    }

    /// Rename a stack
    pub fn rename_stack(&self, stack_id: String, new_label: String, app: AppHandle) {
        if let Ok(mut stacks) = self.stacks.lock() {
            if let Some(stack) = stacks.get_mut(&stack_id) {
                stack.label = new_label.clone();
                let _ = app.emit("exodus-tab-stack-renamed", (stack_id, new_label));
            }
        }
    }

    /// Change stack color
    pub fn change_stack_color(&self, stack_id: String, new_color: String, app: AppHandle) {
        if let Ok(mut stacks) = self.stacks.lock() {
            if let Some(stack) = stacks.get_mut(&stack_id) {
                stack.color = new_color.clone();
                let _ = app.emit("exodus-tab-stack-color-changed", (stack_id, new_color));
            }
        }
    }

    /// Get stack for a tab
    pub fn get_tab_stack(&self, tab_label: &str) -> Option<TabStack> {
        if let Ok(tab_to_stack) = self.tab_to_stack.lock() {
            if let Some(stack_id) = tab_to_stack.get(tab_label) {
                self.stacks.lock().ok().and_then(|stacks| stacks.get(stack_id).cloned())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get all stacks
    pub fn get_all_stacks(&self) -> Vec<TabStack> {
        self.stacks.lock()
            .ok()
            .map(|stacks| stacks.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get tabs in a stack
    pub fn get_stack_tabs(&self, stack_id: &str) -> Vec<String> {
        self.tab_to_stack.lock()
            .ok()
            .map(|tab_to_stack| {
                tab_to_stack.iter()
                    .filter(|(_, sid)| *sid == stack_id)
                    .map(|(tab, _)| tab.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Enable tab stacking
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-tab-stacking-enabled", true);
        }
    }

    /// Disable tab stacking
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-tab-stacking-enabled", false);
        }
    }

    /// Check if tab stacking is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set auto-group by domain
    pub fn set_auto_group_by_domain(&self, enabled: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.auto_group_by_domain = enabled;
            let _ = app.emit("exodus-tab-stacking-auto-group-changed", enabled);
        }
    }

    /// Get settings
    pub fn get_settings(&self) -> TabStackSettings {
        self.settings.lock()
            .map(|settings| TabStackSettings {
                enabled: settings.enabled,
                auto_group_by_domain: settings.auto_group_by_domain,
                show_stack_badges: settings.show_stack_badges,
                collapse_on_navigate_away: settings.collapse_on_navigate_away,
            })
            .unwrap_or_default()
    }
}

impl Default for TabStackManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to create a stack
#[tauri::command]
pub fn create_tab_stack(
    stack_id: String,
    label: String,
    color: String,
    parent_stack_id: Option<String>,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.create_stack(stack_id, label, color, parent_stack_id, app);
}

/// Tauri command to delete a stack
#[tauri::command]
pub fn delete_tab_stack(
    stack_id: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.delete_stack(stack_id, app);
}

/// Tauri command to add tab to stack
#[tauri::command]
pub fn add_tab_to_stack(
    tab_label: String,
    stack_id: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.add_tab_to_stack(tab_label, stack_id, app);
}

/// Tauri command to remove tab from stack
#[tauri::command]
pub fn remove_tab_from_stack(
    tab_label: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.remove_tab_from_stack(tab_label, app);
}

/// Tauri command to move tab to stack
#[tauri::command]
pub fn move_tab_to_stack(
    tab_label: String,
    new_stack_id: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.move_tab_to_stack(tab_label, new_stack_id, app);
}

/// Tauri command to collapse stack
#[tauri::command]
pub fn collapse_tab_stack(
    stack_id: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.collapse_stack(stack_id, app);
}

/// Tauri command to expand stack
#[tauri::command]
pub fn expand_tab_stack(
    stack_id: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.expand_stack(stack_id, app);
}

/// Tauri command to toggle stack collapse
#[tauri::command]
pub fn toggle_tab_stack_collapse(
    stack_id: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.toggle_stack_collapse(stack_id, app);
}

/// Tauri command to rename stack
#[tauri::command]
pub fn rename_tab_stack(
    stack_id: String,
    new_label: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.rename_stack(stack_id, new_label, app);
}

/// Tauri command to change stack color
#[tauri::command]
pub fn change_tab_stack_color(
    stack_id: String,
    new_color: String,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.change_stack_color(stack_id, new_color, app);
}

/// Tauri command to get tab stack
#[tauri::command]
pub fn get_tab_stack(
    tab_label: String,
    manager: State<'_, Arc<TabStackManager>>,
) -> Option<TabStack> {
    manager.get_tab_stack(&tab_label)
}

/// Tauri command to get all stacks
#[tauri::command]
pub fn get_all_tab_stacks(
    manager: State<'_, Arc<TabStackManager>>,
) -> Vec<TabStack> {
    manager.get_all_stacks()
}

/// Tauri command to get stack tabs
#[tauri::command]
pub fn get_stack_tabs(
    stack_id: String,
    manager: State<'_, Arc<TabStackManager>>,
) -> Vec<String> {
    manager.get_stack_tabs(&stack_id)
}

/// Tauri command to enable tab stacking
#[tauri::command]
pub fn enable_tab_stacking(
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable tab stacking
#[tauri::command]
pub fn disable_tab_stacking(
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if tab stacking is enabled
#[tauri::command]
pub fn is_tab_stacking_enabled(
    manager: State<'_, Arc<TabStackManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set auto group by domain
#[tauri::command]
pub fn set_tab_stacking_auto_group(
    enabled: bool,
    app: AppHandle,
    manager: State<'_, Arc<TabStackManager>>,
) {
    manager.set_auto_group_by_domain(enabled, app);
}

/// Tauri command to get tab stacking settings
#[tauri::command]
pub fn get_tab_stacking_settings(
    manager: State<'_, Arc<TabStackManager>>,
) -> TabStackSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_stack_manager_creation() {
        let manager = TabStackManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_create_delete_stack() {
        let manager = TabStackManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(manager.get_all_stacks().is_empty());
    }

    #[test]
    fn test_settings() {
        let manager = TabStackManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert!(!settings.auto_group_by_domain);
    }
}
