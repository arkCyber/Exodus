//! Python microservice - 独立进程的 Python 微服务，通过 IPC 通信

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::microservice::allama_http_client::AllamaHttpClient;
use crate::microservice::{default_allama_base_url, ALLAMA_DEFAULT_PORT};

/// Python 微服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonMicroserviceConfig {
    pub enabled: bool,
    pub python_path: PathBuf,
    pub script_path: PathBuf,
    pub socket_path: PathBuf,
    pub enable_numpy: bool,
    pub enable_pandas: bool,
    pub enable_torch: bool,
    pub max_memory_mb: Option<u64>,
    pub restart_on_failure: bool,
    pub max_restarts: u32,
    /// Allama HTTP base URL for Python clients (`allama_client`).
    pub allama_base_url: String,
}

impl Default for PythonMicroserviceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            python_path: PathBuf::from("python3"),
            script_path: PathBuf::from("python_service.py"),
            socket_path: PathBuf::from("/tmp/exodus-python.sock"),
            enable_numpy: true,
            enable_pandas: false,
            enable_torch: false,
            max_memory_mb: Some(512),
            restart_on_failure: true,
            max_restarts: 3,
            allama_base_url: default_allama_base_url(ALLAMA_DEFAULT_PORT),
        }
    }
}

/// Python 微服务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PythonServiceStatus {
    Stopped,
    Starting,
    Running,
    Error,
    Restarting,
}

/// Python 执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonExecuteRequest {
    pub code: String,
    pub variables: Option<HashMap<String, serde_json::Value>>,
    pub timeout_secs: Option<u64>,
}

/// Python 执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonExecuteResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Python 微服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonServiceInfo {
    pub name: String,
    pub socket_path: PathBuf,
    pub pid: Option<u32>,
    pub status: PythonServiceStatus,
    pub started_at: Option<u64>,
    pub restart_count: u32,
}

/// Python 微服务管理器
pub struct PythonMicroservice {
    config: Arc<RwLock<PythonMicroserviceConfig>>,
    child: Arc<RwLock<Option<Child>>>,
    status: Arc<RwLock<PythonServiceStatus>>,
    restart_count: Arc<RwLock<u32>>,
    started_at: Arc<RwLock<Option<u64>>>,
}

impl PythonMicroservice {
    pub fn new() -> Self {
        Self::with_allama_port(ALLAMA_DEFAULT_PORT)
    }

    /// Create with default Allama base URL on `port`.
    pub fn with_allama_port(port: u16) -> Self {
        let mut cfg = PythonMicroserviceConfig::default();
        cfg.allama_base_url = default_allama_base_url(port);
        Self {
            config: Arc::new(RwLock::new(cfg)),
            child: Arc::new(RwLock::new(None)),
            status: Arc::new(RwLock::new(PythonServiceStatus::Stopped)),
            restart_count: Arc::new(RwLock::new(0)),
            started_at: Arc::new(RwLock::new(None)),
        }
    }

    /// Update Allama URL when AI port changes in settings.
    pub async fn set_allama_port(&self, port: u16) {
        let mut cfg = self.config.write().await;
        cfg.allama_base_url = default_allama_base_url(port);
    }

    /// 启动 Python 微服务
    pub async fn start(&self) -> Result<u32, String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err("Python microservice is disabled".to_string());
        }

        let mut status = self.status.write().await;
        if *status == PythonServiceStatus::Running {
            return Err("Python microservice is already running".to_string());
        }
        *status = PythonServiceStatus::Starting;
        drop(status);

        // 检查 Python 路径
        if !config.python_path.exists() {
            let mut new_status = self.status.write().await;
            *new_status = PythonServiceStatus::Error;
            return Err(format!("Python not found at: {}", config.python_path.display()));
        }

        // 检查脚本路径
        if !config.script_path.exists() {
            let mut new_status = self.status.write().await;
            *new_status = PythonServiceStatus::Error;
            return Err(format!("Python script not found at: {}", config.script_path.display()));
        }

        // 移除现有 socket
        if config.socket_path.exists() {
            std::fs::remove_file(&config.socket_path)
                .map_err(|e| format!("Failed to remove socket: {}", e))?;
        }

        // 创建 socket 目录
        if let Some(parent) = config.socket_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create socket directory: {}", e))?;
        }

        // 启动 Python 进程
        let mut cmd = Command::new(&config.python_path);
        cmd.arg(&config.script_path)
           .arg("--socket-path")
           .arg(&config.socket_path)
           .arg("--service-name")
           .arg("python-service");

        if config.enable_numpy {
            cmd.arg("--enable-numpy");
        }

        if config.enable_pandas {
            cmd.arg("--enable-pandas");
        }

        if config.enable_torch {
            cmd.arg("--enable-torch");
        }

        cmd.env("ALLAMA_BASE_URL", &config.allama_base_url);
        cmd.env("OLLAMA_HOST", &config.allama_base_url);

        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn Python process: {}", e))?;

        let pid = child.id();

        // 更新状态
        let mut status = self.status.write().await;
        *status = PythonServiceStatus::Running;
        drop(status);

        let mut started_at = self.started_at.write().await;
        *started_at = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs());

        let mut child_guard = self.child.write().await;
        *child_guard = Some(child);

        Ok(pid)
    }

    /// 停止 Python 微服务
    pub async fn stop(&self) -> Result<(), String> {
        let mut child = self.child.write().await;
        
        if let Some(mut child_process) = child.take() {
            child_process.kill()
                .map_err(|e| format!("Failed to kill Python process: {}", e))?;
            
            let mut status = self.status.write().await;
            *status = PythonServiceStatus::Stopped;
            
            let mut started_at = self.started_at.write().await;
            *started_at = None;
        }

        Ok(())
    }

    /// 重启 Python 微服务
    pub async fn restart(&self) -> Result<u32, String> {
        let config = self.config.read().await;
        let max_restarts = config.max_restarts;
        drop(config);

        let mut restart_count = self.restart_count.write().await;
        if *restart_count >= max_restarts {
            return Err(format!("Max restarts ({}) reached", max_restarts));
        }
        *restart_count += 1;
        drop(restart_count);

        let mut status = self.status.write().await;
        *status = PythonServiceStatus::Restarting;
        drop(status);

        self.stop().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.start().await
    }

    /// Test hook: mark service running without spawning a process.
    #[cfg(test)]
    pub async fn set_running_for_tests(&self) {
        *self.status.write().await = PythonServiceStatus::Running;
    }

    /// Test hook: run execute path without requiring a live Python child.
    #[cfg(test)]
    pub async fn execute_for_tests(
        &self,
        request: PythonExecuteRequest,
    ) -> Result<PythonExecuteResponse, String> {
        let start = std::time::Instant::now();
        let response = self.execute_via_ipc(request).await?;
        Ok(PythonExecuteResponse {
            success: response.success,
            output: response.output,
            error: response.error,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// 执行 Python 代码（通过 IPC）
    pub async fn execute(&self, request: PythonExecuteRequest) -> Result<PythonExecuteResponse, String> {
        let status = self.status.read().await;
        if *status != PythonServiceStatus::Running {
            return Err("Python microservice is not running".to_string());
        }
        drop(status);

        let start = std::time::Instant::now();

        // 在实际实现中，这里应该通过 Unix Domain Socket 或 JSON-RPC 与 Python 进程通信
        // 这里使用模拟实现
        let response = self.execute_via_ipc(request).await?;

        Ok(PythonExecuteResponse {
            success: response.success,
            output: response.output,
            error: response.error,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// 通过 IPC 执行（模拟）；`ALLAMA_CHAT:` prefix routes to Allama HTTP.
    async fn execute_via_ipc(&self, request: PythonExecuteRequest) -> Result<PythonExecuteResponse, String> {
        let config = self.config.read().await;
        let allama_url = config.allama_base_url.clone();
        drop(config);

        if let Some(prompt) = request.code.strip_prefix("ALLAMA_CHAT:") {
            let prompt = prompt.trim();
            let client = AllamaHttpClient::new(&allama_url);
            if !client.probe().await {
                return Ok(PythonExecuteResponse {
                    success: false,
                    output: None,
                    error: Some(format!("Allama offline at {allama_url}")),
                    execution_time_ms: 0,
                });
            }
            let model = request
                .variables
                .as_ref()
                .and_then(|v| v.get("model"))
                .and_then(|m| m.as_str())
                .unwrap_or("exodus-default");
            let text = client.generate(model, prompt, Some(256), None).await?;
            return Ok(PythonExecuteResponse {
                success: true,
                output: Some(text),
                error: None,
                execution_time_ms: 0,
            });
        }

        if request.code.contains("allama_client")
            || request.code.contains("AllamaClient")
            || request.code.contains("requests.post") && request.code.contains("/api/chat")
        {
            let client = AllamaHttpClient::new(&allama_url);
            if client.probe().await {
                let prompt = request
                    .variables
                    .as_ref()
                    .and_then(|v| v.get("prompt"))
                    .and_then(|p| p.as_str())
                    .unwrap_or("Hello from Exodus Python bridge");
                let text = client.generate("exodus-default", prompt, Some(128), None).await?;
                return Ok(PythonExecuteResponse {
                    success: true,
                    output: Some(format!(
                        "# ALLAMA_BASE_URL={allama_url}\n{text}"
                    )),
                    error: None,
                    execution_time_ms: 0,
                });
            }
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        if request.code.contains("error") || request.code.contains("Error") {
            Ok(PythonExecuteResponse {
                success: false,
                output: None,
                error: Some("Simulated error".to_string()),
                execution_time_ms: 0,
            })
        } else {
            Ok(PythonExecuteResponse {
                success: true,
                output: Some("Execution successful".to_string()),
                error: None,
                execution_time_ms: 0,
            })
        }
    }

    /// 获取服务信息
    pub async fn get_info(&self) -> PythonServiceInfo {
        let config = self.config.read().await;
        let status = self.status.read().await;
        let child = self.child.read().await;
        let restart_count = self.restart_count.read().await;
        let started_at = self.started_at.read().await;

        PythonServiceInfo {
            name: "python-service".to_string(),
            socket_path: config.socket_path.clone(),
            pid: child.as_ref().map(|c| c.id()),
            status: status.clone(),
            started_at: *started_at,
            restart_count: *restart_count,
        }
    }

    /// 更新配置
    pub async fn update_config(&self, config: PythonMicroserviceConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    /// 获取配置
    pub async fn get_config(&self) -> PythonMicroserviceConfig {
        self.config.read().await.clone()
    }

    /// 获取状态
    pub async fn get_status(&self) -> PythonServiceStatus {
        self.status.read().await.clone()
    }

    /// 监控进程状态
    pub async fn monitor_process(&self) -> JoinHandle<()> {
        let child = Arc::clone(&self.child);
        let status = Arc::clone(&self.status);
        let config = Arc::clone(&self.config);
        let restart_count = Arc::clone(&self.restart_count);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;

                let current_child = child.read().await;
                let has_child = current_child.is_some();
                drop(current_child);

                if !has_child {
                    let current_status = status.read().await;
                    let should_restart = *current_status == PythonServiceStatus::Running;
                    drop(current_status);

                    if should_restart {
                        let cfg = config.read().await;
                        if cfg.restart_on_failure {
                            let mut rc = restart_count.write().await;
                            if *rc < cfg.max_restarts {
                                *rc += 1;
                                drop(rc);
                                drop(cfg);

                                let mut new_status = status.write().await;
                                *new_status = PythonServiceStatus::Restarting;
                                drop(new_status);

                                // 重启逻辑（简化）
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                
                                let mut final_status = status.write().await;
                                *final_status = PythonServiceStatus::Error;
                            }
                        }
                    }
                }
            }
        })
    }
}

impl Default for PythonMicroservice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_python_microservice_creation() {
        let service = PythonMicroservice::new();
        let info = service.get_info().await;
        assert_eq!(info.status, PythonServiceStatus::Stopped);
    }

    #[tokio::test]
    async fn test_config_update() {
        let service = PythonMicroservice::new();
        let config = PythonMicroserviceConfig {
            enabled: false,
            ..Default::default()
        };
        service.update_config(config).await;
        
        let retrieved = service.get_config().await;
        assert!(!retrieved.enabled);
    }
}
