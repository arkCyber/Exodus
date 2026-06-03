#!/usr/bin/env python3
"""
Exodus Python 微服务
独立进程运行的 Python 微服务，通过 Unix Domain Socket 与 Rust 后端通信
"""

import argparse
import json
import os
import socket
import sys
import threading
from typing import Dict, Any, Optional

# 可选的库导入
try:
    import numpy as np
    NUMPY_AVAILABLE = True
except ImportError:
    NUMPY_AVAILABLE = False

try:
    import pandas as pd
    PANDAS_AVAILABLE = True
except ImportError:
    PANDAS_AVAILABLE = False

try:
    import torch
    TORCH_AVAILABLE = True
except ImportError:
    TORCH_AVAILABLE = False


class PythonService:
    """Python 微服务主类"""
    
    def __init__(self, socket_path: str, enable_numpy: bool = False, 
                 enable_pandas: bool = False, enable_torch: bool = False):
        self.socket_path = socket_path
        self.enable_numpy = enable_numpy
        self.enable_pandas = enable_pandas
        self.enable_torch = enable_torch
        self.running = False
        self.socket = None
        
        # 全局变量存储
        self.variables: Dict[str, Any] = {}
        
    def start(self):
        """启动服务"""
        self.running = True
        
        # 移除现有 socket
        if os.path.exists(self.socket_path):
            os.remove(self.socket_path)
        
        # 创建 Unix Domain Socket
        self.socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.socket.bind(self.socket_path)
        self.socket.listen(5)
        
        print(f"Python service started on {self.socket_path}", file=sys.stderr)
        print(f"NumPy: {NUMPY_AVAILABLE}, Pandas: {PANDAS_AVAILABLE}, Torch: {TORCH_AVAILABLE}", file=sys.stderr)
        
        # 接受连接
        while self.running:
            try:
                self.socket.settimeout(1.0)
                conn, _ = self.socket.accept()
                threading.Thread(target=self.handle_client, args=(conn,)).start()
            except socket.timeout:
                continue
            except Exception as e:
                if self.running:
                    print(f"Error accepting connection: {e}", file=sys.stderr)
                break
    
    def handle_client(self, conn: socket.socket):
        """处理客户端连接"""
        try:
            while True:
                # 接收数据
                data = conn.recv(4096)
                if not data:
                    break
                
                # 解析请求
                try:
                    request = json.loads(data.decode('utf-8'))
                    response = self.execute_request(request)
                    conn.send(json.dumps(response).encode('utf-8'))
                except json.JSONDecodeError:
                    error_response = {
                        "success": False,
                        "error": "Invalid JSON"
                    }
                    conn.send(json.dumps(error_response).encode('utf-8'))
        except Exception as e:
            print(f"Error handling client: {e}", file=sys.stderr)
        finally:
            conn.close()
    
    def execute_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """执行请求"""
        import time
        start_time = time.time()
        
        try:
            request_type = request.get("type", "execute")
            
            if request_type == "execute":
                return self.execute_code(request)
            elif request_type == "set_variable":
                return self.set_variable(request)
            elif request_type == "get_variable":
                return self.get_variable(request)
            elif request_type == "health":
                return self.health_check()
            else:
                return {
                    "success": False,
                    "error": f"Unknown request type: {request_type}"
                }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "execution_time_ms": int((time.time() - start_time) * 1000)
            }
    
    def execute_code(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """执行 Python 代码"""
        import time
        code = request.get("code", "")
        variables = request.get("variables", {})
        
        start_time = time.time()
        
        try:
            # 合并变量
            local_vars = self.variables.copy()
            local_vars.update(variables)
            
            # 执行代码
            exec_globals = {
                "__builtins__": __builtins__,
                "np": np if (self.enable_numpy and NUMPY_AVAILABLE) else None,
                "pd": pd if (self.enable_pandas and PANDAS_AVAILABLE) else None,
                "torch": torch if (self.enable_torch and TORCH_AVAILABLE) else None,
            }
            
            exec(code, exec_globals, local_vars)
            
            # 捕获输出
            output = local_vars.get("_output", str(local_vars))
            
            return {
                "success": True,
                "output": str(output),
                "execution_time_ms": int((time.time() - start_time) * 1000)
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "execution_time_ms": int((time.time() - start_time) * 1000)
            }
    
    def set_variable(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """设置变量"""
        name = request.get("name")
        value = request.get("value")
        
        if name is None:
            return {"success": False, "error": "Missing variable name"}
        
        self.variables[name] = value
        return {"success": True}
    
    def get_variable(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """获取变量"""
        name = request.get("name")
        
        if name is None:
            return {"success": False, "error": "Missing variable name"}
        
        if name not in self.variables:
            return {"success": False, "error": f"Variable '{name}' not found"}
        
        return {
            "success": True,
            "value": self.variables[name]
        }
    
    def health_check(self) -> Dict[str, Any]:
        """健康检查"""
        return {
            "success": True,
            "numpy": NUMPY_AVAILABLE,
            "pandas": PANDAS_AVAILABLE,
            "torch": TORCH_AVAILABLE,
            "variables_count": len(self.variables)
        }
    
    def stop(self):
        """停止服务"""
        self.running = False
        if self.socket:
            self.socket.close()
        if os.path.exists(self.socket_path):
            os.remove(self.socket_path)


def main():
    parser = argparse.ArgumentParser(description="Exodus Python Microservice")
    parser.add_argument("--socket-path", required=True, help="Unix Domain Socket path")
    parser.add_argument("--service-name", default="python-service", help="Service name")
    parser.add_argument("--enable-numpy", action="store_true", help="Enable NumPy")
    parser.add_argument("--enable-pandas", action="store_true", help="Enable Pandas")
    parser.add_argument("--enable-torch", action="store_true", help="Enable PyTorch")
    
    args = parser.parse_args()
    
    service = PythonService(
        socket_path=args.socket_path,
        enable_numpy=args.enable_numpy,
        enable_pandas=args.enable_pandas,
        enable_torch=args.enable_torch
    )
    
    try:
        service.start()
    except KeyboardInterrupt:
        print("\nShutting down Python service...", file=sys.stderr)
        service.stop()


if __name__ == "__main__":
    main()
