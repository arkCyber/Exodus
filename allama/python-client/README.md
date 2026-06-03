# Allama Python Client

Official Python client library for Allama LLM inference server.

## Installation

```bash
pip install allama-client
```

Or install from source:

```bash
cd python-client
pip install -e .
```

## Quick Start

```python
from allama_client import AllamaClient

# Initialize client
client = AllamaClient(
    base_url="http://127.0.0.1:11435",
    api_key="your-api-key"  # Optional
)

# Chat completion
response = client.chat(
    model="gemma4-4b",
    messages=[{"role": "user", "content": "Hello!"}]
)
print(response["message"]["content"])

# Stream chat
for chunk in client.chat_stream(
    model="gemma4-4b",
    messages=[{"role": "user", "content": "Tell me a story"}]
):
    print(chunk["message"]["content"], end="", flush=True)

# Text completion
response = client.generate(
    model="gemma4-4b",
    prompt="Once upon a time"
)
print(response["response"])

# List models
models = client.list_models()
print(models)
```

## Features

- ✅ Chat completions (streaming and non-streaming)
- ✅ Text generation
- ✅ Model listing
- ✅ Embeddings
- ✅ OpenAI API compatibility
- ✅ Anthropic API compatibility
- ✅ Error handling
- ✅ Timeout support
- ✅ Retry logic

## API Reference

### AllamaClient

```python
AllamaClient(
    base_url: str = "http://127.0.0.1:11435",
    api_key: Optional[str] = None,
    timeout: int = 60,
    max_retries: int = 3
)
```

#### Methods

- `chat(model, messages, **kwargs)` - Chat completion
- `chat_stream(model, messages, **kwargs)` - Streaming chat completion
- `generate(model, prompt, **kwargs)` - Text generation
- `generate_stream(model, prompt, **kwargs)` - Streaming text generation
- `list_models()` - List available models
- `embeddings(model, input, **kwargs)` - Get embeddings
- `anthropic_messages(model, messages, **kwargs)` - Anthropic API compatibility

## Examples

See `examples/` directory for more examples.

## License

MIT
