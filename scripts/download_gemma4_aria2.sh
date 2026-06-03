#!/bin/bash
# Gemma4 E4B 模型多线程下载脚本 (使用 aria2c)

# 配置
MODEL_REPO="unsloth/gemma-4-E4B-it-GGUF"
MODEL_VERSION="Q4_K_M"
OUTPUT_DIR="/Users/arksong/Exodus/allama/models/gemma-4-E4B"
MAX_THREADS=8
PROXY_HOST="127.0.0.1"
PROXY_PORT=10808
USE_PROXY=true

# 模型文件信息
MODEL_FILES=(
    "Q2_K:gemma-4-E4B-it-Q2_K.gguf"
    "Q2_K_L:gemma-4-E4B-it-Q2_K_L.gguf"
    "Q3_K_S:gemma-4-E4B-it-Q3_K_S.gguf"
    "Q3_K_M:gemma-4-E4B-it-Q3_K_M.gguf"
    "Q4_K_M:gemma-4-E4B-it-Q4_K_M.gguf"
    "Q5_K_M:gemma-4-E4B-it-Q5_K_M.gguf"
    "Q6_K:gemma-4-E4B-it-Q6_K.gguf"
    "Q8_0:gemma-4-E4B-it-Q8_0.gguf"
)

# 查找对应的模型文件
MODEL_FILE=""
for item in "${MODEL_FILES[@]}"; do
    version="${item%%:*}"
    file="${item##*:}"
    if [ "$version" = "$MODEL_VERSION" ]; then
        MODEL_FILE="$file"
        break
    fi
done

if [ -z "$MODEL_FILE" ]; then
    echo "不支持的模型版本: $MODEL_VERSION"
    echo "支持的版本: Q2_K, Q2_K_L, Q3_K_S, Q3_K_M, Q4_K_M, Q5_K_M, Q6_K, Q8_0"
    exit 1
fi

# 构建下载 URL
DOWNLOAD_URL="https://huggingface.co/${MODEL_REPO}/resolve/main/${MODEL_FILE}"
OUTPUT_PATH="${OUTPUT_DIR}/${MODEL_FILE}"

echo "=================================================="
echo "Gemma4 E4B 模型多线程下载器 (aria2c)"
echo "=================================================="
echo "下载模型: $MODEL_FILE"
echo "版本: $MODEL_VERSION"
echo "URL: $DOWNLOAD_URL"
echo "输出路径: $OUTPUT_PATH"
echo "线程数: $MAX_THREADS"
echo "--------------------------------------------------"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 配置代理
PROXY_ARGS=""
if [ "$USE_PROXY" = true ]; then
    PROXY_ARGS="--all-proxy=http://${PROXY_HOST}:${PROXY_PORT}"
    echo "使用代理: http://${PROXY_HOST}:${PROXY_PORT}"
fi

# 使用 aria2c 下载
aria2c \
    --dir="$OUTPUT_DIR" \
    --out="$MODEL_FILE" \
    --max-connection-per-server=$MAX_THREADS \
    --split=$MAX_THREADS \
    --min-split-size=10M \
    --continue=true \
    --max-tries=10 \
    --retry-wait=10 \
    --timeout=120 \
    --connect-timeout=30 \
    --check-certificate=true \
    --lowest-speed-limit=10K \
    $PROXY_ARGS \
    "$DOWNLOAD_URL"

# 检查下载结果
if [ $? -eq 0 ]; then
    echo "=================================================="
    echo "✅ 模型下载成功"
    echo "文件路径: $OUTPUT_PATH"
    FILE_SIZE=$(du -h "$OUTPUT_PATH" | cut -f1)
    echo "文件大小: $FILE_SIZE"
    echo "=================================================="
    exit 0
else
    echo "=================================================="
    echo "❌ 模型下载失败"
    echo "=================================================="
    exit 1
fi
