/**
 * Exodus Browser — performance / busy-cursor diagnostic logging.
 * Use to find main-thread stalls and slow Tauri IPC during startup.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';

const PREFIX = '[Exodus:perf]';
/** Warn when a span exceeds this (ms) — may correlate with macOS busy cursor. */
const CURSOR_RISK_MS = 80;
/** Warn when a span exceeds this (ms) — definite jank. */
const SLOW_MS = 200;
/** Log frame gaps larger than this (ms) between rAF ticks. */
const FRAME_GAP_MS = 100;

const spanStarts = new Map<string, number>();

/**
 * Emit a timestamped perf log line (console + optional Rust file log).
 * @param step - Short label (e.g. `webview.create`).
 * @param detail - Optional structured fields.
 */
export function logPerf(step: string, detail?: Record<string, unknown>): void {
  const ts = new Date().toISOString();
  const ms = Math.round(performance.now());
  if (detail !== undefined) {
    console.log(`${PREFIX}[${ts}][${ms}ms] ${step}`, detail);
  } else {
    console.log(`${PREFIX}[${ts}][${ms}ms] ${step}`);
  }
  /* Console only — relaying every line floods Tauri IPC and can delay invokes. */
}

/**
 * Warn-level perf log (slow path / cursor risk).
 * @param step - Short label.
 * @param detail - Optional fields (should include `durationMs` when ending a span).
 */
export function logPerfWarn(step: string, detail?: Record<string, unknown>): void {
  const ts = new Date().toISOString();
  console.warn(`${PREFIX}[${ts}] WARN ${step}`, detail ?? '');
  void relayToBackend('warn', step, detail);
}

/**
 * Start a named timing span (`perfEnd` completes it).
 * @param span - Unique span id (e.g. `BrowserPage.bgInit`).
 */
export function perfStart(span: string): void {
  spanStarts.set(span, performance.now());
  logPerf(`start:${span}`);
}

/**
 * End a span started with `perfStart`; logs duration and warns if slow.
 * @param span - Same id passed to `perfStart`.
 * @param detail - Optional extra fields merged into the log.
 */
export function perfEnd(span: string, detail?: Record<string, unknown>): void {
  const t0 = spanStarts.get(span);
  spanStarts.delete(span);
  const durationMs = t0 !== undefined ? Math.round(performance.now() - t0) : -1;
  const payload = { ...detail, durationMs };
  if (durationMs < 0) {
    logPerfWarn(`end:${span} (no start)`, payload);
    return;
  }
  if (durationMs >= SLOW_MS) {
    logPerfWarn(`SLOW end:${span}`, payload);
  } else if (durationMs >= CURSOR_RISK_MS) {
    logPerfWarn(`CURSOR_RISK end:${span}`, payload);
  } else {
    logPerf(`end:${span}`, payload);
  }
}

/**
 * Time an async function and log duration (re-throws on error).
 * @param span - Span name.
 * @param fn - Async work to measure.
 */
export async function perfAsync<T>(span: string, fn: () => Promise<T>): Promise<T> {
  perfStart(span);
  try {
    const result = await fn();
    perfEnd(span);
    return result;
  } catch (error) {
    perfEnd(span, { error: String(error) });
    throw error;
  }
}

/**
 * Time a sync function and log duration (re-throws on error).
 * @param span - Span name.
 * @param fn - Sync work to measure.
 */
export function perfSync<T>(span: string, fn: () => T): T {
  perfStart(span);
  try {
    const result = fn();
    perfEnd(span);
    return result;
  } catch (error) {
    perfEnd(span, { error: String(error) });
    throw error;
  }
}

/**
 * Watch requestAnimationFrame gaps to detect main-thread blocking (busy cursor).
 * @returns Stop function (call on unmount).
 */
export function startFrameGapMonitor(): () => void {
  let last = performance.now();
  let rafId = 0;
  let stopped = false;

  const tick = (): void => {
    if (stopped) return;
    const now = performance.now();
    const gap = now - last;
    if (gap >= FRAME_GAP_MS) {
      logPerfWarn('main_thread_frame_gap', { gapMs: Math.round(gap) });
    }
    last = now;
    rafId = requestAnimationFrame(tick);
  };

  rafId = requestAnimationFrame(tick);
  logPerf('frame_gap_monitor_started', { thresholdMs: FRAME_GAP_MS });

  return () => {
    stopped = true;
    cancelAnimationFrame(rafId);
    logPerf('frame_gap_monitor_stopped');
  };
}

/**
 * Relay a client log line to Rust `startup.log` when Tauri IPC is ready.
 */
async function relayToBackend(
  level: string,
  step: string,
  detail?: Record<string, unknown>,
): Promise<void> {
  if (!isTauri()) return;
  try {
    const detailStr = detail ? JSON.stringify(detail) : null;
    await invoke('perf_log_client', { level, step, detail: detailStr });
  } catch {
    /* IPC may not be ready yet during very early boot */
  }
}

/**
 * Log a one-line startup diagnosis hint for the busy-cursor investigation.
 */
export function logPerfDiagnosis(hint: string, detail?: Record<string, unknown>): void {
  logPerfWarn(`DIAGNOSIS: ${hint}`, detail);
}

void relayToBackend('info', 'perfLog module loaded');
