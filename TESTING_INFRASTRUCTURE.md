# Exodus Browser - Testing Infrastructure

## Overview

This document describes the comprehensive testing infrastructure for Exodus Browser, including unit tests, integration tests, coverage reporting, CI/CD integration, and E2E testing.

## Test Structure

### Unit Tests
- **Location**: `src/**/*.test.ts`, `src/**/*.test.vue`
- **Framework**: Vitest with Vue Test Utils
- **Environment**: jsdom
- **Coverage**: 69+ test files covering core functionality

### Key Test Files
- `dnsOverHttps.test.ts` - DNS over HTTPS API tests
- `siteShields.test.ts` - Site shield and tracker blocking tests
- `tabLifecycle.test.ts` - Tab lifecycle management tests
- `browserIntegrations.test.ts` - Browser integration tests
- `omnibox.test.ts` - Omnibox search resolution tests
- `cdnIntegrations.test.ts` - P2P CDN integration tests
- `BrowserPage.test.ts` - Main browser page component tests

## Running Tests

### Unit Tests
```bash
# Run all unit tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage

# Run TypeScript check + tests
npm run test:frontend
```

### Coverage Reports
```bash
# Generate coverage report
npm run test:coverage

# View HTML coverage report
open coverage/index.html
```

Coverage thresholds:
- Lines: 70%
- Functions: 70%
- Branches: 65%
- Statements: 70%

### E2E Tests
```bash
# Install Playwright browsers (first time only)
npx playwright install

# Run E2E tests
npm run test:e2e

# Run E2E tests with UI
npm run test:e2e:ui

# Run E2E tests in headed mode
npm run test:e2e:headed
```

## CI/CD Integration

### GitHub Actions
The CI workflow (`.github/workflows/ci.yml`) automatically:
1. Runs full verification suite
2. Executes frontend tests with coverage
3. Uploads coverage reports to Codecov

### Coverage Reporting
- **Provider**: v8
- **Reporters**: text, json, html, lcov, clover
- **Upload**: Codecov integration for PR and main branch coverage tracking

## Test Configuration

### Vitest Configuration (`vitest.config.ts`)
- Includes TypeScript and Vue files
- Uses jsdom environment
- Configured with path aliases (@, $lib)
- Coverage excludes test files, node_modules, build artifacts
- Test timeout: 15 seconds

### Playwright Configuration (`playwright.config.ts`)
- Tests run on Chromium, Firefox, and WebKit
- Automatic dev server startup
- Retry on failure (2 retries in CI)
- Trace and screenshot capture on failure
- HTML reporter for test results

## Writing Tests

### Unit Test Example
```typescript
import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  isTauri: () => true,
}));

describe('myFunction', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('should call Tauri command', async () => {
    await myFunction();
    expect(invokeMock).toHaveBeenCalledWith('command_name', { param: 'value' });
  });
});
```

### E2E Test Example
```typescript
import { test, expect } from '@playwright/test';

test('navigates to URL', async ({ page }) => {
  await page.goto('/');
  const addressBar = page.locator('input[placeholder*="Search"]');
  await addressBar.fill('https://example.com');
  await addressBar.press('Enter');
  await page.waitForLoadState('networkidle');
});
```

## Best Practices

1. **Mock Tauri APIs**: Always mock `@tauri-apps/api/core` in unit tests
2. **Test Environment**: Use `isTauri: () => true` for Tauri-specific tests
3. **Error Handling**: Include tests for error cases and edge cases
4. **Async Tests**: Use `await` properly for async operations
5. **Cleanup**: Reset mocks in `beforeEach` hooks
6. **Coverage**: Aim for >70% coverage on new code

## Test Scripts Summary

| Script | Description |
|--------|-------------|
| `npm test` | Run unit tests |
| `npm run test:watch` | Run tests in watch mode |
| `npm run test:coverage` | Run tests with coverage |
| `npm run test:frontend` | TypeScript check + tests |
| `npm run test:e2e` | Run E2E tests |
| `npm run test:e2e:ui` | Run E2E tests with UI |
| `npm run test:e2e:headed` | Run E2E tests in headed mode |

## Coverage Reports

Coverage reports are generated in the `coverage/` directory:
- `index.html` - Interactive HTML report
- `lcov.info` - LCOV format for CI tools
- `coverage-final.json` - JSON format for custom processing

## Continuous Improvement

- Monitor coverage trends in Codecov
- Add tests for new features
- Update test thresholds as coverage improves
- Review failed tests in CI before merging
- Add E2E tests for critical user flows
