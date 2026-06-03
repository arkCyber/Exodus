//! Python 微服务集成测试
//! 测试 Python 微服务的启动、执行和管理

#[cfg(test)]
mod integration_tests {
    use crate::python_microservice::{
        PythonMicroservice, PythonConfig, PythonExecutionRequest, PythonExecutionResponse
    };
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_python_microservice_initialization() {
        let microservice = PythonMicroservice::new();
        
        // 验证初始状态
        let info = microservice.get_info().await;
        assert!(info.is_ok());
        
        let info = info.unwrap();
        assert_eq!(info.name, "python_microservice");
        assert!(!info.running);
    }

    #[tokio::test]
    async fn test_python_config_management() {
        let microservice = PythonMicroservice::new();
        
        let config = PythonConfig {
            python_path: PathBuf::from("/usr/bin/python3"),
            script_dir: PathBuf::from("./scripts"),
            timeout: 30,
            max_workers: 4,
            enable_sandbox: true,
        };
        
        let result = microservice.update_config(config).await;
        assert!(result.is_ok());
        
        // 验证配置
        let retrieved_config = microservice.get_config().await;
        assert!(retrieved_config.is_ok());
    }

    #[tokio::test]
    async fn test_python_execution_request() {
        let request = PythonExecutionRequest {
            code: "print('Hello, World!')".to_string(),
            args: vec![],
            timeout: Some(10),
        };
        
        assert_eq!(request.code, "print('Hello, World!')");
        assert_eq!(request.args.len(), 0);
    }

    #[tokio::test]
    async fn test_python_execution_response() {
        let response = PythonExecutionResponse {
            success: true,
            stdout: "Hello, World!\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            execution_time: 0.1,
        };
        
        assert!(response.success);
        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout, "Hello, World!\n");
    }

    #[tokio::test]
    async fn test_python_microservice_status() {
        let microservice = PythonMicroservice::new();
        
        let status = microservice.get_status().await;
        assert!(status.is_ok());
        
        let status = status.unwrap();
        assert_eq!(status, "stopped");
    }

    #[tokio::test]
    async fn test_python_config_defaults() {
        let microservice = PythonMicroservice::new();
        
        let config = microservice.get_config().await;
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert!(config.timeout > 0);
        assert!(config.max_workers > 0);
    }

    #[tokio::test]
    async fn test_python_execution_with_args() {
        let request = PythonExecutionRequest {
            code: "import sys; print(sys.argv)".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            timeout: Some(10),
        };
        
        assert_eq!(request.args.len(), 2);
        assert_eq!(request.args[0], "arg1");
    }

    #[tokio::test]
    async fn test_python_execution_timeout() {
        let request = PythonExecutionRequest {
            code: "import time; time.sleep(100)".to_string(),
            args: vec![],
            timeout: Some(1), // 1秒超时
        };
        
        assert_eq!(request.timeout, Some(1));
    }

    #[tokio::test]
    async fn test_python_error_response() {
        let response = PythonExecutionResponse {
            success: false,
            stdout: String::new(),
            stderr: "SyntaxError: invalid syntax".to_string(),
            exit_code: 1,
            execution_time: 0.05,
        };
        
        assert!(!response.success);
        assert_eq!(response.exit_code, 1);
        assert!(response.stderr.contains("SyntaxError"));
    }

    #[tokio::test]
    async fn test_python_sandbox_config() {
        let microservice = PythonMicroservice::new();
        
        let config = PythonConfig {
            python_path: PathBuf::from("/usr/bin/python3"),
            script_dir: PathBuf::from("./scripts"),
            timeout: 30,
            max_workers: 2,
            enable_sandbox: true, // 启用沙箱
        };
        
        let result = microservice.update_config(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_python_multi_worker_config() {
        let microservice = PythonMicroservice::new();
        
        let config = PythonConfig {
            python_path: PathBuf::from("/usr/bin/python3"),
            script_dir: PathBuf::from("./scripts"),
            timeout: 60,
            max_workers: 8, // 8个工作进程
            enable_sandbox: false,
        };
        
        let result = microservice.update_config(config).await;
        assert!(result.is_ok());
    }
}
