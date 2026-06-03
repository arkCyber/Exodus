import { describe, it, expect, vi } from 'vitest';
import { useMicroservice, useServoStatus } from './useMicroservice';

// Mock Tauri API
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('lodash-es', () => ({
  throttle: vi.fn((fn) => fn),
}));

describe('useMicroservice', () => {
  it('provides data state', () => {
    const { data } = useMicroservice({ name: 'test-service' });
    expect(data.value).toBe(null);
  });

  it('provides loading state', () => {
    const { loading } = useMicroservice({ name: 'test-service' });
    expect(loading.value).toBe(false);
  });

  it('provides error state', () => {
    const { error } = useMicroservice({ name: 'test-service' });
    expect(error.value).toBe(null);
  });

  it('provides listenThrottled function', () => {
    const { listenThrottled } = useMicroservice({ name: 'test-service' });
    expect(typeof listenThrottled).toBe('function');
  });

  it('provides callMethod function', () => {
    const { callMethod } = useMicroservice({ name: 'test-service' });
    expect(typeof callMethod).toBe('function');
  });
});

describe('useServoStatus', () => {
  it('provides loadingProgress state', () => {
    const { loadingProgress } = useServoStatus();
    expect(loadingProgress.value).toBe(0);
  });

  it('provides rendering state', () => {
    const { rendering } = useServoStatus();
    expect(rendering.value).toBe(false);
  });

  it('allows updating loadingProgress', () => {
    const { loadingProgress } = useServoStatus();
    loadingProgress.value = 50;
    expect(loadingProgress.value).toBe(50);
  });

  it('allows updating rendering state', () => {
    const { rendering } = useServoStatus();
    rendering.value = true;
    expect(rendering.value).toBe(true);
  });
});
