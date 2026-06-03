/**
 * Exodus Browser — component-scoped diagnostic logging (console + startup.log + timing).
 *
 * `timeStart`/`timeEnd` are console-only unless a span exceeds `SLOW_SPAN_MS` (file relay).
 */

import { logStartup, logStartupWarn, logStartupError, logStartupLocal } from '@/lib/startupLog';

/** Relay span end to startup.log when slower than this (ms). */
const SLOW_SPAN_MS = 150;

/** Component logger with optional timing spans. */
export type DiagnosticLogger = {
  info: (step: string, detail?: unknown) => void;
  warn: (step: string, detail?: unknown) => void;
  error: (step: string, err: unknown) => void;
  timeStart: (span: string, detail?: unknown) => void;
  timeEnd: (span: string, detail?: unknown) => void;
};

/**
 * Create a logger prefixed with `[component]` for grep-friendly startup.log lines.
 * @param component - Short area name, e.g. `NTP`, `Shell`, `Extensions`.
 */
export function createDiagnosticLogger(component: string): DiagnosticLogger {
  const tag = `[${component}]`;
  const spans = new Map<string, number>();

  return {
    info(step: string, detail?: unknown): void {
      logStartup(`${tag} ${step}`, detail);
    },
    warn(step: string, detail?: unknown): void {
      logStartupWarn(`${tag} ${step}`, detail);
    },
    error(step: string, err: unknown): void {
      logStartupError(`${tag} ${step}`, err);
    },
    timeStart(span: string, detail?: unknown): void {
      spans.set(span, performance.now());
      logStartupLocal(`${tag} start:${span}`, detail);
    },
    timeEnd(span: string, detail?: unknown): void {
      const t0 = spans.get(span);
      spans.delete(span);
      const durationMs = t0 !== undefined ? Math.round(performance.now() - t0) : -1;
      const payload =
        detail !== undefined && typeof detail === 'object' && detail !== null
          ? { ...(detail as Record<string, unknown>), durationMs }
          : detail !== undefined
            ? { detail, durationMs }
            : { durationMs };
      logStartupLocal(`${tag} end:${span}`, payload);
      if (durationMs < 0 || durationMs >= SLOW_SPAN_MS) {
        logStartup(`${tag} end:${span}`, payload);
      }
    },
  };
}

/** New tab page + wallpaper pipeline. */
export const ntpLog = createDiagnosticLogger('NTP');

/** Browser shell (BrowserPage, routing, chrome). */
export const shellLog = createDiagnosticLogger('Shell');

/** Web Extension host integration. */
export const extLog = createDiagnosticLogger('Extensions');

logStartup('diagnosticLog module loaded');
