/**
 * Exodus Browser — content script match pattern helpers (mirrors Rust MVP rules).
 */

/**
 * Returns true when a URL matches a Chrome-style pattern (client-side validation helper).
 */
export function urlMatchesPattern(url: string, pattern: string): boolean {
  if (pattern === '<all_urls>') {
    try {
      const u = new URL(url);
      return u.protocol === 'http:' || u.protocol === 'https:';
    } catch {
      return false;
    }
  }
  try {
    const u = new URL(url);
    const [schemePart, rest] = pattern.split('://');
    if (!rest) return false;
    if (!schemeMatches(u.protocol.replace(':', ''), schemePart)) return false;
    const slash = rest.indexOf('/');
    const hostPattern = slash >= 0 ? rest.slice(0, slash) : rest;
    const pathPattern = slash >= 0 ? rest.slice(slash) : '/';
    const host = u.hostname;
    if (!hostMatches(host, hostPattern)) return false;
    return pathMatches(u.pathname, pathPattern);
  } catch {
    return false;
  }
}

function schemeMatches(scheme: string, pattern: string): boolean {
  if (pattern === '*') return scheme === 'http' || scheme === 'https';
  return scheme === pattern;
}

function hostMatches(host: string, pattern: string): boolean {
  if (pattern === '*') return host.length > 0;
  if (pattern.startsWith('*.')) {
    const suffix = pattern.slice(2);
    return host === suffix || host.endsWith(`.${suffix}`);
  }
  return host === pattern;
}

function pathMatches(path: string, pattern: string): boolean {
  if (pattern === '/*' || pattern === '*') return true;
  if (pattern.endsWith('*')) {
    const prefix = pattern.slice(0, -1).replace(/\/$/, '');
    return path.startsWith(prefix) || path.startsWith(`${prefix}/`);
  }
  return path === pattern;
}
