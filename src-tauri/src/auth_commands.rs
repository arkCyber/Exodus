//! Exodus Browser — Authentication Tauri commands
//!
//! Provides Tauri commands for user authentication

use tauri::State;

use super::auth::{AuthService, Tokens, User, AuthError};

/// Register a new user
#[tauri::command]
pub fn auth_register(
    email: String,
    password: String,
    display_name: String,
    state: State<'_, AuthService>,
) -> Result<User, String> {
    state.register(email, password, display_name)
        .map_err(|e| e.to_string())
}

/// Login user
#[tauri::command]
pub fn auth_login(
    email: String,
    password: String,
    state: State<'_, AuthService>,
) -> Result<Tokens, String> {
    state.login(email, password)
        .map_err(|e| e.to_string())
}

/// Verify access token
#[tauri::command]
pub fn auth_verify_token(
    token: String,
    state: State<'_, AuthService>,
) -> Result<bool, String> {
    state.verify_token(&token)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

/// Refresh access token
#[tauri::command]
pub fn auth_refresh_token(
    refresh_token: String,
    state: State<'_, AuthService>,
) -> Result<Tokens, String> {
    state.refresh_token(&refresh_token)
        .map_err(|e| e.to_string())
}

/// Get user by ID
#[tauri::command]
pub fn auth_get_user(
    user_id: String,
    state: State<'_, AuthService>,
) -> Result<Option<User>, String> {
    Ok(state.get_user(&user_id))
}

/// Get user by email
#[tauri::command]
pub fn auth_get_user_by_email(
    email: String,
    state: State<'_, AuthService>,
) -> Result<Option<User>, String> {
    Ok(state.get_user_by_email(&email))
}
