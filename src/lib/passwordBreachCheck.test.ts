/**
 * Exodus Browser — Password Breach Check Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as passwordBreachCheck from './passwordBreachCheck';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('Password Breach Check', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should check if password is compromised', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('compromised');
    
    const result = await passwordBreachCheck.checkPasswordCompromised('password123');
    expect(result).toBe('compromised');
    expect(invoke).toHaveBeenCalledWith('check_password_compromised', { password: 'password123' });
  });

  it('should return safe for non-compromised password', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('safe');
    
    const result = await passwordBreachCheck.checkPasswordCompromised('StrongP@ssw0rd!');
    expect(result).toBe('safe');
  });

  it('should return unknown on network error', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('unknown');
    
    const result = await passwordBreachCheck.checkPasswordCompromised('test');
    expect(result).toBe('unknown');
  });

  it('should check password strength', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('strong');
    
    const result = await passwordBreachCheck.checkPasswordStrength('MyStr0ngP@ss!');
    expect(result).toBe('strong');
    expect(invoke).toHaveBeenCalledWith('check_password_strength', { password: 'MyStr0ngP@ss!' });
  });

  it('should generate password', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('xK9#mP2$vL8@nQ5');
    
    const result = await passwordBreachCheck.generatePassword(16);
    expect(result).toBe('xK9#mP2$vL8@nQ5');
    expect(invoke).toHaveBeenCalledWith('generate_password', { length: 16 });
  });

  it('should generate password with default length', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('AbC123!@#');
    
    const result = await passwordBreachCheck.generatePassword();
    expect(result).toBe('AbC123!@#');
    expect(invoke).toHaveBeenCalledWith('generate_password', { length: 16 });
  });
});
