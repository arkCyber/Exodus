//! Form Auto-fill for Exodus Browser
//! 
//! This module provides intelligent form auto-fill capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Form field type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FieldType {
    Name,
    Email,
    Phone,
    Address,
    City,
    State,
    Zip,
    Country,
    CreditCard,
    Username,
    Password,
    Custom(String),
}

impl FieldType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "name" => FieldType::Name,
            "email" => FieldType::Email,
            "phone" => FieldType::Phone,
            "address" => FieldType::Address,
            "city" => FieldType::City,
            "state" => FieldType::State,
            "zip" => FieldType::Zip,
            "country" => FieldType::Country,
            "creditcard" => FieldType::CreditCard,
            "username" => FieldType::Username,
            "password" => FieldType::Password,
            _ => FieldType::Custom(s.to_string()),
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            FieldType::Name => "name",
            FieldType::Email => "email",
            FieldType::Phone => "phone",
            FieldType::Address => "address",
            FieldType::City => "city",
            FieldType::State => "state",
            FieldType::Zip => "zip",
            FieldType::Country => "country",
            FieldType::CreditCard => "creditcard",
            FieldType::Username => "username",
            FieldType::Password => "password",
            FieldType::Custom(s) => s,
        }
    }
}

/// Auto-fill entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofillEntry {
    /// Entry ID
    pub id: String,
    /// Field type
    pub field_type: FieldType,
    /// Value
    pub value: String,
    /// Label
    pub label: String,
    /// Domain
    pub domain: String,
    /// Last used timestamp
    pub last_used: u64,
    /// Use count
    pub use_count: u32,
}

impl AutofillEntry {
    pub fn new(field_type: FieldType, value: String, label: String, domain: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            field_type,
            value,
            label,
            domain,
            last_used: now,
            use_count: 1,
        }
    }
    
    #[allow(dead_code)]
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }
}

/// Auto-fill profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofillProfile {
    /// Profile ID
    pub id: String,
    /// Profile name
    pub name: String,
    /// Fields
    pub fields: HashMap<String, String>,
    /// Created timestamp
    pub created_at: u64,
    /// Last used timestamp
    pub last_used: u64,
}

impl AutofillProfile {
    pub fn new(name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            fields: HashMap::new(),
            created_at: now,
            last_used: now,
        }
    }
    
    #[allow(dead_code)]
    pub fn add_field(&mut self, field_name: String, value: String) {
        self.fields.insert(field_name, value);
    }
    
    #[allow(dead_code)]
    pub fn get_field(&self, field_name: &str) -> Option<&String> {
        self.fields.get(field_name)
    }
}

/// Auto-fill settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofillSettings {
    /// Enable auto-fill
    pub enabled: bool,
    /// Enable password saving
    pub save_passwords: bool,
    /// Enable address saving
    pub save_addresses: bool,
    /// Enable credit card saving
    pub save_credit_cards: bool,
    /// Auto-fill on page load
    pub autofill_on_load: bool,
    /// Require confirmation before auto-fill
    pub require_confirmation: bool,
    /// Max entries per field type
    pub max_entries_per_type: usize,
}

impl Default for AutofillSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            save_passwords: true,
            save_addresses: true,
            save_credit_cards: false,
            autofill_on_load: false,
            require_confirmation: true,
            max_entries_per_type: 50,
        }
    }
}

/// Form auto-fill manager
pub struct FormAutofillManager {
    entries: Arc<Mutex<Vec<AutofillEntry>>>,
    profiles: Arc<Mutex<Vec<AutofillProfile>>>,
    settings: Arc<Mutex<AutofillSettings>>,
    storage_path: PathBuf,
}

impl FormAutofillManager {
    /// Create a new form auto-fill manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            profiles: Arc::new(Mutex::new(Vec::new())),
            settings: Arc::new(Mutex::new(AutofillSettings::default())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Add an auto-fill entry
    pub fn add_entry(&self, field_type: FieldType, value: String, label: String, domain: String) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.enabled {
            return Ok(());
        }
        
        let field_type_clone = field_type.clone();
        let type_str = field_type_clone.as_str();
        let entry = AutofillEntry::new(field_type, value, label, domain);
        let entry_id = entry.id.clone();
        
        let mut entries = self.entries.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        entries.push(entry);
        
        // Remove old entries of the same type (keep only the most recent)
        entries.retain(|e| e.field_type.as_str() != type_str || e.id == entry_id);
        
        let count = entries.iter().filter(|e| e.field_type.as_str() == type_str).count();
        if count > settings.max_entries_per_type {
            entries.retain(|e| e.field_type.as_str() != type_str || e.id == entry_id);
            let to_remove = count - settings.max_entries_per_type;
            let mut removed = 0;
            entries.retain(|e| {
                if e.field_type.as_str() == type_str && e.id != entry_id && removed < to_remove {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get entries by field type and domain
    pub fn get_entries(&self, field_type: FieldType, domain: &str) -> Vec<AutofillEntry> {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        entries.iter()
            .filter(|e| e.field_type == field_type && (e.domain == domain || e.domain == "*"))
            .cloned()
            .collect()
    }
    
    /// Get all entries
    pub fn get_all_entries(&self) -> Vec<AutofillEntry> {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        entries.clone()
    }
    
    /// Remove an entry
    pub fn remove_entry(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        entries.retain(|e| e.id != id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Clear entries by domain
    pub fn clear_by_domain(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        entries.retain(|e| e.domain != domain);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Create a profile
    pub fn create_profile(&self, name: String) -> Result<String, Box<dyn std::error::Error>> {
        let profile = AutofillProfile::new(name);
        let profile_id = profile.id.clone();
        
        let mut profiles = self.profiles.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        profiles.push(profile);
        self.save_to_disk()?;
        Ok(profile_id)
    }
    
    /// Update a profile
    pub fn update_profile(&self, id: String, fields: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut profiles = self.profiles.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(profile) = profiles.iter_mut().find(|p| p.id == id) {
            profile.fields = fields;
            profile.last_used = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get a profile
    pub fn get_profile(&self, id: &str) -> Option<AutofillProfile> {
        let profiles = self.profiles.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        profiles.iter().find(|p| p.id == id).cloned()
    }
    
    /// Get all profiles
    pub fn get_all_profiles(&self) -> Vec<AutofillProfile> {
        let profiles = self.profiles.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        profiles.clone()
    }
    
    /// Delete a profile
    pub fn delete_profile(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut profiles = self.profiles.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        profiles.retain(|p| p.id != id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get auto-fill settings
    pub fn get_settings(&self) -> AutofillSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update auto-fill settings
    pub fn update_settings(&self, settings: AutofillSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get suggestions for a field
    pub fn get_suggestions(&self, field_type: FieldType, domain: &str, query: &str) -> Vec<AutofillEntry> {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let query_lower = query.to_lowercase();
        entries.iter()
            .filter(|e| {
                e.field_type == field_type &&
                (e.domain == domain || e.domain == "*") &&
                (e.value.to_lowercase().contains(&query_lower) ||
                 e.label.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("autofill_settings.json");
        let entries_path = self.storage_path.join("autofill_entries.json");
        let profiles_path = self.storage_path.join("autofill_profiles.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: AutofillSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if entries_path.exists() {
            let content = std::fs::read_to_string(&entries_path)?;
            let entries: Vec<AutofillEntry> = serde_json::from_str(&content)?;
            if let Ok(mut e) = self.entries.lock() {
                *e = entries;
            }
        }
        
        if profiles_path.exists() {
            let content = std::fs::read_to_string(&profiles_path)?;
            let profiles: Vec<AutofillProfile> = serde_json::from_str(&content)?;
            if let Ok(mut p) = self.profiles.lock() {
                *p = profiles;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("autofill_settings.json");
        let entries_path = self.storage_path.join("autofill_entries.json");
        let profiles_path = self.storage_path.join("autofill_profiles.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let entries = self.entries.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let profiles = self.profiles.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let entries_content = serde_json::to_string_pretty(&*entries)?;
        let profiles_content = serde_json::to_string_pretty(&*profiles)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&entries_path, entries_content)?;
        std::fs::write(&profiles_path, profiles_content)?;
        
        Ok(())
    }
}

/// Document-start script: capture address/email/phone field values on blur.
pub fn form_capture_init_script() -> &'static str {
    r#"(function() {
  if (window.__exodusFormCaptureInstalled) return;
  window.__exodusFormCaptureInstalled = true;
  window.__exodusFormCaptures = [];
  function detectType(el) {
    var ac = (el.getAttribute('autocomplete') || '').toLowerCase();
    var type = (el.type || '').toLowerCase();
    var name = (el.name || el.id || '').toLowerCase();
    if (type === 'email' || ac.indexOf('email') >= 0 || name.indexOf('email') >= 0) return 'email';
    if (type === 'tel' || ac.indexOf('tel') >= 0 || name.indexOf('phone') >= 0) return 'phone';
    if (ac.indexOf('name') >= 0 || name.indexOf('name') >= 0) return 'name';
    if (ac.indexOf('street') >= 0 || name.indexOf('address') >= 0) return 'address';
    if (ac.indexOf('address-level2') >= 0 || name.indexOf('city') >= 0) return 'city';
    if (name.indexOf('zip') >= 0 || name.indexOf('postal') >= 0) return 'zip';
    if (name.indexOf('country') >= 0) return 'country';
    return null;
  }
  function capture(el) {
    if (!el || el.tagName !== 'INPUT' && el.tagName !== 'TEXTAREA') return;
    var ft = detectType(el);
    if (!ft || !el.value || el.value.length < 2) return;
    window.__exodusFormCaptures.push({
      field_type: ft,
      value: el.value,
      label: el.placeholder || el.name || el.id || ft,
      domain: location.hostname
    });
    if (window.__exodusFormCaptures.length > 24) {
      window.__exodusFormCaptures.shift();
    }
  }
  document.addEventListener('change', function(e) {
    capture(e.target);
  }, true);
  document.addEventListener('blur', function(e) {
    capture(e.target);
  }, true);
})();"#
}

/// Build a fill script from field_type → value pairs (JSON-serialized).
pub fn form_fill_script(pairs: &[(String, String)]) -> String {
    let map_json = serde_json::to_string(pairs).unwrap_or_else(|_| "[]".to_string());
    format!(
        r#"(function() {{
  var pairs = {map_json};
  function fillType(ft, val) {{
    if (!val) return;
    var selectors = [];
    if (ft === 'email') selectors = ['input[type="email"]', 'input[autocomplete*="email" i]'];
    else if (ft === 'phone') selectors = ['input[type="tel"]', 'input[autocomplete*="tel" i]'];
    else if (ft === 'name') selectors = ['input[autocomplete*="name" i]', 'input[name*="name" i]'];
    else if (ft === 'address') selectors = ['input[autocomplete*="street" i]', 'input[name*="address" i]'];
    else if (ft === 'city') selectors = ['input[autocomplete*="address-level2" i]', 'input[name*="city" i]'];
    else if (ft === 'zip') selectors = ['input[name*="zip" i]', 'input[name*="postal" i]'];
    else if (ft === 'country') selectors = ['input[autocomplete*="country" i]', 'input[name*="country" i]'];
    else selectors = ['input[name*="' + ft + '" i]'];
    for (var s = 0; s < selectors.length; s++) {{
      var nodes = document.querySelectorAll(selectors[s]);
      for (var i = 0; i < nodes.length; i++) {{
        var el = nodes[i];
        if (el && !el.value) {{
          el.value = val;
          el.dispatchEvent(new Event('input', {{ bubbles: true }}));
        }}
      }}
    }}
  }}
  for (var j = 0; j < pairs.length; j++) {{
    fillType(pairs[j][0], pairs[j][1]);
  }}
}})();"#
    )
}

// Tauri Commands

/// Build form autofill script for eval in a content webview.
#[tauri::command]
pub fn form_build_fill_script(pairs: Vec<(String, String)>) -> String {
    form_fill_script(&pairs)
}

/// Add an auto-fill entry
#[tauri::command]
pub fn add_autofill_entry(
    field_type: String,
    value: String,
    label: String,
    domain: String,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<(), String> {
    let ft = FieldType::from_str(&field_type);
    manager.add_entry(ft, value, label, domain)
        .map_err(|e| format!("Failed to add entry: {}", e))
}

/// Get entries by field type and domain
#[tauri::command]
pub fn get_autofill_entries(
    field_type: String,
    domain: String,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<Vec<AutofillEntry>, String> {
    let ft = FieldType::from_str(&field_type);
    Ok(manager.get_entries(ft, &domain))
}

/// Get all entries
#[tauri::command]
pub fn get_all_autofill_entries(
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<Vec<AutofillEntry>, String> {
    Ok(manager.get_all_entries())
}

/// Remove an entry
#[tauri::command]
pub fn remove_autofill_entry(
    id: String,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<(), String> {
    manager.remove_entry(&id)
        .map_err(|e| format!("Failed to remove entry: {}", e))
}

/// Clear entries by domain
#[tauri::command]
pub fn clear_autofill_by_domain(
    domain: String,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<(), String> {
    manager.clear_by_domain(&domain)
        .map_err(|e| format!("Failed to clear entries: {}", e))
}

/// Create a profile
#[tauri::command]
pub fn create_autofill_profile(
    name: String,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<String, String> {
    manager.create_profile(name)
        .map_err(|e| format!("Failed to create profile: {}", e))
}

/// Update a profile
#[tauri::command]
pub fn update_autofill_profile(
    id: String,
    fields: HashMap<String, String>,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<(), String> {
    manager.update_profile(id, fields)
        .map_err(|e| format!("Failed to update profile: {}", e))
}

/// Get a profile
#[tauri::command]
pub fn get_autofill_profile(
    id: String,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<Option<AutofillProfile>, String> {
    Ok(manager.get_profile(&id))
}

/// Get all profiles
#[tauri::command]
pub fn get_all_autofill_profiles(
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<Vec<AutofillProfile>, String> {
    Ok(manager.get_all_profiles())
}

/// Delete a profile
#[tauri::command]
pub fn delete_autofill_profile(
    id: String,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<(), String> {
    manager.delete_profile(&id)
        .map_err(|e| format!("Failed to delete profile: {}", e))
}

/// Get auto-fill settings
#[tauri::command]
pub fn get_autofill_settings(
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<AutofillSettings, String> {
    Ok(manager.get_settings())
}

/// Update auto-fill settings
#[tauri::command]
pub fn update_autofill_settings(
    settings: AutofillSettings,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get suggestions for a field
#[tauri::command]
pub fn get_autofill_suggestions(
    field_type: String,
    domain: String,
    query: String,
    manager: State<'_, Arc<FormAutofillManager>>,
) -> Result<Vec<AutofillEntry>, String> {
    let ft = FieldType::from_str(&field_type);
    Ok(manager.get_suggestions(ft, &domain, &query))
}
