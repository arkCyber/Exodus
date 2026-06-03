"""
Tests for Allama Python client
"""

import pytest
from unittest.mock import Mock, patch, MagicMock
from allama_client import AllamaClient
from allama_client.exceptions import AllamaError, AllamaConnectionError, AllamaTimeoutError, AllamaAPIError


@pytest.fixture
def client():
    """Create a test client"""
    return AllamaClient(base_url="http://127.0.0.1:11435")


def test_client_initialization():
    """Test client initialization"""
    client = AllamaClient(
        base_url="http://127.0.0.1:11435",
        api_key="test-key",
        timeout=30,
        max_retries=2
    )
    
    assert client.base_url == "http://127.0.0.1:11435"
    assert client.api_key == "test-key"
    assert client.timeout == 30
    assert client.max_retries == 2
    assert "Authorization" in client.session.headers


def test_base_url_normalization():
    """Test base URL normalization"""
    client1 = AllamaClient(base_url="http://127.0.0.1:11435/")
    assert client1.base_url == "http://127.0.0.1:11435"
    
    client2 = AllamaClient(base_url="http://127.0.0.1:11435//")
    assert client2.base_url == "http://127.0.0.1:11435"


@patch('allama_client.client.requests.Session.request')
def test_chat_request(mock_request):
    """Test chat request"""
    mock_response = Mock()
    mock_response.status_code = 200
    mock_response.json.return_value = {
        "message": {"content": "Hello!"}
    }
    mock_request.return_value = mock_response
    
    client = AllamaClient()
    response = client.chat(
        model="test-model",
        messages=[{"role": "user", "content": "Hi"}]
    )
    
    assert response["message"]["content"] == "Hello!"
    mock_request.assert_called_once()


@patch('allama_client.client.requests.Session.request')
def test_generate_request(mock_request):
    """Test generate request"""
    mock_response = Mock()
    mock_response.status_code = 200
    mock_response.json.return_value = {
        "response": "Generated text"
    }
    mock_request.return_value = mock_response
    
    client = AllamaClient()
    response = client.generate(
        model="test-model",
        prompt="Test prompt"
    )
    
    assert response["response"] == "Generated text"


@patch('allama_client.client.requests.Session.request')
def test_list_models(mock_request):
    """Test list models"""
    mock_response = Mock()
    mock_response.status_code = 200
    mock_response.json.return_value = {
        "models": [{"name": "model1"}, {"name": "model2"}]
    }
    mock_request.return_value = mock_response
    
    client = AllamaClient()
    models = client.list_models()
    
    assert len(models["models"]) == 2


@patch('allama_client.client.requests.Session.request')
def test_api_error(mock_request):
    """Test API error handling"""
    mock_response = Mock()
    mock_response.status_code = 400
    mock_response.json.return_value = {"error": "Bad request"}
    mock_request.return_value = mock_response
    
    client = AllamaClient()
    
    with pytest.raises(AllamaAPIError) as exc_info:
        client.chat(model="test", messages=[{"role": "user", "content": "Hi"}])
    
    assert exc_info.value.status_code == 400


@patch('allama_client.client.requests.Session.request')
def test_timeout_error(mock_request):
    """Test timeout error handling"""
    import requests
    mock_request.side_effect = requests.Timeout("Request timeout")
    
    client = AllamaClient(max_retries=1)
    
    with pytest.raises(AllamaTimeoutError):
        client.chat(model="test", messages=[{"role": "user", "content": "Hi"}])


@patch('allama_client.client.requests.Session.request')
def test_connection_error(mock_request):
    """Test connection error handling"""
    import requests
    mock_request.side_effect = requests.ConnectionError("Connection failed")
    
    client = AllamaClient(max_retries=1)
    
    with pytest.raises(AllamaConnectionError):
        client.chat(model="test", messages=[{"role": "user", "content": "Hi"}])


def test_context_manager():
    """Test client as context manager"""
    with AllamaClient() as client:
        assert client is not None
    # Session should be closed after exiting context


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
