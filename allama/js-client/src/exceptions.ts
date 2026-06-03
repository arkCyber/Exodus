/**
 * Allama client exceptions
 */

export class AllamaError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AllamaError';
  }
}

export class AllamaConnectionError extends AllamaError {
  constructor(message: string) {
    super(message);
    this.name = 'AllamaConnectionError';
  }
}

export class AllamaTimeoutError extends AllamaError {
  constructor(message: string) {
    super(message);
    this.name = 'AllamaTimeoutError';
  }
}

export class AllamaAPIError extends AllamaError {
  public statusCode?: number;
  public response?: any;

  constructor(message: string, statusCode?: number, response?: any) {
    super(message);
    this.name = 'AllamaAPIError';
    this.statusCode = statusCode;
    this.response = response;
  }
}
