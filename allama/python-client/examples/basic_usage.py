"""
Basic usage examples for Allama Python client
"""

from allama_client import AllamaClient

# Initialize client
client = AllamaClient(
    base_url="http://127.0.0.1:11435",
    # api_key="your-api-key"  # Optional
)

# Example 1: Simple chat
print("=== Example 1: Simple Chat ===")
response = client.chat(
    model="gemma4-4b",
    messages=[{"role": "user", "content": "Hello! What is 2+2?"}]
)
print(f"Response: {response['message']['content']}\n")

# Example 2: Streaming chat
print("=== Example 2: Streaming Chat ===")
print("Response: ", end="", flush=True)
for chunk in client.chat_stream(
    model="gemma4-4b",
    messages=[{"role": "user", "content": "Tell me a short story about a robot"}]
):
    if "message" in chunk and "content" in chunk["message"]:
        print(chunk["message"]["content"], end="", flush=True)
print("\n")

# Example 3: Text generation
print("=== Example 3: Text Generation ===")
response = client.generate(
    model="gemma4-4b",
    prompt="The future of AI is"
)
print(f"Response: {response['response']}\n")

# Example 4: List models
print("=== Example 4: List Models ===")
models = client.list_models()
print(f"Available models: {models}\n")

# Example 5: Embeddings
print("=== Example 5: Embeddings ===")
try:
    response = client.embeddings(
        model="gemma4-4b",
        input="Hello, world!"
    )
    print(f"Embedding dimension: {len(response['data'][0]['embedding'])}")
except Exception as e:
    print(f"Embeddings not available: {e}\n")

# Example 6: Anthropic API compatibility
print("=== Example 6: Anthropic API ===")
try:
    response = client.anthropic_messages(
        model="gemma4-4b",
        messages=[{"role": "user", "content": "Say hello"}],
        max_tokens=50
    )
    print(f"Response: {response['content'][0]['text']}\n")
except Exception as e:
    print(f"Anthropic API not available: {e}\n")

# Clean up
client.close()
