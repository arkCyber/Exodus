/**
 * Basic usage examples for Allama JavaScript/TypeScript client
 */

import { AllamaClient } from '../src';

async function main() {
  // Initialize client
  const client = new AllamaClient({
    baseUrl: 'http://127.0.0.1:11435',
    // apiKey: 'your-api-key' // Optional
  });

  try {
    // Example 1: Simple chat
    console.log('=== Example 1: Simple Chat ===');
    const chatResponse = await client.chat({
      model: 'gemma4-4b',
      messages: [{ role: 'user', content: 'Hello! What is 2+2?' }],
    });
    console.log(`Response: ${chatResponse.message.content}\n`);

    // Example 2: Streaming chat
    console.log('=== Example 2: Streaming Chat ===');
    console.log('Response: ');
    for await (const chunk of client.chatStream({
      model: 'gemma4-4b',
      messages: [{ role: 'user', content: 'Tell me a short story about a robot' }],
    })) {
      if (chunk.message?.content) {
        process.stdout.write(chunk.message.content);
      }
    }
    console.log('\n');

    // Example 3: Text generation
    console.log('=== Example 3: Text Generation ===');
    const generateResponse = await client.generate({
      model: 'gemma4-4b',
      prompt: 'The future of AI is',
    });
    console.log(`Response: ${generateResponse.response}\n`);

    // Example 4: List models
    console.log('=== Example 4: List Models ===');
    const models = await client.listModels();
    console.log(`Available models: ${JSON.stringify(models, null, 2)}\n`);

    // Example 5: Embeddings
    console.log('=== Example 5: Embeddings ===');
    try {
      const embeddingsResponse = await client.embeddings({
        model: 'gemma4-4b',
        input: 'Hello, world!',
      });
      console.log(`Embedding dimension: ${embeddingsResponse.data[0].embedding.length}`);
    } catch (error: any) {
      console.log(`Embeddings not available: ${error.message}\n`);
    }

    // Example 6: Anthropic API compatibility
    console.log('=== Example 6: Anthropic API ===');
    try {
      const anthropicResponse = await client.anthropicMessages({
        model: 'gemma4-4b',
        messages: [{ role: 'user', content: 'Say hello' }],
        max_tokens: 50,
      });
      console.log(`Response: ${anthropicResponse.content[0].text}\n`);
    } catch (error: any) {
      console.log(`Anthropic API not available: ${error.message}\n`);
    }

  } catch (error: any) {
    console.error('Error:', error.message);
  } finally {
    client.close();
  }
}

main().catch(console.error);
