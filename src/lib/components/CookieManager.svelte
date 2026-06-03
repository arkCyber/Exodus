<script lang="ts">
  /**
   * Exodus Browser — Cookie Manager UI
   */

  import { invoke } from '@tauri-apps/api/core';

  interface CookieEntry {
    id: string;
    domain: string;
    name: string;
    value: string;
    path: string;
    expires: number;
    created_at: number;
    last_accessed: number;
    secure: boolean;
    http_only: boolean;
    same_site: string;
  }

  let cookies: CookieEntry[] = [];
  let filteredCookies: CookieEntry[] = [];
  let searchQuery = '';
  let showDeleteDialog = false;
  let selectedCookie: CookieEntry | null = null;

  async function loadCookies() {
    try {
      const result = await invoke<CookieEntry[]>('list_cookies');
      cookies = result;
      filterCookies();
    } catch (error) {
      console.error('Failed to load cookies:', error);
    }
  }

  function filterCookies() {
    if (!searchQuery) {
      filteredCookies = cookies;
    } else {
      const query = searchQuery.toLowerCase();
      filteredCookies = cookies.filter(
        (c) =>
          c.domain.toLowerCase().includes(query) ||
          c.name.toLowerCase().includes(query)
      );
    }
  }

  async function deleteCookie(id: string) {
    try {
      await invoke('delete_cookie', { id });
      await loadCookies();
      showDeleteDialog = false;
    } catch (error) {
      console.error('Failed to delete cookie:', error);
    }
  }

  async function deleteCookiesForDomain(domain: string) {
    try {
      await invoke('delete_cookies_for_domain', { domain });
      await loadCookies();
    } catch (error) {
      console.error('Failed to delete cookies for domain:', error);
    }
  }

  async function deleteAllCookies() {
    if (confirm('Are you sure you want to delete all cookies?')) {
      try {
        await invoke('delete_all_cookies');
        await loadCookies();
      } catch (error) {
        console.error('Failed to delete all cookies:', error);
      }
    }
  }

  async function cleanupExpiredCookies() {
    try {
      const count = await invoke<number>('cleanup_expired_cookies');
      alert(`Cleaned up ${count} expired cookies`);
      await loadCookies();
    } catch (error) {
      console.error('Failed to cleanup expired cookies:', error);
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  function isExpired(expires: number): boolean {
    return expires > 0 && expires * 1000 < Date.now();
  }

  // Load cookies on mount
  loadCookies();

  $: searchQuery, filterCookies();
</script>

<div class="cookie-manager">
  <div class="header">
    <h2>Cookie Manager</h2>
    <div class="actions">
      <input
        type="text"
        placeholder="Search cookies..."
        bind:value={searchQuery}
        class="search-input"
      />
      <button class="btn btn-secondary" on:click={cleanupExpiredCookies}>
        Cleanup Expired
      </button>
      <button class="btn btn-danger" on:click={deleteAllCookies}>
        Delete All
      </button>
    </div>
  </div>

  <div class="cookie-list">
    {#if filteredCookies.length === 0}
      <div class="empty-state">
        <p>No cookies found</p>
      </div>
    {:else}
      {#each filteredCookies as cookie (cookie.id)}
        <div class="cookie-item {isExpired(cookie.expires) ? 'expired' : ''}">
          <div class="cookie-info">
            <div class="domain">{cookie.domain}</div>
            <div class="name">{cookie.name}</div>
            <div class="value">{cookie.value}</div>
            <div class="meta">
                <span class="path">Path: {cookie.path}</span>
                {#if cookie.expires > 0}
                  <span class="expires">Expires: {formatDate(cookie.expires)}</span>
                {:else}
                  <span class="expires">Session cookie</span>
                {/if}
                {#if cookie.secure}
                  <span class="badge secure">Secure</span>
                {/if}
                {#if cookie.http_only}
                  <span class="badge http-only">HttpOnly</span>
                {/if}
            </div>
          </div>
          <div class="cookie-actions">
            <button
              class="btn-icon"
              title="Delete cookies for this domain"
              on:click={() => deleteCookiesForDomain(cookie.domain)}
            >
              🗑️
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Delete Cookie Dialog -->
  {#if showDeleteDialog && selectedCookie}
    {@const cookieRow = selectedCookie}
    <div class="dialog-overlay" role="presentation" on:click={() => (showDeleteDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Delete Cookie</h3>
        <p>Are you sure you want to delete this cookie?</p>
        <div class="cookie-details">
          <div><strong>Domain:</strong> {cookieRow.domain}</div>
          <div><strong>Name:</strong> {cookieRow.name}</div>
        </div>
        <div class="form-actions">
          <button class="btn btn-secondary" on:click={() => (showDeleteDialog = false)}>
            Cancel
          </button>
          <button class="btn btn-danger" on:click={() => deleteCookie(cookieRow.id)}>
            Delete
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .cookie-manager {
    padding: 20px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .header h2 {
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 10px;
  }

  .search-input {
    padding: 8px 12px;
    border: 1px solid #555;
    border-radius: 6px;
    background: #333;
    color: #eee;
  }

  .cookie-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #888;
  }

  .cookie-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 15px;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .cookie-item.expired {
    opacity: 0.6;
    border-color: #666;
  }

  .cookie-info {
    flex: 1;
  }

  .domain {
    font-weight: bold;
    margin-bottom: 5px;
    color: #eee;
  }

  .name {
    color: #aaa;
    font-size: 14px;
    margin-bottom: 3px;
  }

  .value {
    color: #888;
    font-size: 12px;
    margin-bottom: 8px;
    font-family: monospace;
    word-break: break-all;
  }

  .meta {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    font-size: 12px;
    color: #888;
  }

  .badge {
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: bold;
  }

  .badge.secure {
    background: #059669;
    color: white;
  }

  .badge.http-only {
    background: #d97706;
    color: white;
  }

  .cookie-actions {
    display: flex;
    gap: 5px;
  }

  .btn-icon {
    background: #444;
    border: 1px solid #555;
    color: #eee;
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-icon:hover {
    background: #555;
  }

  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: #333;
    border: 1px solid #555;
    border-radius: 8px;
    padding: 20px;
    min-width: 400px;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
  }

  .cookie-details {
    margin: 20px 0;
    padding: 15px;
    background: #444;
    border-radius: 4px;
  }

  .cookie-details div {
    margin-bottom: 5px;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    border: none;
  }

  .btn-primary {
    background: #6366f1;
    color: white;
  }

  .btn-primary:hover {
    background: #4f46e5;
  }

  .btn-secondary {
    background: #444;
    color: #eee;
  }

  .btn-secondary:hover {
    background: #555;
  }

  .btn-danger {
    background: #dc2626;
    color: white;
  }

  .btn-danger:hover {
    background: #b91c1c;
  }
</style>
