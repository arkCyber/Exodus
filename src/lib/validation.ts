/**
 * Aerospace-grade input validation layer
 * Provides strict validation and sanitization for all user inputs
 */

/**
 * Validates and sanitizes URLs
 * @param url - The URL to validate
 * @returns Sanitized URL or null if invalid
 */
export function validateUrl(url: string): string | null {
  if (!url || typeof url !== 'string') {
    return null;
  }

  // Trim whitespace
  const trimmed = url.trim();

  // Check for dangerous protocols
  const dangerousProtocols = ['javascript:', 'data:', 'vbscript:', 'file:', 'ftp:'];
  for (const protocol of dangerousProtocols) {
    if (trimmed.toLowerCase().startsWith(protocol)) {
      return null;
    }
  }

  // Check maximum length
  if (trimmed.length > 2048) {
    return null;
  }

  // Basic URL validation
  try {
    // Add protocol if missing
    let normalized = trimmed;
    if (!normalized.startsWith('http://') && !normalized.startsWith('https://')) {
      normalized = `https://${normalized}`;
    }

    const preHost = normalized.replace(/^https?:\/\//i, '').split(/[/?#]/)[0]?.split(':')[0] ?? '';
    if (/^\d+(\.\d+)*$/.test(preHost)) {
      const octets = preHost.split('.');
      if (octets.length !== 4) {
        return null;
      }
      for (const part of octets) {
        const num = parseInt(part, 10);
        if (isNaN(num) || num < 0 || num > 255) {
          return null;
        }
      }
    }

    const urlObj = new URL(normalized);

    // Validate hostname
    if (!urlObj.hostname || urlObj.hostname.length > 253) {
      return null;
    }

    // Check for invalid characters in hostname
    if (!/^[a-zA-Z0-9.-]+$/.test(urlObj.hostname)) {
      return null;
    }

    // Reject partial numeric hostnames (e.g. 1.2.3) that are not valid IPv4
    if (/^\d+(\.\d+)+$/.test(urlObj.hostname) && !/^\d+\.\d+\.\d+\.\d+$/.test(urlObj.hostname)) {
      return null;
    }

    // Check for IP address format
    if (/^\d+\.\d+\.\d+\.\d+$/.test(urlObj.hostname)) {
      const parts = urlObj.hostname.split('.');
      for (const part of parts) {
        const num = parseInt(part, 10);
        if (isNaN(num) || num < 0 || num > 255) {
          return null;
        }
      }
    }

    return normalized;
  } catch {
    return null;
  }
}

/**
 * Validates and sanitizes text input
 * @param text - The text to validate
 * @param maxLength - Maximum allowed length
 * @returns Sanitized text or null if invalid
 */
export function validateText(text: string, maxLength: number = 1000): string | null {
  if (!text || typeof text !== 'string') {
    return null;
  }

  const trimmed = text.trim();

  // Check length
  if (trimmed.length > maxLength) {
    return null;
  }

  // Check for null bytes
  if (trimmed.includes('\0')) {
    return null;
  }

  // Remove control characters except newlines and tabs
  const sanitized = trimmed.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, '');

  return sanitized;
}

/**
 * Validates and sanitizes HTML content
 * @param html - The HTML to validate
 * @returns Sanitized HTML or null if invalid
 */
export function validateHtml(html: string): string | null {
  if (!html || typeof html !== 'string') {
    return null;
  }

  // Check for dangerous patterns
  const dangerousPatterns = [
    /<script/i,
    /javascript:/i,
    /on\w+\s*=/i,
    /<iframe/i,
    /<object/i,
    /<embed/i,
    /<form/i,
  ];

  for (const pattern of dangerousPatterns) {
    if (pattern.test(html)) {
      return null;
    }
  }

  // Basic sanitization
  const sanitized = html
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;');

  return sanitized;
}

/**
 * Validates file path
 * @param path - The file path to validate
 * @returns Sanitized path or null if invalid
 */
export function validatePath(path: string): string | null {
  if (!path || typeof path !== 'string') {
    return null;
  }

  const trimmed = path.trim();

  // Check for path traversal
  if (trimmed.includes('..') || trimmed.includes('~')) {
    return null;
  }

  // Check for absolute paths (security risk)
  if (trimmed.startsWith('/') || /^[A-Za-z]:/.test(trimmed)) {
    return null;
  }

  // Check maximum length
  if (trimmed.length > 260) {
    return null;
  }

  // Check for invalid characters
  const invalidChars = /[<>:"|?*]/;
  if (invalidChars.test(trimmed)) {
    return null;
  }

  return trimmed;
}

/**
 * Validates numeric input
 * @param value - The value to validate
 * @param min - Minimum allowed value
 * @param max - Maximum allowed value
 * @returns Validated number or null if invalid
 */
export function validateNumber(value: any, min?: number, max?: number): number | null {
  if (value === null || value === undefined || value === '') {
    return null;
  }

  const num = Number(value);

  if (isNaN(num)) {
    return null;
  }

  if (min !== undefined && num < min) {
    return null;
  }

  if (max !== undefined && num > max) {
    return null;
  }

  return num;
}

/**
 * Validates email address
 * @param email - The email to validate
 * @returns Sanitized email or null if invalid
 */
export function validateEmail(email: string): string | null {
  if (!email || typeof email !== 'string') {
    return null;
  }

  const trimmed = email.trim().toLowerCase();

  // Basic email validation
  const emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
  if (!emailRegex.test(trimmed)) {
    return null;
  }

  // Check maximum length
  if (trimmed.length > 254) {
    return null;
  }

  return trimmed;
}

/**
 * Validates JSON data
 * @param json - The JSON string to validate
 * @returns Parsed object or null if invalid
 */
export function validateJson(json: string): any | null {
  if (!json || typeof json !== 'string') {
    return null;
  }

  try {
    const parsed = JSON.parse(json);
    
    // Check for circular references (basic check)
    const seen = new WeakSet();
    function checkCircular(obj: any): boolean {
      if (obj && typeof obj === 'object') {
        if (seen.has(obj)) {
          return true;
        }
        seen.add(obj);
        for (const key in obj) {
          if (checkCircular(obj[key])) {
            return true;
          }
        }
      }
      return false;
    }

    if (checkCircular(parsed)) {
      return null;
    }

    return parsed;
  } catch {
    return null;
  }
}

/**
 * Validates and sanitizes user agent string
 * @param userAgent - The user agent to validate
 * @returns Sanitized user agent or null if invalid
 */
export function validateUserAgent(userAgent: string): string | null {
  if (!userAgent || typeof userAgent !== 'string') {
    return null;
  }

  const trimmed = userAgent.trim();

  // Check maximum length
  if (trimmed.length > 500) {
    return null;
  }

  // Remove null bytes and control characters
  const sanitized = trimmed.replace(/[\x00-\x1F\x7F]/g, '');

  return sanitized;
}

/**
 * Validates and sanitizes referrer URL
 * @param referrer - The referrer to validate
 * @returns Sanitized referrer or null if invalid
 */
export function validateReferrer(referrer: string): string | null {
  if (!referrer || typeof referrer !== 'string') {
    return null;
  }

  return validateUrl(referrer);
}
