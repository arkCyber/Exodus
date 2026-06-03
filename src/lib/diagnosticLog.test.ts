/**
 * Unit tests for diagnosticLog helpers.
 */
import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { createDiagnosticLogger } from './diagnosticLog';

vi.mock('./startupLog', () => ({
  logStartup: vi.fn(),
  logStartupWarn: vi.fn(),
  logStartupError: vi.fn(),
  logStartupLocal: vi.fn(),
}));

describe('diagnosticLog', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('createDiagnosticLogger prefixes messages', async () => {
    const { logStartup } = await import('./startupLog');
    const log = createDiagnosticLogger('Test');
    log.info('hello', { ok: true });
    expect(logStartup).toHaveBeenCalledWith('[Test] hello', { ok: true });
  });

  it('timeEnd relays slow spans to startup.log', async () => {
    const { logStartup } = await import('./startupLog');
    const log = createDiagnosticLogger('Timing');
    log.timeStart('slow');
    await new Promise((r) => setTimeout(r, 160));
    log.timeEnd('slow');
    expect(logStartup).toHaveBeenCalledWith('[Timing] end:slow', expect.objectContaining({ durationMs: expect.any(Number) }));
  });

  it('timeEnd skips file relay for fast spans', async () => {
    const { logStartup } = await import('./startupLog');
    vi.mocked(logStartup).mockClear();
    const log = createDiagnosticLogger('Fast');
    log.timeStart('quick');
    log.timeEnd('quick');
    expect(logStartup).not.toHaveBeenCalled();
  });
});
