"""
Allama Python Client - Official Python client for Allama LLM inference server
"""

from .client import AllamaClient
from .exceptions import (
    AllamaError,
    AllamaConnectionError,
    AllamaTimeoutError,
    AllamaAPIError,
)

__version__ = "0.1.0"
__all__ = [
    "AllamaClient",
    "AllamaError",
    "AllamaConnectionError",
    "AllamaTimeoutError",
    "AllamaAPIError",
]
