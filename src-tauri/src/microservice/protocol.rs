//! JSON-RPC 2.0 Protocol Implementation for Microservice Communication

use serde::{Deserialize, Serialize};
use std::fmt;

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcRequest {
    /// Must be "2.0"
    pub jsonrpc: String,
    /// Method name to invoke
    pub method: String,
    /// Method parameters (can be a structured value or an array)
    pub params: serde_json::Value,
    /// Request identifier (can be a string, number, or null)
    pub id: serde_json::Value,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    #[allow(dead_code)]
    pub fn new(method: impl Into<String>, params: serde_json::Value, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcResponse {
    /// Must be "2.0"
    pub jsonrpc: String,
    /// Result if successful (null if error)
    pub result: Option<serde_json::Value>,
    /// Error if failed (null if successful)
    pub error: Option<JsonRpcError>,
    /// Request identifier (must match the request)
    pub id: serde_json::Value,
}

impl JsonRpcResponse {
    /// Create a successful response
    #[allow(dead_code)]
    pub fn success(result: serde_json::Value, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    #[allow(dead_code)]
    pub fn error(error: JsonRpcError, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data (optional)
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Standard error codes as per JSON-RPC 2.0 specification
    #[allow(dead_code)]
    pub const PARSE_ERROR: i32 = -32700;
    #[allow(dead_code)]
    pub const INVALID_REQUEST: i32 = -32600;
    #[allow(dead_code)]
    pub const METHOD_NOT_FOUND: i32 = -32601;
    #[allow(dead_code)]
    pub const INVALID_PARAMS: i32 = -32602;
    #[allow(dead_code)]
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Create a new error
    #[allow(dead_code)]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create an error with data
    #[allow(dead_code)]
    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Parse error
    #[allow(dead_code)]
    pub fn parse_error() -> Self {
        Self::new(Self::PARSE_ERROR, "Parse error")
    }

    /// Invalid request
    #[allow(dead_code)]
    pub fn invalid_request() -> Self {
        Self::new(Self::INVALID_REQUEST, "Invalid request")
    }

    /// Method not found
    #[allow(dead_code)]
    pub fn method_not_found(method: String) -> Self {
        Self::with_data(Self::METHOD_NOT_FOUND, "Method not found", serde_json::json!({"method": method}))
    }

    /// Invalid params
    #[allow(dead_code)]
    pub fn invalid_params(detail: String) -> Self {
        Self::with_data(Self::INVALID_PARAMS, "Invalid params", serde_json::json!({"detail": detail}))
    }

    /// Internal error
    #[allow(dead_code)]
    pub fn internal_error(detail: String) -> Self {
        Self::with_data(Self::INTERNAL_ERROR, "Internal error", serde_json::json!({"detail": detail}))
    }
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request() {
        let request = JsonRpcRequest::new(
            "test.method",
            serde_json::json!({"param1": "value1"}),
            serde_json::json!(1),
        );

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test.method");
        assert_eq!(request.params, serde_json::json!({"param1": "value1"}));
        assert_eq!(request.id, serde_json::json!(1));
    }

    #[test]
    fn test_jsonrpc_request_with_null_id() {
        let request = JsonRpcRequest::new(
            "test.method",
            serde_json::json!({}),
            serde_json::Value::Null,
        );

        assert_eq!(request.id, serde_json::Value::Null);
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let response = JsonRpcResponse::success(
            serde_json::json!({"result": "success"}),
            serde_json::json!(1),
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, serde_json::json!(1));
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let error = JsonRpcError::method_not_found("unknown.method".to_string());
        let response = JsonRpcResponse::error(error, serde_json::json!(1));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.id, serde_json::json!(1));
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse::success(
            serde_json::json!({"status": "ok"}),
            serde_json::json!(1),
        );

        let json = serde_json::to_string(&response).expect("Failed to serialize response");
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        // Note: JSON serialization includes "error": null, so we check for success result
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_error_constructors() {
        let error = JsonRpcError::parse_error();
        assert_eq!(error.code, -32700);
        assert_eq!(error.message, "Parse error");

        let error = JsonRpcError::invalid_request();
        assert_eq!(error.code, -32600);

        let error = JsonRpcError::method_not_found("test.method".to_string());
        assert_eq!(error.code, -32601);
        assert!(error.data.is_some());

        let error = JsonRpcError::invalid_params("missing field".to_string());
        assert_eq!(error.code, -32602);
        assert!(error.data.is_some());

        let error = JsonRpcError::internal_error("something failed".to_string());
        assert_eq!(error.code, -32603);
        assert!(error.data.is_some());
    }

    #[test]
    fn test_error_display() {
        let error = JsonRpcError::parse_error();
        assert_eq!(error.to_string(), "[-32700] Parse error");

        let error = JsonRpcError::method_not_found("test".to_string());
        assert!(error.to_string().contains("-32601"));
        assert!(error.to_string().contains("Method not found"));
    }

    #[test]
    fn test_error_with_data() {
        let data = serde_json::json!({"detail": "param 'x' is required"});
        let error = JsonRpcError::with_data(
            JsonRpcError::INVALID_PARAMS,
            "Invalid params",
            data,
        );

        assert_eq!(error.code, -32602);
        assert!(error.data.is_some());
    }

    #[test]
    fn test_jsonrpc_request_from_json() {
        let json = r#"{
            "jsonrpc": "2.0",
            "method": "test.method",
            "params": {"x": 1},
            "id": 42
        }"#;

        let request: JsonRpcRequest = serde_json::from_str(json).expect("Failed to parse JSON");
        assert_eq!(request.method, "test.method");
        assert_eq!(request.id, serde_json::json!(42));
    }

    #[test]
    fn test_jsonrpc_response_to_json() {
        let response = JsonRpcResponse::success(
            serde_json::json!({"value": 123}),
            serde_json::json!(1),
        );

        let json = serde_json::to_string(&response).expect("Failed to serialize response");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");

        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["result"]["value"], 123);
        assert!(parsed["error"].is_null());
    }

    #[test]
    fn test_batch_roundtrip() {
        let requests = vec![
            JsonRpcRequest::new("method1", serde_json::json!({}), serde_json::json!(1)),
            JsonRpcRequest::new("method2", serde_json::json!({}), serde_json::json!(2)),
        ];

        let json = serde_json::to_string(&requests).expect("Failed to serialize requests");
        let parsed: Vec<JsonRpcRequest> = serde_json::from_str(&json).expect("Failed to parse JSON");

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].method, "method1");
        assert_eq!(parsed[1].method, "method2");
    }
}
