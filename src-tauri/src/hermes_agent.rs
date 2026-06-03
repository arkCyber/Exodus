//! Hermes Agent - 智能体核心，负责策略规划和执行

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::microservice::allama_http_client::{AllamaHttpClient, ChatMessage as AllamaChatMessage};
use crate::microservice::ALLAMA_DEFAULT_PORT;

/// 智能体任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// 智能体任务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    Navigation,
    FormFill,
    DataExtraction,
    Analysis,
    Automation,
    Custom(String),
}

/// 智能体任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub task_type: TaskType,
    pub description: String,
    pub status: TaskStatus,
    pub priority: u32,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// 智能体策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStrategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<StrategyStep>,
    pub enabled: bool,
    pub created_at: u64,
}

/// 策略步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStep {
    pub id: String,
    pub step_type: String,
    pub action: String,
    pub parameters: serde_json::Value,
    pub condition: Option<String>,
}

/// 智能体上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub current_url: Option<String>,
    pub tab_id: Option<String>,
    pub variables: HashMap<String, serde_json::Value>,
    pub history: Vec<String>,
}

/// 智能体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesConfig {
    pub enabled: bool,
    pub max_concurrent_tasks: usize,
    pub task_timeout_secs: u64,
    pub enable_learning: bool,
    pub enable_python_backend: bool,
    pub python_service_url: Option<String>,
    /// Allama HTTP port (Ollama replacement). Default 11435.
    pub allama_http_port: u16,
    /// Optional override base URL for Allama (`http://127.0.0.1:11435`).
    pub allama_base_url: Option<String>,
}

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_tasks: 5,
            task_timeout_secs: 300,
            enable_learning: true,
            enable_python_backend: false,
            python_service_url: None,
            allama_http_port: ALLAMA_DEFAULT_PORT,
            allama_base_url: None,
        }
    }
}

/// 智能体统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesStats {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub active_tasks: usize,
    pub total_strategies: usize,
    pub active_strategies: usize,
    pub average_task_duration_ms: f64,
}

/// Hermes 智能体核心
pub struct HermesAgent {
    tasks: Arc<RwLock<HashMap<String, AgentTask>>>,
    strategies: Arc<RwLock<HashMap<String, AgentStrategy>>>,
    context: Arc<RwLock<AgentContext>>,
    config: Arc<RwLock<HermesConfig>>,
    stats: Arc<RwLock<HermesStats>>,
    task_queue: Arc<RwLock<Vec<String>>>,
    allama_http_port: Arc<RwLock<u16>>,
}

impl HermesAgent {
    pub fn new() -> Self {
        Self::with_allama_port(ALLAMA_DEFAULT_PORT)
    }

    /// Create Hermes with Allama HTTP on `port` (default 11435).
    pub fn with_allama_port(port: u16) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            strategies: Arc::new(RwLock::new(HashMap::new())),
            context: Arc::new(RwLock::new(AgentContext {
                session_id: uuid::Uuid::new_v4().to_string(),
                user_id: None,
                current_url: None,
                tab_id: None,
                variables: HashMap::new(),
                history: Vec::new(),
            })),
            config: Arc::new(RwLock::new(HermesConfig::default())),
            stats: Arc::new(RwLock::new(HermesStats {
                total_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                active_tasks: 0,
                total_strategies: 0,
                active_strategies: 0,
                average_task_duration_ms: 0.0,
            })),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            allama_http_port: Arc::new(RwLock::new(port)),
        }
    }

    /// Update Allama HTTP port (keeps config in sync).
    pub async fn set_allama_http_port(&self, port: u16) {
        *self.allama_http_port.write().await = port;
        let mut cfg = self.config.write().await;
        cfg.allama_http_port = port;
    }

    async fn allama_client(&self) -> AllamaHttpClient {
        let cfg = self.config.read().await;
        if let Some(ref url) = cfg.allama_base_url {
            return AllamaHttpClient::new(url.clone());
        }
        let port = *self.allama_http_port.read().await;
        AllamaHttpClient::from_port(port)
    }

    async fn allama_chat_for_task(
        &self,
        task: &AgentTask,
        system_hint: &str,
    ) -> Result<String, String> {
        let client = self.allama_client().await;
        if !client.probe().await {
            return Err("Allama HTTP API is offline on configured port".to_string());
        }
        let model = task
            .metadata
            .get("model")
            .cloned()
            .unwrap_or_else(|| "exodus-default".to_string());
        let user_prompt = task
            .metadata
            .get("prompt")
            .cloned()
            .unwrap_or_else(|| task.description.clone());
        client
            .chat(
                &model,
                vec![
                    AllamaChatMessage {
                        role: "system".to_string(),
                        content: system_hint.to_string(),
                    },
                    AllamaChatMessage {
                        role: "user".to_string(),
                        content: user_prompt,
                    },
                ],
                Some(512),
                None,
            )
            .await
    }

    /// 创建新任务
    pub async fn create_task(
        &self,
        task_type: TaskType,
        description: String,
        priority: u32,
        metadata: HashMap<String, String>,
    ) -> Result<String, String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err("Hermes agent is disabled".to_string());
        }
        drop(config);

        let task_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let task = AgentTask {
            id: task_id.clone(),
            task_type,
            description,
            status: TaskStatus::Pending,
            priority,
            created_at: now,
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            metadata,
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);

        let mut queue = self.task_queue.write().await;
        queue.push(task_id.clone());

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_tasks += 1;

        Ok(task_id)
    }

    /// 执行任务
    pub async fn execute_task(&self, task_id: &str) -> Result<serde_json::Value, String> {
        let mut tasks = self.tasks.write().await;
        
        let task = tasks.get_mut(task_id)
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        if task.status != TaskStatus::Pending {
            return Err(format!("Task {} is not in pending state", task_id));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        task.status = TaskStatus::InProgress;
        task.started_at = Some(now);
        let task_type = task.task_type.clone();
        drop(tasks);

        // Execute task based on type
        let result = match task_type {
            TaskType::Navigation => self.execute_navigation_task(task_id).await,
            TaskType::FormFill => self.execute_form_fill_task(task_id).await,
            TaskType::DataExtraction => self.execute_extraction_task(task_id).await,
            TaskType::Analysis => self.execute_analysis_task(task_id).await,
            TaskType::Automation => self.execute_automation_task(task_id).await,
            TaskType::Custom(_) => self.execute_custom_task(task_id).await,
        };

        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(task_id)
            .ok_or_else(|| format!("Task {} not found", task_id))?;
        let completed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        match result {
            Ok(value) => {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(completed_at);
                task.result = Some(value.clone());

                let mut stats = self.stats.write().await;
                stats.completed_tasks += 1;
                stats.active_tasks = stats.active_tasks.saturating_sub(1);

                // Update average duration
                if let Some(started) = task.started_at {
                    let duration = (completed_at - started) * 1000;
                    let total_duration = stats.average_task_duration_ms * (stats.completed_tasks - 1) as f64;
                    stats.average_task_duration_ms = (total_duration + duration as f64) / stats.completed_tasks as f64;
                }

                Ok(value)
            }
            Err(e) => {
                task.status = TaskStatus::Failed;
                task.completed_at = Some(completed_at);
                task.error = Some(e.clone());

                let mut stats = self.stats.write().await;
                stats.failed_tasks += 1;
                stats.active_tasks = stats.active_tasks.saturating_sub(1);

                Err(e)
            }
        }
    }

    /// 执行导航任务
    async fn execute_navigation_task(&self, _task_id: &str) -> Result<serde_json::Value, String> {
        // In real implementation, this would navigate to a URL
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(serde_json::json!({
            "status": "success",
            "message": "Navigation completed"
        }))
    }

    /// 执行表单填充任务
    async fn execute_form_fill_task(&self, _task_id: &str) -> Result<serde_json::Value, String> {
        // In real implementation, this would fill form fields
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        Ok(serde_json::json!({
            "status": "success",
            "message": "Form filled successfully"
        }))
    }

    /// 执行数据提取任务
    async fn execute_extraction_task(&self, _task_id: &str) -> Result<serde_json::Value, String> {
        // In real implementation, this would extract data from page
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        Ok(serde_json::json!({
            "status": "success",
            "data": {
                "title": "Extracted Data",
                "items": vec!["item1", "item2", "item3"]
            }
        }))
    }

    /// 执行分析任务（优先 Allama HTTP :11435）
    async fn execute_analysis_task(&self, task_id: &str) -> Result<serde_json::Value, String> {
        let task = self
            .tasks
            .read()
            .await
            .get(task_id)
            .cloned()
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        if task.metadata.get("inference_engine").map(|s| s.as_str()) == Some("stub") {
            return self.execute_analysis_task_stub().await;
        }

        match self
            .allama_chat_for_task(&task, "You are Hermes. Analyze the user content concisely.")
            .await
        {
            Ok(summary) => Ok(serde_json::json!({
                "status": "success",
                "backend": "allama-http",
                "analysis": {
                    "summary": summary,
                }
            })),
            Err(e) => {
                eprintln!("[Hermes] Allama analysis fallback: {e}");
                self.execute_analysis_task_stub().await
            }
        }
    }

    async fn execute_analysis_task_stub(&self) -> Result<serde_json::Value, String> {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(serde_json::json!({
            "status": "success",
            "backend": "stub",
            "analysis": {
                "sentiment": "neutral",
                "summary": "Content analysis completed (Allama offline)"
            }
        }))
    }

    /// 执行自动化任务
    async fn execute_automation_task(&self, _task_id: &str) -> Result<serde_json::Value, String> {
        // In real implementation, this would execute automation script
        tokio::time::sleep(Duration::from_millis(250)).await;
        
        Ok(serde_json::json!({
            "status": "success",
            "message": "Automation completed"
        }))
    }

    /// 执行自定义任务（metadata `use_allama=true` 时走 Allama HTTP）
    async fn execute_custom_task(&self, task_id: &str) -> Result<serde_json::Value, String> {
        let task = self
            .tasks
            .read()
            .await
            .get(task_id)
            .cloned()
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        let use_allama = task.metadata.get("use_allama").map(|v| v == "true").unwrap_or(false)
            || task.metadata.get("inference_engine").map(|s| s.as_str()) == Some("allama");

        if use_allama {
            let text = self
                .allama_chat_for_task(&task, "You are Hermes. Complete the custom task.")
                .await?;
            return Ok(serde_json::json!({
                "status": "success",
                "backend": "allama-http",
                "message": text,
            }));
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(serde_json::json!({
            "status": "success",
            "backend": "stub",
            "message": "Custom task completed"
        }))
    }

    /// 获取任务
    pub async fn get_task(&self, task_id: &str) -> Option<AgentTask> {
        self.tasks.read().await.get(task_id).cloned()
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<AgentTask> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// 获取策略
    pub async fn get_strategy(&self, strategy_id: &str) -> Option<AgentStrategy> {
        self.strategies.read().await.get(strategy_id).cloned()
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        
        let task = tasks.get_mut(task_id)
            .ok_or_else(|| format!("Task {} not found", task_id))?;

        if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
            return Err(format!("Task {} is already completed or failed", task_id));
        }

        task.status = TaskStatus::Cancelled;
        task.completed_at = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs());

        let mut stats = self.stats.write().await;
        stats.active_tasks = stats.active_tasks.saturating_sub(1);

        Ok(())
    }

    /// 创建策略
    pub async fn create_strategy(
        &self,
        name: String,
        description: String,
        steps: Vec<StrategyStep>,
    ) -> Result<String, String> {
        let strategy_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let strategy = AgentStrategy {
            id: strategy_id.clone(),
            name,
            description,
            steps,
            enabled: true,
            created_at: now,
        };

        let mut strategies = self.strategies.write().await;
        strategies.insert(strategy_id.clone(), strategy);

        let mut stats = self.stats.write().await;
        stats.total_strategies += 1;
        stats.active_strategies += 1;

        Ok(strategy_id)
    }

    /// 执行策略
    pub async fn execute_strategy(&self, strategy_id: &str) -> Result<Vec<serde_json::Value>, String> {
        let strategies = self.strategies.read().await;
        let strategy = strategies.get(strategy_id)
            .ok_or_else(|| format!("Strategy {} not found", strategy_id))?;

        if !strategy.enabled {
            return Err(format!("Strategy {} is disabled", strategy_id));
        }
        let steps = strategy.steps.clone();
        drop(strategies);

        let mut results = Vec::new();

        for step in &steps {
            let task_id = self.create_task(
                TaskType::Custom(step.step_type.clone()),
                format!("Execute step: {}", step.action),
                1,
                HashMap::new(),
            ).await?;

            let result = self.execute_task(&task_id).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 更新上下文
    pub async fn update_context(&self, context: AgentContext) {
        let mut ctx = self.context.write().await;
        *ctx = context;
    }

    /// 获取上下文
    pub async fn get_context(&self) -> AgentContext {
        self.context.read().await.clone()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> HermesStats {
        self.stats.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, config: HermesConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    /// 获取配置
    pub async fn get_config(&self) -> HermesConfig {
        self.config.read().await.clone()
    }
}

impl Default for HermesAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_task() {
        let agent = HermesAgent::new();
        let task_id = agent.create_task(
            TaskType::Navigation,
            "Test task".to_string(),
            1,
            HashMap::new(),
        ).await;

        assert!(task_id.is_ok());
        let task = agent.get_task(&task_id.expect("Expected task_id")).await;
        assert!(task.is_some());
    }

    #[tokio::test]
    async fn test_execute_task() {
        let agent = HermesAgent::new();
        let task_id = agent.create_task(
            TaskType::Navigation,
            "Test task".to_string(),
            1,
            HashMap::new(),
        ).await.expect("Failed to create task");

        let result = agent.execute_task(&task_id).await;
        assert!(result.is_ok());

        let task = agent.get_task(&task_id).await.expect("Expected task");
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let agent = HermesAgent::new();
        let task_id = agent.create_task(
            TaskType::Navigation,
            "Test task".to_string(),
            1,
            HashMap::new(),
        ).await.expect("Failed to create task");

        let result = agent.cancel_task(&task_id).await;
        assert!(result.is_ok());

        let task = agent.get_task(&task_id).await.expect("Expected task");
        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_get_all_tasks() {
        let agent = HermesAgent::new();
        
        agent.create_task(TaskType::Navigation, "Task 1".to_string(), 1, HashMap::new()).await.expect("Failed to create task 1");
        agent.create_task(TaskType::FormFill, "Task 2".to_string(), 2, HashMap::new()).await.expect("Failed to create task 2");
        
        let tasks = agent.get_all_tasks().await;
        assert!(tasks.len() >= 2);
    }

    #[tokio::test]
    async fn test_strategy() {
        let agent = HermesAgent::new();
        let step = StrategyStep {
            id: "step1".to_string(),
            step_type: "navigation".to_string(),
            action: "navigate".to_string(),
            parameters: serde_json::json!({}),
            condition: None,
        };

        let strategy_id = agent.create_strategy(
            "Test Strategy".to_string(),
            "Test description".to_string(),
            vec![step],
        ).await;

        assert!(strategy_id.is_ok());
        let strategy = agent.get_strategy(&strategy_id.expect("Expected strategy_id")).await;
        assert!(strategy.is_some());
    }

    #[tokio::test]
    async fn test_execute_strategy() {
        let agent = HermesAgent::new();
        let step = StrategyStep {
            id: "step1".to_string(),
            step_type: "navigation".to_string(),
            action: "navigate".to_string(),
            parameters: serde_json::json!({}),
            condition: None,
        };

        let strategy_id = agent.create_strategy(
            "Test Strategy".to_string(),
            "Test description".to_string(),
            vec![step],
        ).await.expect("Failed to create strategy");

        let results = agent.execute_strategy(&strategy_id).await;
        assert!(results.is_ok());
    }

    #[tokio::test]
    async fn test_context_management() {
        let agent = HermesAgent::new();
        
        let mut context = agent.get_context().await;
        context.variables.insert("test_key".to_string(), serde_json::json!("test_value"));
        agent.update_context(context).await;
        
        let retrieved_context = agent.get_context().await;
        assert_eq!(retrieved_context.variables.get("test_key"), Some(&serde_json::json!("test_value")));
    }

    #[tokio::test]
    async fn test_config_update() {
        let agent = HermesAgent::new();
        let new_config = HermesConfig {
            enabled: true,
            max_concurrent_tasks: 5,
            task_timeout_secs: 60,
            enable_learning: true,
            enable_python_backend: false,
            python_service_url: None,
            allama_http_port: ALLAMA_DEFAULT_PORT,
            allama_base_url: None,
        };
        
        agent.update_config(new_config.clone()).await;
        let retrieved_config = agent.get_config().await;
        
        assert_eq!(retrieved_config.enable_python_backend, false);
        assert_eq!(retrieved_config.max_concurrent_tasks, 5);
    }

    #[tokio::test]
    async fn test_stats() {
        let agent = HermesAgent::new();
        let stats = agent.get_stats().await;
        
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
        assert_eq!(stats.failed_tasks, 0);
    }

    #[tokio::test]
    async fn test_multiple_task_types() {
        let agent = HermesAgent::new();
        
        let nav_task = agent.create_task(TaskType::Navigation, "Nav task".to_string(), 1, HashMap::new()).await.expect("Failed to create nav task");
        let form_task = agent.create_task(TaskType::FormFill, "Form task".to_string(), 1, HashMap::new()).await.expect("Failed to create form task");
        let data_task = agent.create_task(TaskType::DataExtraction, "Data task".to_string(), 1, HashMap::new()).await.expect("Failed to create data task");
        
        assert!(agent.get_task(&nav_task).await.is_some());
        assert!(agent.get_task(&form_task).await.is_some());
        assert!(agent.get_task(&data_task).await.is_some());
    }
}
