/**
 * Exodus Browser — omnibox URL vs search resolution (Chrome-like).
 */

/**
 * Build a search results URL from a template containing `{query}`.
 */
export function buildSearchUrl(template: string, query: string): string {
  return template.replace('{query}', encodeURIComponent(query));
}

export type OmniboxResolution =
  | { kind: 'ask'; query: string }
  | { kind: 'navigate'; url: string };

/**
 * Decide whether input is a URL, local /ask search, or web search query.
 */
export function resolveOmniboxInput(
  raw: string,
  searchEngineUrl: string,
): OmniboxResolution | null {
  const input = raw.trim();
  if (!input) return null;

  if (input.startsWith('/ask ')) {
    return { kind: 'ask', query: input.slice(5).trim() };
  }

  if (input.startsWith('data:')) {
    return { kind: 'navigate', url: input };
  }

  if (input.startsWith('http://') || input.startsWith('https://')) {
    return { kind: 'navigate', url: input };
  }

  if (/^localhost(:\d+)?(\/|$)/i.test(input)) {
    return { kind: 'navigate', url: `http://${input}` };
  }

  if (/^\d{1,3}(\.\d{1,3}){3}(:\d+)?(\/|$)/.test(input)) {
    return { kind: 'navigate', url: `http://${input}` };
  }

  if (!input.includes(' ')) {
    if (/^[\w.-]+\.[a-z]{2,}([\/?#]|$)/i.test(input)) {
      return { kind: 'navigate', url: `https://${input}` };
    }
    if (input.includes('.') && !input.startsWith('.')) {
      try {
        const u = new URL(`https://${input}`);
        if (u.hostname.includes('.')) {
          return { kind: 'navigate', url: u.href };
        }
      } catch {
        /* fall through to search */
      }
    }
  }

  return { kind: 'navigate', url: buildSearchUrl(searchEngineUrl, input) };
}
