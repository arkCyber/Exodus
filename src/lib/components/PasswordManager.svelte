<script lang="ts">
  /**
   * Exodus Browser — Password Manager UI
   */

  import { invoke } from '@tauri-apps/api/core';
  import type { PasswordEntry } from '$lib/browserTypes';

  let passwords: PasswordEntry[] = [];
  let filteredPasswords: PasswordEntry[] = [];
  let searchQuery = '';
  let showAddDialog = false;
  let showEditDialog = false;
  let showGenerateDialog = false;
  let selectedPassword: PasswordEntry | null = null;

  // Form fields
  let newUrl = '';
  let newUsername = '';
  let newPassword = '';
  let newSiteName = '';

  // Password generation
  let generatedPassword = '';
  let passwordLength = 16;
  let includeSymbols = true;

  async function loadPasswords() {
    try {
      const result = await invoke<PasswordEntry[]>('list_passwords');
      passwords = result;
      filterPasswords();
    } catch (error) {
      console.error('Failed to load passwords:', error);
    }
  }

  function filterPasswords() {
    if (!searchQuery) {
      filteredPasswords = passwords;
    } else {
      const query = searchQuery.toLowerCase();
      filteredPasswords = passwords.filter(
        (p) =>
          p.url.toLowerCase().includes(query) ||
          p.username.toLowerCase().includes(query) ||
          p.site_name.toLowerCase().includes(query)
      );
    }
  }

  async function savePassword() {
    try {
      const entry: PasswordEntry = {
        id: crypto.randomUUID(),
        url: newUrl,
        username: newUsername,
        password: newPassword,
        site_name: newSiteName,
        created_at: Date.now() / 1000,
        updated_at: Date.now() / 1000,
        use_count: 0,
      };
      await invoke('save_password', { entry });
      await loadPasswords();
      showAddDialog = false;
      newUrl = '';
      newUsername = '';
      newPassword = '';
      newSiteName = '';
    } catch (error) {
      console.error('Failed to save password:', error);
    }
  }

  async function deletePassword(id: string) {
    try {
      await invoke('delete_password', { id });
      await loadPasswords();
    } catch (error) {
      console.error('Failed to delete password:', error);
    }
  }

  async function generatePassword() {
    try {
      const result = await invoke<string>('generate_password', {
        length: passwordLength,
        includeSymbols,
      });
      generatedPassword = result;
    } catch (error) {
      console.error('Failed to generate password:', error);
    }
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  // Load passwords on mount
  loadPasswords();

  $: searchQuery, filterPasswords();
</script>

<div class="password-manager">
  <div class="header">
    <h2>Password Manager</h2>
    <div class="actions">
      <input
        type="text"
        placeholder="Search passwords..."
        bind:value={searchQuery}
        class="search-input"
      />
      <button class="btn btn-primary" on:click={() => (showAddDialog = true)}>
        Add Password
      </button>
      <button class="btn btn-secondary" on:click={() => (showGenerateDialog = true)}>
        Generate Password
      </button>
    </div>
  </div>

  <div class="password-list">
    {#if filteredPasswords.length === 0}
      <div class="empty-state">
        <p>No passwords found</p>
      </div>
    {:else}
      {#each filteredPasswords as password (password.id)}
        <div class="password-item">
          <div class="password-info">
            <div class="site-name">{password.site_name}</div>
            <div class="url">{password.url}</div>
            <div class="username">{password.username}</div>
          </div>
          <div class="password-actions">
            <button
              class="btn-icon"
              title="Copy password"
              on:click={() => copyToClipboard(password.password)}
            >
              📋
            </button>
            <button
              class="btn-icon"
              title="Delete"
              on:click={() => deletePassword(password.id)}
            >
              🗑️
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Add Password Dialog -->
  {#if showAddDialog}
    <div class="dialog-overlay" on:click={() => (showAddDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Add Password</h3>
        <form on:submit|preventDefault={savePassword}>
          <div class="form-group">
            <label>Site Name</label>
            <input type="text" bind:value={newSiteName} required />
          </div>
          <div class="form-group">
            <label>URL</label>
            <input type="url" bind:value={newUrl} required />
          </div>
          <div class="form-group">
            <label>Username</label>
            <input type="text" bind:value={newUsername} required />
          </div>
          <div class="form-group">
            <label>Password</label>
            <input type="password" bind:value={newPassword} required />
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showAddDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Save</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Generate Password Dialog -->
  {#if showGenerateDialog}
    <div class="dialog-overlay" on:click={() => (showGenerateDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Generate Password</h3>
        <div class="form-group">
          <label>Password Length: {passwordLength}</label>
          <input type="range" min="8" max="32" bind:value={passwordLength} />
        </div>
        <div class="form-group">
          <label>
            <input type="checkbox" bind:checked={includeSymbols} />
            Include Symbols
          </label>
        </div>
        <div class="form-actions">
          <button class="btn btn-primary" on:click={generatePassword}>Generate</button>
        </div>
        {#if generatedPassword}
          <div class="generated-password">
            <div class="password-display">{generatedPassword}</div>
            <button class="btn btn-secondary" on:click={() => copyToClipboard(generatedPassword)}>
              Copy
            </button>
          </div>
        {/if}
        <div class="form-actions">
          <button type="button" class="btn btn-secondary" on:click={() => (showGenerateDialog = false)}>
            Close
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .password-manager {
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

  .password-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #888;
  }

  .password-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .password-info {
    flex: 1;
  }

  .site-name {
    font-weight: bold;
    margin-bottom: 5px;
  }

  .url {
    color: #888;
    font-size: 12px;
    margin-bottom: 3px;
  }

  .username {
    color: #aaa;
    font-size: 14px;
  }

  .password-actions {
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
    max-width: 500px;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
  }

  .form-group {
    margin-bottom: 15px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    color: #aaa;
  }

  .form-group input[type='text'],
  .form-group input[type='url'],
  .form-group input[type='password'] {
    width: 100%;
    padding: 8px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .form-group input[type='range'] {
    width: 100%;
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

  .generated-password {
    margin: 20px 0;
    padding: 15px;
    background: #444;
    border-radius: 4px;
  }

  .password-display {
    font-family: monospace;
    font-size: 16px;
    margin-bottom: 10px;
    word-break: break-all;
  }
</style>
