/**
 * Playwright helpers — Exodus browser shell (Vite dev / shared E2E setup).
 */
import { expect, type Page } from '@playwright/test';

export const OMNIBOX = '.url-input, #exodus-omnibox-input';
export const TAB_STRIP = '.exodus-chrome-tabstrip .tab-item';
export const STATUS_MESSAGE = '.status-bar .message';

/** NTP layout localStorage keys (keep in sync with ntpLayoutStore). */
export const NTP_LAYOUT_STORAGE_KEYS = [
  'exodus-top-sites',
  'exodus-pinned-sites',
  'exodus-removed-ntp-sites',
  'exodus-ntp-quick-links',
  'exodus-removed-ntp-quick-links',
  'exodus-ntp-layout-customized-v1',
] as const;

/** Default top-site URLs normalized the same way as ntpTopSitesStore. */
export const DEFAULT_NTP_TOP_SITE_URLS = [
  'https://www.google.com/',
  'https://github.com/',
  'https://en.wikipedia.org/',
  'https://www.youtube.com/',
  'https://twitter.com/',
  'https://www.reddit.com/',
  'https://stackoverflow.com/',
  'https://developer.mozilla.org/',
];

/** Open full-page chrome://settings via menu or omnibox. */
export async function openSettingsModal(page: Page): Promise<void> {
  await page.locator('.perm-backdrop').evaluateAll((nodes) => {
    for (const el of nodes) {
      (el as HTMLElement).style.pointerEvents = 'none';
    }
  });
  await page.locator('.chrome-menu-btn').evaluate((el) => (el as HTMLButtonElement).click());
  await page
    .locator('.chrome-menu-dropdown button.menu-item')
    .filter({ hasText: 'Settings' })
    .evaluate((el) => (el as HTMLButtonElement).click());
  await expect(page.getByTestId('chrome-settings-page')).toBeVisible({ timeout: 15_000 });
}

/** Navigate directly to chrome://settings (stable for E2E). */
export async function openChromeSettings(page: Page, section?: string): Promise<void> {
  const url = section ? `chrome://settings/${section}` : 'chrome://settings';
  const omnibox = page.locator(OMNIBOX).first();
  await omnibox.fill(url);
  await omnibox.press('Enter');
  await expect(page.getByTestId('chrome-settings-page')).toBeVisible({ timeout: 15_000 });
}

/** Dismiss site/extension permission backdrops that block toolbar clicks. */
export async function dismissPermissionPrompts(page: Page): Promise<void> {
  for (let i = 0; i < 5; i++) {
    const backdrop = page.locator('.perm-backdrop');
    if (!(await backdrop.isVisible().catch(() => false))) return;
    const deny = page.getByRole('button', { name: 'Deny', exact: true });
    if (await deny.isVisible().catch(() => false)) {
      await deny.click({ force: true });
    } else {
      await page.keyboard.press('Escape');
    }
    await backdrop.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  }
}

/** Retry navigation when the dev server is still starting or briefly unavailable. */
export async function gotoWithRetry(page: Page, url: string, attempts = 4): Promise<void> {
  let lastError: unknown;
  for (let attempt = 1; attempt <= attempts; attempt++) {
    try {
      await page.goto(url);
      return;
    } catch (error) {
      lastError = error;
      const message = error instanceof Error ? error.message : String(error);
      const retriable =
        message.includes('ERR_CONNECTION_REFUSED') ||
        message.includes('ECONNREFUSED') ||
        message.includes('NS_ERROR_CONNECTION_REFUSED');
      if (!retriable || attempt === attempts) {
        throw error;
      }
      await page.waitForTimeout(1000 * attempt);
    }
  }
  throw lastError;
}

/** Open `/` and wait for browser chrome. */
export async function gotoBrowserShell(page: Page): Promise<void> {
  await gotoWithRetry(page, '/');
  await expect(page.locator('.browser-page')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
  await page
    .getByRole('button', { name: 'Deny', exact: true })
    .click({ force: true, timeout: 12_000 })
    .catch(() => {});
  await dismissPermissionPrompts(page);
}

/** Tab title texts in strip order (left → right); vertical strip only. */
export async function tabStripTitles(page: Page): Promise<string[]> {
  return page.locator(`${TAB_STRIP} .tab-title`).allTextContents();
}

/** Tab aria-labels in strip order (icon-only horizontal strip). */
export async function tabStripAriaLabels(page: Page): Promise<string[]> {
  return page.locator(TAB_STRIP).evaluateAll((nodes) =>
    nodes.map((n) => n.getAttribute('aria-label') ?? ''),
  );
}

/**
 * HTML5 drag-reorder between tab strip indices (pinned/unpinned rules still apply in app).
 */
export async function dragTabStripIndex(
  page: Page,
  fromIndex: number,
  toIndex: number,
): Promise<void> {
  const from = page.locator(TAB_STRIP).nth(fromIndex);
  const to = page.locator(TAB_STRIP).nth(toIndex);
  await from.dragTo(to);
}
