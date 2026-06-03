//! Hermes 智能体集成测试
//! 测试 Hermes 智能体的任务管理、策略链和上下文管理

#[cfg(test)]
mod integration_tests {
    use crate::hermes_agent::{
        HermesAgent, TaskType, TaskPriority, StrategyStep, HermesConfig,
        HermesContext, HermesStats
    };
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_hermes_agent_full_workflow() {
        let agent = HermesAgent::new();
        
        // 1. 创建任务
        let task_id = agent.create_task(
            TaskType::Navigation,
            "Navigate to example.com".to_string(),
            TaskPriority::High,
            HashMap::from([("url".to_string(), "https://example.com".to_string())])
        ).await;
        
        assert!(task_id.is_ok());
        
        let task_id = task_id.unwrap();
        
        // 2. 获取任务
        let task = agent.get_task(&task_id).await;
        assert!(task.is_some());
        assert_eq!(task.unwrap().description, "Navigate to example.com");
        
        // 3. 获取所有任务
        let tasks = agent.get_all_tasks().await;
        assert_eq!(tasks.len(), 1);
        
        // 4. 获取统计
        let stats = agent.get_stats().await;
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.active_tasks, 1);
    }

    #[tokio::test]
    async fn test_strategy_chain_creation() {
        let agent = HermesAgent::new();
        
        let steps = vec![
            StrategyStep {
                id: "step1".to_string(),
                step_type: "Navigation".to_string(),
                action: "navigate".to_string(),
                parameters: HashMap::from([("url".to_string(), "https://example.com".to_string())]),
                condition: None,
            },
            StrategyStep {
                id: "step2".to_string(),
                step_type: "DataExtraction".to_string(),
                action: "extract".to_string(),
                parameters: HashMap::from([("selector".to_string(), "h1".to_string())]),
                condition: None,
            },
        ];
        
        let strategy_id = agent.create_strategy(
            "Test Strategy".to_string(),
            "Test description".to_string(),
            steps
        ).await;
        
        assert!(strategy_id.is_ok());
        
        // 获取策略
        let strategy = agent.get_strategy(&strategy_id.unwrap()).await;
        assert!(strategy.is_some());
    }

    #[tokio::test]
    async fn test_context_management() {
        let agent = HermesAgent::new();
        
        // 更新上下文
        agent.update_context(
            HashMap::from([("user_id".to_string(), "123".to_string())])
        ).await;
        
        // 获取上下文
        let context = agent.get_context().await;
        assert_eq!(context.get("user_id"), Some(&"123".to_string()));
        
        // 添加更多上下文
        agent.update_context(
            HashMap::from([("session_id".to_string(), "abc".to_string())])
        ).await;
        
        let context = agent.get_context().await;
        assert_eq!(context.len(), 2);
    }

    #[tokio::test]
    async fn test_config_management() {
        let agent = HermesAgent::new();
        
        let new_config = HermesConfig {
            enabled: true,
            enable_python_backend: true,
            max_concurrent_tasks: 10,
            task_timeout: 300,
            enable_learning: true,
            enable_context_compression: true,
        };
        
        agent.update_config(new_config).await;
        
        let config = agent.get_config().await;
        assert!(config.enabled);
        assert_eq!(config.max_concurrent_tasks, 10);
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let agent = HermesAgent::new();
        
        let task_id = agent.create_task(
            TaskType::Custom,
            "Test task".to_string(),
            TaskPriority::Medium,
            HashMap::new()
        ).await.unwrap();
        
        // 取消任务
        let result = agent.cancel_task(&task_id).await;
        assert!(result.is_ok());
        
        // 验证任务状态
        let task = agent.get_task(&task_id).await;
        assert!(task.is_some());
    }

    #[tokio::test]
    async fn test_multiple_tasks() {
        let agent = HermesAgent::new();
        
        // 创建多个任务
        let task_ids = vec![
            agent.create_task(TaskType::Navigation, "Task 1".to_string(), TaskPriority::High, HashMap::new()).await.unwrap(),
            agent.create_task(TaskType::DataExtraction, "Task 2".to_string(), TaskPriority::Medium, HashMap::new()).await.unwrap(),
            agent.create_task(TaskType::Automation, "Task 3".to_string(), TaskPriority::Low, HashMap::new()).await.unwrap(),
        ];
        
        // 验证任务数量
        let tasks = agent.get_all_tasks().await;
        assert_eq!(tasks.len(), 3);
        
        // 验证任务ID
        for task_id in task_ids {
            let task = agent.get_task(&task_id).await;
            assert!(task.is_some());
        }
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let agent = HermesAgent::new();
        
        // 创建任务
        agent.create_task(TaskType::Custom, "Test".to_string(), TaskPriority::Medium, HashMap::new()).await.unwrap();
        
        let stats = agent.get_stats().await;
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.active_tasks, 1);
        assert_eq!(stats.completed_tasks, 0);
        assert_eq!(stats.failed_tasks, 0);
    }

    #[tokio::test]
    async fn test_task_priority() {
        let agent = HermesAgent::new();
        
        let high_priority = agent.create_task(
            TaskType::Custom,
            "High priority".to_string(),
            TaskPriority::High,
            HashMap::new()
        ).await.unwrap();
        
        let low_priority = agent.create_task(
            TaskType::Custom,
            "Low priority".to_string(),
            TaskPriority::Low,
            HashMap::new()
        ).await.unwrap();
        
        let high_task = agent.get_task(&high_priority).await;
        let low_task = agent.get_task(&low_priority).await;
        
        assert_eq!(high_task.unwrap().priority, TaskPriority::High);
        assert_eq!(low_task.unwrap().priority, TaskPriority::Low);
    }

    #[tokio::test]
    async fn test_task_types() {
        let agent = HermesAgent::new();
        
        let task_types = vec![
            TaskType::Navigation,
            TaskType::FormFill,
            TaskType::DataExtraction,
            TaskType::Automation,
            TaskType::Custom,
        ];
        
        for task_type in task_types {
            let task_id = agent.create_task(
                task_type.clone(),
                format!("{:?} task", task_type),
                TaskPriority::Medium,
                HashMap::new()
            ).await;
            
            assert!(task_id.is_ok());
        }
        
        let tasks = agent.get_all_tasks().await;
        assert_eq!(tasks.len(), 5);
    }

    #[tokio::test]
    async fn test_strategy_execution() {
        let agent = HermesAgent::new();
        
        let steps = vec![
            StrategyStep {
                id: "step1".to_string(),
                step_type: "Navigation".to_string(),
                action: "navigate".to_string(),
                parameters: HashMap::from([("url".to_string(), "https://example.com".to_string())]),
                condition: None,
            },
        ];
        
        let strategy_id = agent.create_strategy(
            "Simple Strategy".to_string(),
            "Test".to_string(),
            steps
        ).await.unwrap();
        
        // 执行策略
        let result = agent.execute_strategy(&strategy_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_conditional_steps() {
        let agent = HermesAgent::new();
        
        let steps = vec![
            StrategyStep {
                id: "step1".to_string(),
                step_type: "Navigation".to_string(),
                action: "navigate".to_string(),
                parameters: HashMap::new(),
                condition: None,
            },
            StrategyStep {
                id: "step2".to_string(),
                step_type: "DataExtraction".to_string(),
                action: "extract".to_string(),
                parameters: HashMap::new(),
                condition: Some("step1_success".to_string()),
            },
        ];
        
        let strategy_id = agent.create_strategy(
            "Conditional Strategy".to_string(),
            "Test".to_string(),
            steps
        ).await;
        
        assert!(strategy_id.is_ok());
    }
}
