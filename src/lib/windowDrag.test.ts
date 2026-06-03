/**
 * Exodus Browser — window drag helper tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

const startDragging = vi.fn(async () => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ startDragging }),
}));

import { isWindowDragBlockedTarget, startWindowDragFromMouseDown } from './windowDrag';

describe('windowDrag', () => {
  beforeEach(() => {
    startDragging.mockClear();
    document.body.innerHTML = '';
  });

  it('blocks drag targets inside buttons', () => {
    document.body.innerHTML = '<button id="b"><span id="s">x</span></button>';
    const span = document.getElementById('s')!;
    expect(isWindowDragBlockedTarget(span)).toBe(true);
  });

  it('blocks drag targets inside inputs', () => {
    document.body.innerHTML = '<input id="i" />';
    expect(isWindowDragBlockedTarget(document.getElementById('i'))).toBe(true);
  });

  it('allows drag on plain toolbar background', () => {
    document.body.innerHTML = '<div id="bg" class="address-bar"></div>';
    const bg = document.getElementById('bg')!;
    expect(isWindowDragBlockedTarget(bg)).toBe(false);
  });

  it('startWindowDragFromMouseDown calls startDragging on blank mousedown', () => {
    document.body.innerHTML = '<div id="bg" class="address-bar"></div>';
    const bg = document.getElementById('bg')!;
    startWindowDragFromMouseDown({ button: 0, target: bg, defaultPrevented: false } as unknown as MouseEvent);
    expect(startDragging).toHaveBeenCalledTimes(1);
  });

  it('startWindowDragFromMouseDown skips buttons', () => {
    document.body.innerHTML = '<button id="b">Go</button>';
    const btn = document.getElementById('b')!;
    startWindowDragFromMouseDown({ button: 0, target: btn, defaultPrevented: false } as unknown as MouseEvent);
    expect(startDragging).not.toHaveBeenCalled();
  });

  it('startWindowDragFromMouseDown ignores non-primary button', () => {
    document.body.innerHTML = '<div id="bg"></div>';
    const bg = document.getElementById('bg')!;
    startWindowDragFromMouseDown({ button: 1, target: bg, defaultPrevented: false } as unknown as MouseEvent);
    expect(startDragging).not.toHaveBeenCalled();
  });
});
