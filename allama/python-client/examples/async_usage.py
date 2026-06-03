"""
Async usage examples using asyncio
"""

import asyncio
import aiohttp
from allama_client import AllamaClient


async def async_chat_example():
    """Example of using the client with asyncio"""
    
    # Note: The current client is synchronous, but you can use it with asyncio
    # by running in an executor or using aiohttp directly
    
    client = AllamaClient(base_url="http://127.0.0.1:11435")
    
    # Run synchronous client in thread pool
    loop = asyncio.get_event_loop()
    response = await loop.run_in_executor(
        None,
        lambda: client.chat(
            model="gemma4-4b",
            messages=[{"role": "user", "content": "Hello from async!"}]
        )
    )
    
    print(f"Async response: {response['message']['content']}")
    client.close()


if __name__ == "__main__":
    asyncio.run(async_chat_example())
