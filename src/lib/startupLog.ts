/**
 * Exodus Browser — timestamped frontend startup logging (DevTools console).
 */

const PREFIX = '[Exodus]';

/** Log a startup milestone with ISO timestamp. */
export function logStartup(step: string, detail?: unknown): void {
  const ts = new Date().toISOString();
  if (detail !== undefined) {
    console.log(`${PREFIX}[${ts}] ${step}`, detail);
  } else {
    console.log(`${PREFIX}[${ts}] ${step}`);
  }
}

/** Log a startup error. */
export function logStartupError(step: string, error: unknown): void {
  const ts = new Date().toISOString();
  console.error(`${PREFIX}[${ts}] ${step}`, error);
}

logStartup('startupLog module loaded');
