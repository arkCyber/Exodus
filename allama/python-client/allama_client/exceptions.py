"""
Allama client exceptions
"""


class AllamaError(Exception):
    """Base exception for Allama client errors"""
    pass


class AllamaConnectionError(AllamaError):
    """Connection error when communicating with Allama server"""
    pass


class AllamaTimeoutError(AllamaError):
    """Timeout error when request takes too long"""
    pass


class AllamaAPIError(AllamaError):
    """API error returned by Allama server"""
    
    def __init__(self, message: str, status_code: int = None, response: dict = None):
        self.message = message
        self.status_code = status_code
        self.response = response
        super().__init__(message)
