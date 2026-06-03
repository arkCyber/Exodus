# Session Restore Test Plan

## Test Scenarios

### 1. Basic Session Save and Restore
**Steps:**
1. Open browser with default new tab
2. Navigate to multiple URLs (e.g., https://example.com, https://github.com, https://duckduckgo.com)
3. Close the browser window
4. Reopen the browser
5. Verify that all tabs are restored with correct URLs and titles
6. Verify that the active tab is the same as before closing

**Expected Result:**
- All tabs are restored
- Tab order is preserved
- Active tab is correct
- Tab titles are loaded correctly

### 2. Session Restore with Disabled Setting
**Steps:**
1. Open Settings > Privacy
2. Disable "Restore tabs on startup"
3. Navigate to multiple URLs
4. Close the browser
5. Reopen the browser
6. Verify that only the default new tab is open

**Expected Result:**
- Only default new tab is open
- Previous session is not restored

### 3. Session Restore with Private Mode
**Steps:**
1. Enable Private Mode in Settings > Privacy
2. Navigate to multiple URLs
3. Close the browser
4. Reopen the browser
5. Verify session behavior

**Expected Result:**
- Behavior depends on implementation (private mode may override session restore)

### 4. Session Restore with HTTPS-Only Mode
**Steps:**
1. Enable HTTPS-Only Mode in Settings > Privacy
2. Navigate to HTTP URLs (should upgrade to HTTPS)
3. Close the browser
4. Reopen the browser
5. Verify that URLs are correctly restored

**Expected Result:**
- URLs are restored with HTTPS scheme

### 5. Session Restore with Many Tabs
**Steps:**
1. Open 10-20 tabs with different URLs
2. Close the browser
3. Reopen the browser
4. Verify performance and correctness

**Expected Result:**
- All tabs are restored
- Performance is acceptable
- No errors or crashes

### 6. Session Restore After Browser Crash
**Steps:**
1. Navigate to multiple URLs
2. Force-kill the browser process (simulate crash)
3. Reopen the browser
4. Verify session behavior

**Expected Result:**
- Session may or may not be restored depending on `beforeunload` event handling

### 7. Session Restore with Invalid URLs
**Steps:**
1. Manually edit session database to include invalid URLs
2. Reopen the browser
3. Verify error handling

**Expected Result:**
- Invalid URLs are handled gracefully
- Valid URLs are still restored
- No crash or error shown to user

### 8. Session Restore Clear Function
**Steps:**
1. Navigate to multiple URLs
2. Close the browser (session saved)
3. Reopen the browser (session restored)
4. Clear session using backend command
5. Close and reopen the browser
6. Verify that no session is restored

**Expected Result:**
- Session is cleared
- Only default new tab opens on restart

## Test Data

### Sample URLs for Testing
- https://example.com
- https://github.com
- https://duckduckgo.com
- https://docs.rs
- https://developer.mozilla.org
- https://stackoverflow.com
- https://reddit.com

## Known Limitations

1. Session restore only saves tab metadata (URL, title, id), not page state (scroll position, form data)
2. Session restore only triggers when the browser starts with a single default new tab
3. Session restore depends on `beforeunload` event, which may not fire in crash scenarios

## Automation Potential

These tests could be automated using:
- Tauri test framework
- E2E testing with Playwright
- Mock database for edge case testing
