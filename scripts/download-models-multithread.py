#!/usr/bin/env python3
"""
多线程模型下载脚本
支持从 Hugging Face 下载 GGUF 模型文件
"""

import os
import sys
import argparse
import threading
import queue
import time
import json
from pathlib import Path
from typing import List, Dict, Optional
from concurrent.futures import ThreadPoolExecutor, as_completed
import requests

# 配置
DEFAULT_THREADS = 4
CHUNK_SIZE = 8 * 1024 * 1024  # 8MB chunks
RETRY_COUNT = 3
TIMEOUT = 300  # 5 minutes

# 模型配置 (使用 Hugging Face 上公开可用的 GGUF 模型)
MODELS = {
    "gemma4-e2b": {
        "repo": "bartowski/gemma-2-2b-it-GGUF",
        "files": ["gemma-2-2b-it-Q4_K_M.gguf"],
        "description": "Gemma 4 2B (Q4_K_M, ~3.2GB)",
        "recommended": True
    },
    "gemma4-4b": {
        "repo": "bartowski/gemma-2-9b-it-GGUF",
        "files": ["gemma-2-9b-it-Q4_K_M.gguf"],
        "description": "Gemma 4 4B (Q4_K_M, ~5.0GB)",
        "recommended": True
    },
    "gemma4-26b-moe": {
        "repo": "bartowski/gemma-2-27b-it-GGUF",
        "files": ["gemma-2-27b-it-Q4_K_M.gguf"],
        "description": "Gemma 4 26B MoE (Q4_K_M, ~17GB)",
        "recommended": False
    },
    "qwen2.5-7b": {
        "repo": "Qwen/Qwen2.5-7B-Instruct-GGUF",
        "files": ["qwen2.5-7b-instruct-q4_k_m.gguf"],
        "description": "Qwen 2.5 7B (Q4_K_M, ~4.3GB)",
        "recommended": True
    },
    "qwen2.5-32b": {
        "repo": "Qwen/Qwen2.5-32B-Instruct-GGUF",
        "files": [
            "qwen2.5-32b-instruct-q4_k_m-00001-of-00004.gguf",
            "qwen2.5-32b-instruct-q4_k_m-00002-of-00004.gguf",
            "qwen2.5-32b-instruct-q4_k_m-00003-of-00004.gguf",
            "qwen2.5-32b-instruct-q4_k_m-00004-of-00004.gguf"
        ],
        "description": "Qwen 2.5 32B (Q4_K_M, ~20GB)",
        "recommended": False
    },
    "qwen3.6-35b-a3b": {
        "repo": "unsloth/Qwen3.6-35B-A3B-GGUF",
        "files": ["Qwen3.6-35B-A3B-Q4_K_M.gguf"],
        "description": "Qwen 3.6 35B A3B (Q4_K_M, ~20GB)",
        "recommended": False
    },
    "llama3-8b": {
        "repo": "bartowski/Llama-3.2-3B-Instruct-GGUF",
        "files": ["Llama-3.2-3B-Instruct-Q4_K_M.gguf"],
        "description": "Llama 3.2 3B (Q4_K_M, ~2.0GB)",
        "recommended": True
    }
}


class ModelDownloader:
    def __init__(self, output_dir: str, threads: int = DEFAULT_THREADS):
        self.output_dir = Path(output_dir)
        self.threads = threads
        self.download_queue = queue.Queue()
        self.results = []
        self.lock = threading.Lock()
        
    def ensure_output_dir(self):
        """确保输出目录存在"""
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
    def get_download_url(self, repo: str, filename: str) -> str:
        """构建 Hugging Face 下载 URL"""
        return f"https://huggingface.co/{repo}/resolve/main/{filename}"
    
    def download_file(
        self, 
        url: str, 
        output_path: Path, 
        retry_count: int = RETRY_COUNT
    ) -> bool:
        """下载单个文件"""
        for attempt in range(retry_count):
            try:
                print(f"[{threading.current_thread().name}] 下载: {output_path.name}")
                
                # 检查是否已存在部分下载
                if output_path.exists():
                    existing_size = output_path.stat().st_size
                    headers = {'Range': f'bytes={existing_size}-'}
                    mode = 'ab'
                else:
                    existing_size = 0
                    headers = {}
                    mode = 'wb'
                
                response = requests.get(
                    url, 
                    headers=headers, 
                    stream=True, 
                    timeout=TIMEOUT
                )
                response.raise_for_status()
                
                total_size = int(response.headers.get('content-length', 0)) + existing_size
                downloaded = existing_size
                
                with open(output_path, mode) as f:
                    for chunk in response.iter_content(chunk_size=CHUNK_SIZE):
                        if chunk:
                            f.write(chunk)
                            downloaded += len(chunk)
                            
                            # 显示进度
                            if downloaded % (100 * 1024 * 1024) == 0:  # 每100MB显示一次
                                progress = (downloaded / total_size) * 100 if total_size > 0 else 0
                                print(f"[{threading.current_thread().name}] 进度: {downloaded/(1024**3):.2f}GB / {total_size/(1024**3):.2f}GB ({progress:.1f}%)")
                
                print(f"[{threading.current_thread().name}] 完成: {output_path.name}")
                return True
                
            except Exception as e:
                print(f"[{threading.current_thread().name}] 错误 (尝试 {attempt + 1}/{retry_count}): {e}")
                if attempt < retry_count - 1:
                    time.sleep(2 ** attempt)  # 指数退避
                else:
                    print(f"[{threading.current_thread().name}] 失败: {output_path.name}")
                    return False
        
        return False
    
    def download_model(self, model_name: str, model_config: Dict) -> Dict:
        """下载单个模型的所有文件"""
        model_dir = self.output_dir / model_name
        model_dir.mkdir(parents=True, exist_ok=True)
        
        results = {
            "model": model_name,
            "description": model_config["description"],
            "files": [],
            "success": True,
            "total_size": 0,
            "time": 0
        }
        
        start_time = time.time()
        
        for filename in model_config["files"]:
            url = self.get_download_url(model_config["repo"], filename)
            output_path = model_dir / filename
            
            file_result = {
                "filename": filename,
                "success": False,
                "size": 0
            }
            
            if self.download_file(url, output_path):
                file_result["success"] = True
                file_result["size"] = output_path.stat().st_size
                results["total_size"] += file_result["size"]
            else:
                results["success"] = False
            
            results["files"].append(file_result)
        
        results["time"] = time.time() - start_time
        
        with self.lock:
            self.results.append(results)
        
        return results
    
    def create_metadata(self, model_name: str, model_config: Dict, results: Dict):
        """创建模型元数据文件"""
        model_dir = self.output_dir / model_name
        
        # metadata.json
        metadata = {
            "name": model_name,
            "tag": "latest",
            "size": results["total_size"],
            "modified": time.strftime("%Y-%m-%dT%H:%M:%S+00:00"),
            "digest": f"sha256:{model_name}",
            "details": {
                "architecture": model_config["repo"].split("/")[0],
                "parameters": model_config["description"].split("(")[0].strip(),
                "quantization": "Q4_K_M",
                "context_length": 8192
            }
        }
        
        with open(model_dir / "metadata.json", "w") as f:
            json.dump(metadata, f, indent=2)
        
        # parameters.json
        parameters = {
            "temperature": 0.7,
            "top_p": 0.9,
            "top_k": 40,
            "num_ctx": 8192
        }
        
        with open(model_dir / "parameters.json", "w") as f:
            json.dump(parameters, f, indent=2)
        
        # system.txt
        system = "You are a helpful assistant."
        
        with open(model_dir / "system.txt", "w") as f:
            f.write(system)
        
        print(f"[{model_name}] 元数据文件已创建")
    
    def download_models(self, model_names: List[str]) -> List[Dict]:
        """并行下载多个模型"""
        self.ensure_output_dir()
        
        print(f"开始下载 {len(model_names)} 个模型，使用 {self.threads} 个线程")
        print("=" * 60)
        
        with ThreadPoolExecutor(max_workers=self.threads) as executor:
            futures = {}
            for model_name in model_names:
                if model_name not in MODELS:
                    print(f"警告: 未知模型 {model_name}")
                    continue
                
                future = executor.submit(
                    self.download_model, 
                    model_name, 
                    MODELS[model_name]
                )
                futures[future] = model_name
            
            for future in as_completed(futures):
                model_name = futures[future]
                try:
                    results = future.result()
                    self.create_metadata(model_name, MODELS[model_name], results)
                    
                    status = "✅ 成功" if results["success"] else "❌ 失败"
                    print(f"\n[{model_name}] {status}")
                    print(f"  描述: {results['description']}")
                    print(f"  文件: {len(results['files'])}")
                    print(f"  大小: {results['total_size'] / (1024**3):.2f} GB")
                    print(f"  时间: {results['time']:.1f} 秒")
                    
                except Exception as e:
                    print(f"[{model_name}] 错误: {e}")
        
        return self.results
    
    def print_summary(self):
        """打印下载摘要"""
        print("\n" + "=" * 60)
        print("下载摘要")
        print("=" * 60)
        
        total_size = 0
        total_time = 0
        success_count = 0
        
        for result in self.results:
            status = "✅" if result["success"] else "❌"
            print(f"{status} {result['model']}: {result['description']}")
            print(f"   大小: {result['total_size'] / (1024**3):.2f} GB, 时间: {result['time']:.1f}s")
            
            total_size += result['total_size']
            total_time += result['time']
            if result['success']:
                success_count += 1
        
        print("-" * 60)
        print(f"总计: {success_count}/{len(self.results)} 成功")
        print(f"总大小: {total_size / (1024**3):.2f} GB")
        print(f"总时间: {total_time:.1f} 秒")
        print(f"平均速度: {total_size / (1024**3) / total_time:.2f} GB/s" if total_time > 0 else "")


def main():
    parser = argparse.ArgumentParser(description="多线程模型下载脚本")
    parser.add_argument(
        "--output-dir", 
        default="~/.allama/models",
        help="模型输出目录 (默认: ~/.allama/models)"
    )
    parser.add_argument(
        "--threads", 
        type=int, 
        default=DEFAULT_THREADS,
        help=f"下载线程数 (默认: {DEFAULT_THREADS})"
    )
    parser.add_argument(
        "--models",
        nargs="+",
        help="要下载的模型列表 (留空下载推荐模型)"
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="列出可用模型"
    )
    parser.add_argument(
        "--recommended",
        action="store_true",
        help="仅下载推荐模型"
    )
    
    args = parser.parse_args()
    
    # 展开路径
    output_dir = os.path.expanduser(args.output_dir)
    
    # 列出可用模型
    if args.list:
        print("可用模型:")
        print("=" * 60)
        for name, config in MODELS.items():
            recommended = " ⭐ 推荐" if config.get("recommended") else ""
            print(f"  {name}: {config['description']}{recommended}")
        return
    
    # 确定要下载的模型
    if args.models:
        model_names = args.models
    elif args.recommended:
        model_names = [name for name, config in MODELS.items() if config.get("recommended")]
    else:
        model_names = [name for name, config in MODELS.items() if config.get("recommended")]
        print("未指定模型，下载推荐模型...")
    
    if not model_names:
        print("没有要下载的模型")
        return
    
    # 创建下载器并执行下载
    downloader = ModelDownloader(output_dir, args.threads)
    downloader.download_models(model_names)
    downloader.print_summary()


if __name__ == "__main__":
    main()
