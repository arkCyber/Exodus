"""
Allama client implementation
"""

import json
import time
from typing import Dict, List, Optional, Union, Iterator, Any
import requests

from .exceptions import (
    AllamaError,
    AllamaConnectionError,
    AllamaTimeoutError,
    AllamaAPIError,
)


class AllamaClient:
    """
    Official Python client for Allama LLM inference server
    """
    
    def __init__(
        self,
        base_url: str = "http://127.0.0.1:11435",
        api_key: Optional[str] = None,
        timeout: int = 60,
        max_retries: int = 3,
    ):
        """
        Initialize Allama client
        
        Args:
            base_url: Base URL of Allama server (default: http://127.0.0.1:11435)
            api_key: Optional API key for authentication
            timeout: Request timeout in seconds (default: 60)
            max_retries: Maximum number of retry attempts (default: 3)
        """
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout
        self.max_retries = max_retries
        self.session = requests.Session()
        
        if api_key:
            self.session.headers.update({"Authorization": f"Bearer {api_key}"})
    
    def _request(
        self,
        method: str,
        endpoint: str,
        data: Optional[Dict] = None,
        params: Optional[Dict] = None,
        stream: bool = False,
    ) -> Union[Dict, Iterator[Dict]]:
        """
        Make HTTP request to Allama server with retry logic
        
        Args:
            method: HTTP method (GET, POST, etc.)
            endpoint: API endpoint
            data: Request body data
            params: Query parameters
            stream: Whether to stream response
            
        Returns:
            Response data or iterator for streaming responses
        """
        url = f"{self.base_url}{endpoint}"
        headers = {"Content-Type": "application/json"}
        
        last_error = None
        for attempt in range(self.max_retries):
            try:
                response = self.session.request(
                    method=method,
                    url=url,
                    headers=headers,
                    json=data,
                    params=params,
                    timeout=self.timeout,
                    stream=stream,
                )
                
                if response.status_code >= 400:
                    error_data = None
                    try:
                        error_data = response.json()
                    except:
                        error_data = {"raw": response.text}
                    
                    raise AllamaAPIError(
                        f"API error: {response.status_code}",
                        status_code=response.status_code,
                        response=error_data,
                    )
                
                if stream:
                    return self._stream_response(response)
                else:
                    return response.json()
                    
            except requests.Timeout as e:
                last_error = AllamaTimeoutError(f"Request timeout: {e}")
                if attempt < self.max_retries - 1:
                    time.sleep(2 ** attempt)  # Exponential backoff
                    continue
                raise last_error
                
            except requests.ConnectionError as e:
                last_error = AllamaConnectionError(f"Connection error: {e}")
                if attempt < self.max_retries - 1:
                    time.sleep(2 ** attempt)
                    continue
                raise last_error
                
            except AllamaAPIError:
                raise  # Don't retry API errors
                
            except Exception as e:
                last_error = AllamaError(f"Unexpected error: {e}")
                if attempt < self.max_retries - 1:
                    time.sleep(2 ** attempt)
                    continue
                raise last_error
        
        raise last_error
    
    def _stream_response(self, response: requests.Response) -> Iterator[Dict]:
        """
        Stream SSE response from server
        
        Args:
            response: HTTP response object
            
        Yields:
            Parsed JSON chunks
        """
        for line in response.iter_lines():
            if line:
                line = line.decode('utf-8')
                if line.startswith('data: '):
                    data = line[6:]  # Remove 'data: ' prefix
                    if data == '[DONE]':
                        break
                    try:
                        yield json.loads(data)
                    except json.JSONDecodeError:
                        continue
    
    def chat(
        self,
        model: str,
        messages: List[Dict[str, str]],
        stream: bool = False,
        temperature: float = 0.7,
        max_tokens: int = 512,
        top_p: float = 0.9,
        **kwargs
    ) -> Union[Dict, Iterator[Dict]]:
        """
        Chat completion (OpenAI-compatible)
        
        Args:
            model: Model name
            messages: List of message dicts with 'role' and 'content'
            stream: Whether to stream response
            temperature: Sampling temperature (0.0-1.0)
            max_tokens: Maximum tokens to generate
            top_p: Nucleus sampling parameter
            **kwargs: Additional parameters
            
        Returns:
            Response dict or iterator for streaming
        """
        data = {
            "model": model,
            "messages": messages,
            "stream": stream,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "top_p": top_p,
            **kwargs
        }
        
        return self._request("POST", "/v1/chat/completions", data=data, stream=stream)
    
    def chat_stream(self, model: str, messages: List[Dict[str, str]], **kwargs) -> Iterator[Dict]:
        """
        Streaming chat completion (convenience method)
        
        Args:
            model: Model name
            messages: List of message dicts
            **kwargs: Additional parameters
            
        Yields:
            Response chunks
        """
        return self.chat(model, messages, stream=True, **kwargs)
    
    def generate(
        self,
        model: str,
        prompt: str,
        stream: bool = False,
        temperature: float = 0.7,
        max_tokens: int = 512,
        **kwargs
    ) -> Union[Dict, Iterator[Dict]]:
        """
        Text generation (Ollama-compatible)
        
        Args:
            model: Model name
            prompt: Input prompt
            stream: Whether to stream response
            temperature: Sampling temperature
            max_tokens: Maximum tokens to generate
            **kwargs: Additional parameters
            
        Returns:
            Response dict or iterator for streaming
        """
        data = {
            "model": model,
            "prompt": prompt,
            "stream": stream,
            "options": {
                "temperature": temperature,
                "num_predict": max_tokens,
                **kwargs
            }
        }
        
        return self._request("POST", "/api/generate", data=data, stream=stream)
    
    def generate_stream(self, model: str, prompt: str, **kwargs) -> Iterator[Dict]:
        """
        Streaming text generation (convenience method)
        
        Args:
            model: Model name
            prompt: Input prompt
            **kwargs: Additional parameters
            
        Yields:
            Response chunks
        """
        return self.generate(model, prompt, stream=True, **kwargs)
    
    def list_models(self) -> Dict:
        """
        List available models
        
        Returns:
            Dict with model information
        """
        return self._request("GET", "/api/tags")
    
    def embeddings(
        self,
        model: str,
        input: Union[str, List[str]],
        **kwargs
    ) -> Dict:
        """
        Get embeddings (OpenAI-compatible)
        
        Args:
            model: Model name
            input: Input text or list of texts
            **kwargs: Additional parameters
            
        Returns:
            Embedding response
        """
        data = {
            "model": model,
            "input": input,
            **kwargs
        }
        
        return self._request("POST", "/v1/embeddings", data=data)
    
    def anthropic_messages(
        self,
        model: str,
        messages: List[Dict[str, str]],
        max_tokens: int = 512,
        stream: bool = False,
        **kwargs
    ) -> Union[Dict, Iterator[Dict]]:
        """
        Anthropic API compatibility (/v1/messages)
        
        Args:
            model: Model name
            messages: List of message dicts
            max_tokens: Maximum tokens to generate
            stream: Whether to stream response
            **kwargs: Additional parameters
            
        Returns:
            Response dict or iterator for streaming
        """
        data = {
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "stream": stream,
            **kwargs
        }
        
        return self._request("POST", "/v1/messages", data=data, stream=stream)
    
    def close(self):
        """Close the session"""
        self.session.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
