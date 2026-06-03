/**
 * Exodus Browser — extension toolbar popup layout tests.
 */

import { describe, expect, it } from 'vitest';
import {
  CHROME_POPUP_HEIGHT,
  CHROME_POPUP_WIDTH,
  computeToolbarPopupRect,
  toolbarPopupLabel,
} from './extensionToolbarPopup';

describe('extensionToolbarPopup', () => {
  it('builds stable popup webview labels', () => {
    expect(toolbarPopupLabel('hello-world')).toBe('exodus-ext-popup-hello-world');
    expect(toolbarPopupLabel('bad/id!')).toBe('exodus-ext-popup-badid');
  });

  it('anchors popup below toolbar icon by default', () => {
    const anchor = new DOMRect(100, 40, 28, 28);
    const rect = computeToolbarPopupRect(anchor);

    expect(rect.width).toBe(CHROME_POPUP_WIDTH);
    expect(rect.height).toBe(CHROME_POPUP_HEIGHT);
    expect(rect.top).toBeGreaterThan(anchor.bottom);
    expect(rect.left).toBeGreaterThanOrEqual(8);
  });

  it('flips popup above icon when viewport is too short', () => {
    const originalInnerHeight = window.innerHeight;
    Object.defineProperty(window, 'innerHeight', {
      configurable: true,
      value: 800,
    });

    const anchor = new DOMRect(100, 550, 28, 28);
    const rect = computeToolbarPopupRect(anchor);

    expect(rect.bottom).toBeLessThanOrEqual(anchor.top);

    Object.defineProperty(window, 'innerHeight', {
      configurable: true,
      value: originalInnerHeight,
    });
  });
});
