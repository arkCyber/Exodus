# Allama JavaScript/TypeScript Client

Official JavaScript/TypeScript client library for Allama LLM inference server.

## Installation

```bash
npm install @allama/client
# or
yarn add @allama/client
# or
pnpm add @allama/client
```

## Quick Start

```typescript
import { AllamaClient } from '@allama/client';

// Initialize client
const client = new AllamaClient({
  baseUrl: 'http://127.0.0.1:11435',
  apiKey: 'your-api-key' // Optional
});

// Chat completion
const response = await client.chat({
  model: 'gemma4-4b',
  messages: [{ role: 'user', content: 'Hello!' }]
});
console.log(response.message.content);

// Stream chat
for await (const chunk of client.chatStream({
  model: 'gemma4-4b',
  messages: [{ role: 'user', content: 'Tell me a story' }]
})) {
  console.log(chunk.message.content);
}

// Text generation
const result = await client.generate({
  model: 'gemma4-4b',
  prompt: 'Once upon a time'
});
console.log(result.response);

// List models
const models = await client.listModels();
console.log(models);
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
- ✅ TypeScript support
- ✅ Browser and Node.js support

## API Reference

### AllamaClient

```typescript
new AllamaClient({
  baseUrl?: string; // Default: 'http://127.0.0.1:11435'
  apiKey?: string;
  timeout?: number; // Default: 60000ms
  maxRetries?: number; // Default: 3
})
```

#### Methods

- `chat(options)` - Chat completion
- `chatStream(options)` - Streaming chat completion
- `generate(options)` - Text generation
- `generateStream(options)` - Streaming text generation
- `listModels()` - List available models
- `embeddings(options)` - Get embeddings
- `anthropicMessages(options)` - Anthropic API compatibility

## Examples

See `examples/` directory for more examples.

## License

MIT
