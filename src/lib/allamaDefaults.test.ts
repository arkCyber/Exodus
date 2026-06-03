/**
 * Exodus Browser — Allama / AI default settings tests.
 */
import { describe, expect, it } from 'vitest';
import type { ExodusConfigDto } from './browserSettings';

const ALLAMA_DEFAULT_PORT = 11435;

describe('Allama defaults', () => {
  it('uses port 11435 and spawn_allama in config DTO', () => {
    const cfg: ExodusConfigDto = {
      ai_port: ALLAMA_DEFAULT_PORT,
      ai_model: 'exodus-default',
      embedding_model: 'nomic-embed-text',
      homepage_url: 'https://example.com',
      search_engine_url: 'https://duckduckgo.com/?q={query}',
      status_clear_ms: 5000,
      spawn_sidecar: false,
      spawn_allama: true,
    };
    expect(cfg.ai_port).toBe(11435);
    expect(cfg.spawn_allama).toBe(true);
    expect(cfg.spawn_sidecar).toBe(false);
  });

  it('Ollama legacy port differs from Allama default', () => {
    const OLLAMA_LEGACY_PORT = 11434;
    expect(ALLAMA_DEFAULT_PORT).not.toBe(OLLAMA_LEGACY_PORT);
  });
});
