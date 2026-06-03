/**
 * Vitest global setup — jsdom polyfills and shared browser API stubs.
 */

import { vi } from 'vitest';

/** Blob URLs for download helpers in jsdom. */
if (typeof URL.createObjectURL !== 'function') {
  URL.createObjectURL = () => 'blob:mock';
  URL.revokeObjectURL = () => {};
}

/** matchMedia for theme and responsive composables in jsdom. */
if (typeof window.matchMedia !== 'function') {
  window.matchMedia = (query: string) => ({
    matches: query.includes('prefers-color-scheme: dark'),
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  });
}

/** Chrome-style heap stats for memory-monitor tests in jsdom. */
if (!(performance as Performance & { memory?: unknown }).memory) {
  Object.defineProperty(performance, 'memory', {
    configurable: true,
    value: {
      usedJSHeapSize: 12_000_000,
      totalJSHeapSize: 24_000_000,
      jsHeapSizeLimit: 2_147_483_648,
    },
  });
}

/** Reduce timer noise from singleton monitors during unit tests. */
vi.stubGlobal(
  'requestAnimationFrame',
  (cb: FrameRequestCallback) => setTimeout(() => cb(performance.now()), 0) as unknown as number,
);
