import { describe, it, expect } from 'vitest';
import {
  validateUrl,
  validateText,
  validateHtml,
  validatePath,
  validateNumber,
  validateEmail,
  validateJson,
  validateUserAgent,
  validateReferrer,
} from './validation';

describe('Validation - Boundary Conditions', () => {
  describe('validateUrl', () => {
    it('should reject null and undefined', () => {
      expect(validateUrl(null as any)).toBeNull();
      expect(validateUrl(undefined as any)).toBeNull();
    });

    it('should reject non-string values', () => {
      expect(validateUrl(123 as any)).toBeNull();
      expect(validateUrl({} as any)).toBeNull();
      expect(validateUrl([] as any)).toBeNull();
    });

    it('should reject dangerous protocols', () => {
      expect(validateUrl('javascript:alert(1)')).toBeNull();
      expect(validateUrl('data:text/html,<script>alert(1)</script>')).toBeNull();
      expect(validateUrl('vbscript:msgbox(1)')).toBeNull();
      expect(validateUrl('file:///etc/passwd')).toBeNull();
      expect(validateUrl('ftp://example.com')).toBeNull();
    });

    it('should reject URLs exceeding maximum length', () => {
      const longUrl = 'https://example.com/' + 'a'.repeat(2048);
      expect(validateUrl(longUrl)).toBeNull();
    });

    it('should reject invalid hostnames', () => {
      expect(validateUrl('https://' + 'a'.repeat(254) + '.com')).toBeNull();
      expect(validateUrl('https://exa$mple.com')).toBeNull();
    });

    it('should reject invalid IP addresses', () => {
      expect(validateUrl('https://999.999.999.999')).toBeNull();
      expect(validateUrl('https://1.2.3')).toBeNull();
      expect(validateUrl('https://1.2.3.4.5')).toBeNull();
    });

    it('should accept valid URLs', () => {
      expect(validateUrl('https://example.com')).toBe('https://example.com');
      expect(validateUrl('http://example.com')).toBe('http://example.com');
      expect(validateUrl('example.com')).toBe('https://example.com');
      expect(validateUrl('https://192.168.1.1')).toBe('https://192.168.1.1');
    });

    it('should handle whitespace', () => {
      expect(validateUrl('  https://example.com  ')).toBe('https://example.com');
    });
  });

  describe('validateText', () => {
    it('should reject null and undefined', () => {
      expect(validateText(null as any)).toBeNull();
      expect(validateText(undefined as any)).toBeNull();
    });

    it('should reject non-string values', () => {
      expect(validateText(123 as any)).toBeNull();
      expect(validateText({} as any)).toBeNull();
    });

    it('should reject text exceeding maximum length', () => {
      const longText = 'a'.repeat(1001);
      expect(validateText(longText)).toBeNull();
    });

    it('should reject text with null bytes', () => {
      expect(validateText('hello\x00world')).toBeNull();
    });

    it('should remove control characters', () => {
      const result = validateText('hello\x01world');
      expect(result).toBe('helloworld');
    });

    it('should preserve newlines and tabs', () => {
      const result = validateText('hello\nworld\ttest');
      expect(result).toBe('hello\nworld\ttest');
    });

    it('should accept valid text', () => {
      expect(validateText('hello world')).toBe('hello world');
      expect(validateText('')).toBeNull();
    });
  });

  describe('validateHtml', () => {
    it('should reject null and undefined', () => {
      expect(validateHtml(null as any)).toBeNull();
      expect(validateHtml(undefined as any)).toBeNull();
    });

    it('should reject dangerous HTML patterns', () => {
      expect(validateHtml('<script>alert(1)</script>')).toBeNull();
      expect(validateHtml('<img src=x onerror=alert(1)>')).toBeNull();
      expect(validateHtml('<iframe src="evil.html"></iframe>')).toBeNull();
      expect(validateHtml('<object data="evil.swf"></object>')).toBeNull();
      expect(validateHtml('<embed src="evil.swf">')).toBeNull();
      expect(validateHtml('<form action="evil.com">')).toBeNull();
    });

    it('should sanitize HTML entities', () => {
      const result = validateHtml('<p>hello</p>');
      expect(result).toBe('&lt;p&gt;hello&lt;/p&gt;');
    });

    it('should sanitize quotes', () => {
      const result = validateHtml('"hello"');
      expect(result).toBe('&quot;hello&quot;');
    });

    it('should sanitize apostrophes', () => {
      const result = validateHtml("'hello'");
      expect(result).toBe('&#x27;hello&#x27;');
    });
  });

  describe('validatePath', () => {
    it('should reject null and undefined', () => {
      expect(validatePath(null as any)).toBeNull();
      expect(validatePath(undefined as any)).toBeNull();
    });

    it('should reject path traversal attempts', () => {
      expect(validatePath('../etc/passwd')).toBeNull();
      expect(validatePath('..\\windows\\system32')).toBeNull();
      expect(validatePath('~/../../etc/passwd')).toBeNull();
    });

    it('should reject absolute paths', () => {
      expect(validatePath('/etc/passwd')).toBeNull();
      expect(validatePath('C:\\Windows\\System32')).toBeNull();
    });

    it('should reject paths exceeding maximum length', () => {
      const longPath = 'a'.repeat(261);
      expect(validatePath(longPath)).toBeNull();
    });

    it('should reject paths with invalid characters', () => {
      expect(validatePath('file<name>')).toBeNull();
      expect(validatePath('file:name')).toBeNull();
      expect(validatePath('file"name')).toBeNull();
      expect(validatePath('file|name')).toBeNull();
      expect(validatePath('file?name')).toBeNull();
      expect(validatePath('file*name')).toBeNull();
    });

    it('should accept valid relative paths', () => {
      expect(validatePath('folder/file.txt')).toBe('folder/file.txt');
      expect(validatePath('file.txt')).toBe('file.txt');
    });
  });

  describe('validateNumber', () => {
    it('should reject non-numeric values', () => {
      expect(validateNumber('abc')).toBeNull();
      expect(validateNumber({})).toBeNull();
      expect(validateNumber(null)).toBeNull();
      expect(validateNumber(undefined)).toBeNull();
    });

    it('should enforce minimum value', () => {
      expect(validateNumber(5, 10)).toBeNull();
      expect(validateNumber(10, 10)).toBe(10);
      expect(validateNumber(15, 10)).toBe(15);
    });

    it('should enforce maximum value', () => {
      expect(validateNumber(15, undefined, 10)).toBeNull();
      expect(validateNumber(10, undefined, 10)).toBe(10);
      expect(validateNumber(5, undefined, 10)).toBe(5);
    });

    it('should accept valid numbers', () => {
      expect(validateNumber(5)).toBe(5);
      expect(validateNumber(0)).toBe(0);
      expect(validateNumber(-5)).toBe(-5);
      expect(validateNumber(3.14)).toBe(3.14);
    });

    it('should handle string numbers', () => {
      expect(validateNumber('5')).toBe(5);
      expect(validateNumber('3.14')).toBe(3.14);
    });
  });

  describe('validateEmail', () => {
    it('should reject null and undefined', () => {
      expect(validateEmail(null as any)).toBeNull();
      expect(validateEmail(undefined as any)).toBeNull();
    });

    it('should reject invalid email formats', () => {
      expect(validateEmail('invalid')).toBeNull();
      expect(validateEmail('invalid@')).toBeNull();
      expect(validateEmail('@example.com')).toBeNull();
      expect(validateEmail('user@')).toBeNull();
    });

    it('should reject emails exceeding maximum length', () => {
      const longEmail = 'a'.repeat(255) + '@example.com';
      expect(validateEmail(longEmail)).toBeNull();
    });

    it('should accept valid emails', () => {
      expect(validateEmail('user@example.com')).toBe('user@example.com');
      expect(validateEmail('USER@EXAMPLE.COM')).toBe('user@example.com');
      expect(validateEmail('user.name@example.com')).toBe('user.name@example.com');
      expect(validateEmail('user+tag@example.com')).toBe('user+tag@example.com');
    });

    it('should handle whitespace', () => {
      expect(validateEmail('  user@example.com  ')).toBe('user@example.com');
    });
  });

  describe('validateJson', () => {
    it('should reject null and undefined', () => {
      expect(validateJson(null as any)).toBeNull();
      expect(validateJson(undefined as any)).toBeNull();
    });

    it('should reject invalid JSON', () => {
      expect(validateJson('not json')).toBeNull();
      expect(validateJson('{invalid}')).toBeNull();
    });

    it('should reject malformed JSON strings', () => {
      expect(validateJson('{not valid json')).toBeNull();
    });

    it('should accept valid JSON', () => {
      expect(validateJson('{"key":"value"}')).toEqual({ key: 'value' });
      expect(validateJson('[]')).toEqual([]);
      expect(validateJson('123')).toBe(123);
      expect(validateJson('"string"')).toBe('string');
    });
  });

  describe('validateUserAgent', () => {
    it('should reject null and undefined', () => {
      expect(validateUserAgent(null as any)).toBeNull();
      expect(validateUserAgent(undefined as any)).toBeNull();
    });

    it('should reject user agents exceeding maximum length', () => {
      const longUA = 'Mozilla/5.0 ' + 'a'.repeat(500);
      expect(validateUserAgent(longUA)).toBeNull();
    });

    it('should remove control characters', () => {
      const result = validateUserAgent('Mozilla\x00/5.0');
      expect(result).toBe('Mozilla/5.0');
    });

    it('should accept valid user agents', () => {
      expect(validateUserAgent('Mozilla/5.0')).toBe('Mozilla/5.0');
      expect(validateUserAgent('Chrome/90.0')).toBe('Chrome/90.0');
    });
  });

  describe('validateReferrer', () => {
    it('should delegate to validateUrl', () => {
      expect(validateReferrer('https://example.com')).toBe('https://example.com');
      expect(validateReferrer('javascript:alert(1)')).toBeNull();
    });
  });
});
