import { describe, it, expect, vi } from 'vitest';
import { useTauri } from './useTauri';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(),
}));

describe('useTauri', () => {
  it('provides loading state', () => {
    const { loading } = useTauri();
    expect(loading.value).toBe(false);
  });

  it('provides error state', () => {
    const { error } = useTauri();
    expect(error.value).toBe(null);
  });

  it('provides invokeCommand function', () => {
    const { invokeCommand } = useTauri();
    expect(typeof invokeCommand).toBe('function');
  });

  it('sets loading to true during command execution', async () => {
    const { loading, invokeCommand } = useTauri();
    
    const promise = invokeCommand('test_command');
    expect(loading.value).toBe(true);
    
    await promise;
    expect(loading.value).toBe(false);
  });
});
