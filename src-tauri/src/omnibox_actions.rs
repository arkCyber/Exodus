//! Omnibox Quick Actions for Exodus Browser
//! Provides quick actions like calculator, unit conversion, etc. from the address bar

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Omnibox action type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OmniboxAction {
    Calculator {
        expression: String,
        result: String,
    },
    UnitConversion {
        from: String,
        to: String,
        value: f64,
        result: f64,
    },
    Search {
        query: String,
        engine: String,
    },
    None,
}

/// Omnibox action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmnActionResult {
    pub action: OmniboxAction,
    pub display_text: String,
    pub url: Option<String>,
}

/// Omnibox Actions Manager
pub struct OmniboxActionsManager {
    enabled: Arc<Mutex<bool>>,
}

impl OmniboxActionsManager {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// Enable omnibox actions
    pub fn enable(&self) {
        if let Ok(mut enabled) = self.enabled.lock() {
            *enabled = true;
        }
    }

    /// Disable omnibox actions
    pub fn disable(&self) {
        if let Ok(mut enabled) = self.enabled.lock() {
            *enabled = false;
        }
    }

    /// Check if omnibox actions are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.lock()
            .map(|enabled| *enabled)
            .unwrap_or(false)
    }

    /// Parse input and determine if it's a quick action
    pub fn parse_input(&self, input: &str) -> Option<OmnActionResult> {
        if !self.is_enabled() {
            return None;
        }

        let input = input.trim();
        
        // Try calculator first
        if let Some(result) = self.try_calculator(input) {
            return Some(result);
        }

        // Try unit conversion
        if let Some(result) = self.try_unit_conversion(input) {
            return Some(result);
        }

        None
    }

    /// Try to parse as calculator expression
    fn try_calculator(&self, input: &str) -> Option<OmnActionResult> {
        // Simple calculator - check for basic math operations
        if input.contains('+') || input.contains('-') || input.contains('*') || input.contains('/') {
            if let Ok(result) = self.evaluate_math(input) {
                return Some(OmnActionResult {
                    action: OmniboxAction::Calculator {
                        expression: input.to_string(),
                        result: format!("{}", result),
                    },
                    display_text: format!("{} = {}", input, result),
                    url: None,
                });
            }
        }
        None
    }

    /// Evaluate simple math expression
    fn evaluate_math(&self, input: &str) -> Result<f64, String> {
        // Very simple evaluator - just handles basic operations
        // In a real implementation, you'd use a proper expression parser
        let input = input.replace(" ", "");
        
        if let Some((left, right)) = input.split_once('+') {
            let l: f64 = left.parse().map_err(|_| "Invalid number")?;
            let r: f64 = right.parse().map_err(|_| "Invalid number")?;
            return Ok(l + r);
        }
        
        if let Some((left, right)) = input.split_once('-') {
            let l: f64 = left.parse().map_err(|_| "Invalid number")?;
            let r: f64 = right.parse().map_err(|_| "Invalid number")?;
            return Ok(l - r);
        }
        
        if let Some((left, right)) = input.split_once('*') {
            let l: f64 = left.parse().map_err(|_| "Invalid number")?;
            let r: f64 = right.parse().map_err(|_| "Invalid number")?;
            return Ok(l * r);
        }
        
        if let Some((left, right)) = input.split_once('/') {
            let l: f64 = left.parse().map_err(|_| "Invalid number")?;
            let r: f64 = right.parse().map_err(|_| "Invalid number")?;
            if r == 0.0 {
                return Err("Division by zero".to_string());
            }
            return Ok(l / r);
        }
        
        Err("Invalid expression".to_string())
    }

    /// Try to parse as unit conversion
    fn try_unit_conversion(&self, input: &str) -> Option<OmnActionResult> {
        // Simple unit conversion - look for "to" pattern
        // e.g., "100 km to miles"
        let lower = input.to_lowercase();
        
        if lower.contains(" to ") {
            let parts: Vec<&str> = lower.split(" to ").collect();
            if parts.len() == 2 {
                let from_part = parts[0].trim();
                let to_unit = parts[1].trim();
                
                // Parse value and from unit
                if let Some((value_str, from_unit)) = self.split_value_and_unit(from_part) {
                    if let Ok(value) = value_str.parse::<f64>() {
                        if let Some(result) = self.convert_unit(value, from_unit, to_unit) {
                            return Some(OmnActionResult {
                                action: OmniboxAction::UnitConversion {
                                    from: from_unit.to_string(),
                                    to: to_unit.to_string(),
                                    value,
                                    result,
                                },
                                display_text: format!("{} {} = {} {}", value, from_unit, result, to_unit),
                                url: None,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// Split value and unit from string
    fn split_value_and_unit<'a>(&self, input: &'a str) -> Option<(&'a str, &'a str)> {
        let input = input.trim();
        // Find where the number ends
        let mut split_idx = 0;
        for (i, c) in input.char_indices() {
            if c.is_whitespace() || !c.is_ascii_digit() && c != '.' && c != '-' {
                split_idx = i;
                break;
            }
        }
        
        if split_idx > 0 {
            let value = &input[..split_idx];
            let unit = input[split_idx..].trim();
            if !unit.is_empty() {
                return Some((value, unit));
            }
        }
        None
    }

    /// Convert between units (simplified implementation)
    fn convert_unit(&self, value: f64, from: &str, to: &str) -> Option<f64> {
        // Very simplified conversion - just a few examples
        // In a real implementation, you'd have a comprehensive conversion library
        
        let from_lower = from.to_lowercase();
        let to_lower = to.to_lowercase();
        
        // Length conversions
        if from_lower == "km" && to_lower == "miles" {
            return Some(value * 0.621371);
        }
        if from_lower == "miles" && to_lower == "km" {
            return Some(value / 0.621371);
        }
        if from_lower == "m" && to_lower == "ft" {
            return Some(value * 3.28084);
        }
        if from_lower == "ft" && to_lower == "m" {
            return Some(value / 3.28084);
        }
        
        // Weight conversions
        if from_lower == "kg" && to_lower == "lbs" {
            return Some(value * 2.20462);
        }
        if from_lower == "lbs" && to_lower == "kg" {
            return Some(value / 2.20462);
        }
        
        // Temperature conversions
        if from_lower == "c" && to_lower == "f" {
            return Some(value * 9.0 / 5.0 + 32.0);
        }
        if from_lower == "f" && to_lower == "c" {
            return Some((value - 32.0) * 5.0 / 9.0);
        }
        
        None
    }
}

impl Default for OmniboxActionsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable omnibox actions
#[tauri::command]
pub fn enable_omnibox_actions(
    manager: State<'_, Arc<OmniboxActionsManager>>,
) {
    manager.enable();
}

/// Tauri command to disable omnibox actions
#[tauri::command]
pub fn disable_omnibox_actions(
    manager: State<'_, Arc<OmniboxActionsManager>>,
) {
    manager.disable();
}

/// Tauri command to check if omnibox actions are enabled
#[tauri::command]
pub fn is_omnibox_actions_enabled(
    manager: State<'_, Arc<OmniboxActionsManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to parse omnibox input
#[tauri::command]
pub fn parse_omnibox_input(
    input: String,
    manager: State<'_, Arc<OmniboxActionsManager>>,
) -> Option<OmnActionResult> {
    manager.parse_input(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_omnibox_actions_manager_creation() {
        let manager = OmniboxActionsManager::new();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_calculator() {
        let manager = OmniboxActionsManager::new();
        
        let result = manager.parse_input("2 + 3");
        assert!(result.is_some());
        
        let action = result.expect("Expected result to exist");
        match action.action {
            OmniboxAction::Calculator { expression, result } => {
                assert_eq!(expression, "2 + 3");
                assert_eq!(result, "5");
            }
            _ => panic!("Expected Calculator action"),
        }
    }

    #[test]
    fn test_calculator_subtraction() {
        let manager = OmniboxActionsManager::new();
        
        let result = manager.parse_input("10 - 3");
        assert!(result.is_some());
        
        let action = result.expect("Expected result to exist");
        match action.action {
            OmniboxAction::Calculator { result, .. } => {
                assert_eq!(result, "7");
            }
            _ => panic!("Expected Calculator action"),
        }
    }

    #[test]
    fn test_calculator_multiplication() {
        let manager = OmniboxActionsManager::new();
        
        let result = manager.parse_input("4 * 5");
        assert!(result.is_some());
        
        let action = result.expect("Expected result to exist");
        match action.action {
            OmniboxAction::Calculator { result, .. } => {
                assert_eq!(result, "20");
            }
            _ => panic!("Expected Calculator action"),
        }
    }

    #[test]
    fn test_calculator_division() {
        let manager = OmniboxActionsManager::new();
        
        let result = manager.parse_input("20 / 4");
        assert!(result.is_some());
        
        let action = result.expect("Expected result to exist");
        match action.action {
            OmniboxAction::Calculator { result, .. } => {
                assert_eq!(result, "5");
            }
            _ => panic!("Expected Calculator action"),
        }
    }

    #[test]
    fn test_unit_conversion() {
        let manager = OmniboxActionsManager::new();
        
        let result = manager.parse_input("100 km to miles");
        assert!(result.is_some());
        
        let action = result.expect("Expected result to exist");
        match action.action {
            OmniboxAction::UnitConversion { from, to, value, result } => {
                assert_eq!(from, "km");
                assert_eq!(to, "miles");
                assert_eq!(value, 100.0);
                assert!((result - 62.1371).abs() < 0.01);
            }
            _ => panic!("Expected UnitConversion action"),
        }
    }

    #[test]
    fn test_no_action() {
        let manager = OmniboxActionsManager::new();
        
        let result = manager.parse_input("hello world");
        assert!(result.is_none());
    }

    #[test]
    fn test_enable_disable() {
        let manager = OmniboxActionsManager::new();
        
        manager.disable();
        assert!(!manager.is_enabled());
        
        manager.enable();
        assert!(manager.is_enabled());
    }
}
