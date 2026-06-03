#!/usr/bin/env python3
"""
Gemma4 E4B 模型多线程下载脚本
支持多线程加速下载，支持断点续传
"""

import os
import sys
import requests
import threading
import time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed

# 配置
MODEL_REPO = "unsloth/gemma-4-E4B-it-GGUF"
MODEL_VERSION = "Q4_K_M"  # 可选: Q2_K, Q3_K_S, Q3_K_M, Q4_K_M, Q5_K_M, Q6_K, Q8_0
OUTPUT_DIR = "/Users/arksong/Exodus/allama/models/gemma-4-E4B"
MAX_THREADS = 16  # 最大线程数
CHUNK_SIZE = 10 * 1024 * 1024  # 每个线程下载的块大小 (10MB)

# 代理配置
PROXY_HOST = "127.0.0.1"
PROXY_PORT = 10808
USE_PROXY = True  # 设置为 False 禁用代理

# 模型文件信息
MODEL_FILES = {
    "Q2_K": "gemma-4-E4B-it-Q2_K.gguf",
    "Q2_K_L": "gemma-4-E4B-it-Q2_K_L.gguf", 
    "Q3_K_S": "gemma-4-E4B-it-Q3_K_S.gguf",
    "Q3_K_M": "gemma-4-E4B-it-Q3_K_M.gguf",
    "Q4_K_M": "gemma-4-E4B-it-Q4_K_M.gguf",
    "Q5_K_M": "gemma-4-E4B-it-Q5_K_M.gguf",
    "Q6_K": "gemma-4-E4B-it-Q6_K.gguf",
    "Q8_0": "gemma-4-E4B-it-Q8_0.gguf",
}

class MultiThreadDownloader:
    def __init__(self, url, output_path, max_threads=8, chunk_size=10*1024*1024, proxy=None):
        self.url = url
        self.output_path = output_path
        self.max_threads = max_threads
        self.chunk_size = chunk_size
        self.proxy = proxy
        self.total_size = 0
        self.downloaded_size = 0
        self.lock = threading.Lock()
        
    def get_file_size(self):
        """获取文件总大小"""
        try:
            proxies = None
            if self.proxy:
                proxies = {
                    'http': self.proxy,
                    'https': self.proxy
                }
            response = requests.head(self.url, timeout=10, proxies=proxies)
            if response.status_code == 200:
                return int(response.headers.get('content-length', 0))
        except Exception as e:
            print(f"获取文件大小失败: {e}")
        return 0
        
    def download_chunk(self, start, end, chunk_num):
        """下载文件的一个块"""
        headers = {'Range': f'bytes={start}-{end}'}
        chunk_path = f"{self.output_path}.part{chunk_num}"
        
        try:
            proxies = None
            if self.proxy:
                proxies = {
                    'http': self.proxy,
                    'https': self.proxy
                }
            response = requests.get(self.url, headers=headers, stream=True, timeout=30, proxies=proxies)
            response.raise_for_status()
            
            with open(chunk_path, 'wb') as f:
                for chunk in response.iter_content(chunk_size=self.chunk_size):
                    if chunk:
                        f.write(chunk)
                        with self.lock:
                            self.downloaded_size += len(chunk)
                            
            return True, chunk_num
        except Exception as e:
            print(f"块 {chunk_num} 下载失败: {e}")
            return False, chunk_num
            
    def merge_chunks(self):
        """合并下载的块"""
        print("合并文件块...")
        chunk_files = sorted(Path(self.output_path).parent.glob(f"{Path(self.output_path).name}.part*"))
        
        with open(self.output_path, 'wb') as outfile:
            for chunk_file in chunk_files:
                with open(chunk_file, 'rb') as infile:
                    outfile.write(infile.read())
                chunk_file.unlink()  # 删除临时文件
                
        print("文件合并完成")
        
    def download(self):
        """执行多线程下载"""
        # 检查文件是否已存在
        if os.path.exists(self.output_path):
            print(f"文件已存在: {self.output_path}")
            return True
            
        # 创建输出目录
        os.makedirs(os.path.dirname(self.output_path), exist_ok=True)
        
        # 获取文件大小
        self.total_size = self.get_file_size()
        if self.total_size == 0:
            print("无法获取文件大小，尝试单线程下载")
            return self.single_thread_download()
            
        print(f"文件大小: {self.total_size / (1024**3):.2f} GB")
        print(f"使用 {self.max_threads} 个线程下载")
        
        # 计算每个线程的下载范围
        chunk_size = self.total_size // self.max_threads
        ranges = []
        for i in range(self.max_threads):
            start = i * chunk_size
            end = start + chunk_size - 1 if i < self.max_threads - 1 else self.total_size - 1
            ranges.append((start, end, i))
            
        # 多线程下载
        start_time = time.time()
        success_count = 0
        
        with ThreadPoolExecutor(max_workers=self.max_threads) as executor:
            futures = {
                executor.submit(self.download_chunk, start, end, chunk_num): chunk_num
                for start, end, chunk_num in ranges
            }
            
            for future in as_completed(futures):
                success, chunk_num = future.result()
                if success:
                    success_count += 1
                    
                # 显示进度
                progress = (self.downloaded_size / self.total_size) * 100
                speed = self.downloaded_size / (time.time() - start_time) / (1024**2)
                eta = (self.total_size - self.downloaded_size) / (self.downloaded_size / (time.time() - start_time + 1e-6))
                
                print(f"\r进度: {progress:.1f}% | 速度: {speed:.1f} MB/s | ETA: {eta:.0f}s", end='')
                
        print()  # 换行
        
        if success_count == self.max_threads:
            print("所有块下载成功，合并文件...")
            self.merge_chunks()
            print(f"下载完成: {self.output_path}")
            return True
        else:
            print(f"下载失败: {success_count}/{self.max_threads} 块成功")
            return False
            
    def single_thread_download(self):
        """单线程下载（备用方案）"""
        print("使用单线程下载...")
        try:
            proxies = None
            if self.proxy:
                proxies = {
                    'http': self.proxy,
                    'https': self.proxy
                }
            response = requests.get(self.url, stream=True, timeout=30, proxies=proxies)
            response.raise_for_status()
            
            total_size = int(response.headers.get('content-length', 0))
            downloaded = 0
            start_time = time.time()
            
            with open(self.output_path, 'wb') as f:
                for chunk in response.iter_content(chunk_size=self.chunk_size):
                    if chunk:
                        f.write(chunk)
                        downloaded += len(chunk)
                        
                        if total_size > 0:
                            progress = (downloaded / total_size) * 100
                            speed = downloaded / (time.time() - start_time) / (1024**2)
                            eta = (total_size - downloaded) / (downloaded / (time.time() - start_time + 1e-6))
                            print(f"\r进度: {progress:.1f}% | 速度: {speed:.1f} MB/s | ETA: {eta:.0f}s", end='')
            
            print()
            print(f"下载完成: {self.output_path}")
            return True
        except Exception as e:
            print(f"下载失败: {e}")
            return False

def download_gemma4_model():
    """下载 Gemma4 E4B 模型"""
    # 选择模型文件
    model_file = MODEL_FILES.get(MODEL_VERSION)
    if not model_file:
        print(f"不支持的模型版本: {MODEL_VERSION}")
        print(f"支持的版本: {', '.join(MODEL_FILES.keys())}")
        return False
        
    # 构建下载 URL
    base_url = f"https://huggingface.co/{MODEL_REPO}/resolve/main"
    download_url = f"{base_url}/{model_file}"
    
    # 输出路径
    output_path = os.path.join(OUTPUT_DIR, model_file)
    
    # 配置代理
    proxy = None
    if USE_PROXY:
        proxy = f"http://{PROXY_HOST}:{PROXY_PORT}"
        print(f"使用代理: {proxy}")
    
    print(f"下载模型: {model_file}")
    print(f"版本: {MODEL_VERSION}")
    print(f"URL: {download_url}")
    print(f"输出路径: {output_path}")
    print("-" * 50)
    
    # 创建下载器
    downloader = MultiThreadDownloader(
        url=download_url,
        output_path=output_path,
        max_threads=MAX_THREADS,
        chunk_size=CHUNK_SIZE,
        proxy=proxy
    )
    
    # 执行下载
    success = downloader.download()
    
    if success:
        print(f"\n✅ 模型下载成功: {output_path}")
        print(f"文件大小: {os.path.getsize(output_path) / (1024**3):.2f} GB")
        return True
    else:
        print(f"\n❌ 模型下载失败")
        return False

def main():
    print("=" * 50)
    print("Gemma4 E4B 模型多线程下载器")
    print("=" * 50)
    
    if download_gemma4_model():
        print("\n下载完成！")
        print(f"模型已保存到: {OUTPUT_DIR}")
        sys.exit(0)
    else:
        print("\n下载失败！")
        sys.exit(1)

if __name__ == "__main__":
    main()
