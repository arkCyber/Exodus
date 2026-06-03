//! Exodus Browser — Authentication Service
//!
//! Provides user authentication for cloud sync with JWT tokens,
//! password hashing, and session management.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

/// User account
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub last_login: Option<chrono::DateTime<Utc>>,
}

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
}

/// Authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Authentication error
#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    TokenExpired,
    InvalidToken,
    EmailAlreadyExists,
    HashError(String),
    JwtError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::TokenExpired => write!(f, "Token expired"),
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::EmailAlreadyExists => write!(f, "Email already exists"),
            AuthError::HashError(msg) => write!(f, "Hash error: {}", msg),
            AuthError::JwtError(msg) => write!(f, "JWT error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

/// Authentication service
pub struct AuthService {
    jwt_secret: String,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
    users: Arc<Mutex<Vec<User>>>,
    refresh_tokens: Arc<Mutex<std::collections::HashMap<String, (String, i64)>>>, // token -> (user_id, expiry)
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            access_token_expiry: Duration::hours(1),
            refresh_token_expiry: Duration::days(30),
            users: Arc::new(Mutex::new(Vec::new())),
            refresh_tokens: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Hash password using bcrypt
    fn hash_password(password: &str) -> Result<String, AuthError> {
        hash(password, DEFAULT_COST)
            .map_err(|e| AuthError::HashError(e.to_string()))
    }

    /// Verify password using bcrypt
    fn verify_password(password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }

    /// Register a new user
    pub fn register(&self, email: String, password: String, display_name: String) -> Result<User, AuthError> {
        let mut users = self.users.lock().map_err(|e| AuthError::HashError(e.to_string()))?;

        // Check if email already exists
        if users.iter().any(|u| u.email == email) {
            return Err(AuthError::EmailAlreadyExists);
        }

        let password_hash = Self::hash_password(&password)?;
        let user = User {
            user_id: Uuid::new_v4().to_string(),
            email: email.clone(),
            password_hash,
            display_name,
            created_at: Utc::now(),
            last_login: None,
        };

        users.push(user.clone());
        Ok(user)
    }

    /// Login user
    pub fn login(&self, email: String, password: String) -> Result<Tokens, AuthError> {
        let mut users = self.users.lock().map_err(|e| AuthError::HashError(e.to_string()))?;

        // Find user by email
        let user = users.iter_mut()
            .find(|u| u.email == email)
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        if !Self::verify_password(&password, &user.password_hash) {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last login
        user.last_login = Some(Utc::now());

        // Generate tokens
        self.generate_tokens(&user.user_id, &user.email)
    }

    /// Generate JWT tokens
    fn generate_tokens(&self, user_id: &str, email: &str) -> Result<Tokens, AuthError> {
        let now = Utc::now();
        let exp = now + self.access_token_expiry;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());
        let access_token = encode(&Header::default(), &claims, &encoding_key)
            .map_err(|e| AuthError::JwtError(e.to_string()))?;

        // Generate refresh token and store it
        let refresh_token = Uuid::new_v4().to_string();
        let refresh_expiry = Utc::now() + self.refresh_token_expiry;
        
        let mut refresh_tokens = self.refresh_tokens.lock()
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        refresh_tokens.insert(refresh_token.clone(), (user_id.to_string(), refresh_expiry.timestamp()));

        Ok(Tokens {
            access_token,
            refresh_token,
            expires_in: self.access_token_expiry.num_seconds(),
        })
    }

    /// Verify access token
    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let token_data = decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::HS256))
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            })?;

        Ok(token_data.claims)
    }

    /// Refresh access token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<Tokens, AuthError> {
        // Validate refresh token format (UUID)
        let _refresh_uuid = Uuid::parse_str(refresh_token)
            .map_err(|_| AuthError::InvalidToken)?;

        // Get refresh token mapping
        let mut refresh_tokens = self.refresh_tokens.lock()
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        
        // Check if refresh token exists and is not expired
        let (user_id, expiry) = refresh_tokens.get(refresh_token)
            .ok_or_else(|| AuthError::InvalidToken)?
            .clone();
        
        let now = Utc::now().timestamp();
        if now > expiry {
            refresh_tokens.remove(refresh_token);
            return Err(AuthError::TokenExpired);
        }

        // Get user
        let users = self.users.lock()
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        let user = users.iter()
            .find(|u| u.user_id == user_id)
            .ok_or_else(|| AuthError::UserNotFound)?;

        // Generate new tokens
        let new_tokens = self.generate_tokens(&user.user_id, &user.email)?;
        
        // Remove old refresh token and add new one
        refresh_tokens.remove(refresh_token);
        let new_refresh_expiry = Utc::now() + self.refresh_token_expiry;
        refresh_tokens.insert(new_tokens.refresh_token.clone(), (user_id, new_refresh_expiry.timestamp()));

        Ok(new_tokens)
    }

    /// Get user by ID
    pub fn get_user(&self, user_id: &str) -> Option<User> {
        let users = self.users.lock().ok()?;
        users.iter().find(|u| u.user_id == user_id).cloned()
    }

    /// Get user by email
    pub fn get_user_by_email(&self, email: &str) -> Option<User> {
        let users = self.users.lock().ok()?;
        users.iter().find(|u| u.email == email).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service_creation() {
        let service = AuthService::new("test-secret-key".to_string());
        assert_eq!(service.get_user("nonexistent"), None);
    }

    #[test]
    fn test_user_registration() {
        let service = AuthService::new("test-secret-key".to_string());
        
        let user = service.register(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test User".to_string(),
        );
        
        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, "Test User");
        assert_ne!(user.password_hash, "password123"); // Should be hashed
    }

    #[test]
    fn test_duplicate_email_registration() {
        let service = AuthService::new("test-secret-key".to_string());
        
        // First registration
        let _ = service.register(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test User".to_string(),
        );
        
        // Second registration with same email should fail
        let result = service.register(
            "test@example.com".to_string(),
            "password456".to_string(),
            "Another User".to_string(),
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::EmailAlreadyExists));
    }

    #[test]
    fn test_user_login_success() {
        let service = AuthService::new("test-secret-key".to_string());
        
        // Register user
        let _ = service.register(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test User".to_string(),
        );
        
        // Login with correct credentials
        let tokens = service.login(
            "test@example.com".to_string(),
            "password123".to_string(),
        );
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert!(tokens.expires_in > 0);
    }

    #[test]
    fn test_refresh_token() {
        let service = AuthService::new("test-secret-key".to_string());
        
        // Register and login
        let _ = service.register(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test User".to_string(),
        );
        
        let tokens = service.login(
            "test@example.com".to_string(),
            "password123".to_string(),
        ).unwrap();
        
        // Refresh token
        let new_tokens = service.refresh_token(&tokens.refresh_token);
        assert!(new_tokens.is_ok());
        let new_tokens = new_tokens.unwrap();
        assert!(!new_tokens.access_token.is_empty());
        assert_ne!(new_tokens.access_token, tokens.access_token);
    }

    #[test]
    fn test_refresh_token_invalid() {
        let service = AuthService::new("test-secret-key".to_string());
        
        // Try to refresh with invalid token
        let result = service.refresh_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_user_login_wrong_password() {
        let service = AuthService::new("test-secret-key".to_string());
        
        // Register user
        let _ = service.register(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test User".to_string(),
        );
        
        // Login with wrong password
        let result = service.login(
            "test@example.com".to_string(),
            "wrongpassword".to_string(),
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    }

    #[test]
    fn test_token_verification() {
        let service = AuthService::new("test-secret-key".to_string());
        
        // Register and login
        let _ = service.register(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test User".to_string(),
        );
        
        let tokens = service.login(
            "test@example.com".to_string(),
            "password123".to_string(),
        ).unwrap();
        
        // Verify token
        let claims = service.verify_token(&tokens.access_token);
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn test_get_user_by_id() {
        let service = AuthService::new("test-secret-key".to_string());
        
        // Register user
        let user = service.register(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test User".to_string(),
        ).unwrap();
        
        // Get user by ID
        let retrieved = service.get_user(&user.user_id);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.user_id, user.user_id);
        assert_eq!(retrieved.email, user.email);
    }

    #[test]
    fn test_get_user_by_email() {
        let service = AuthService::new("test-secret-key".to_string());
        
        // Register user
        let _ = service.register(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test User".to_string(),
        );
        
        // Get user by email
        let retrieved = service.get_user_by_email("test@example.com");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.email, "test@example.com");
    }
}
