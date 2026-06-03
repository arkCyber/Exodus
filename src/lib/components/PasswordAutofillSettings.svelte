<script lang="ts">
  /**
   * Exodus Browser — password autofill / auto-save toggles in Settings.
   */
  import {
    loadPasswordManagerSettings,
    type PasswordManagerSettings,
  } from '$lib/passwordAutofill';
  import { invoke } from '@tauri-apps/api/core';

  type Props = {
    onStatus: (message: string) => void;
  };

  let { onStatus }: Props = $props();

  let settings = $state<PasswordManagerSettings>({
    auto_save: true,
    auto_fill: true,
    require_master_password: false,
    min_password_length: 8,
    require_strength_check: true,
    enable_breach_detection: true,
    auto_lock_timeout: 300,
    enable_sync: false,
  });
  let loaded = $state(false);

  async function load() {
    try {
      settings = await loadPasswordManagerSettings();
      loaded = true;
    } catch (error) {
      console.error('PasswordAutofillSettings load failed:', error);
      onStatus('Failed to load password settings');
    }
  }

  async function persist() {
    try {
      await invoke('update_password_manager_settings', { settings });
      onStatus('Password settings saved');
    } catch (error) {
      console.error('update_password_manager_settings failed:', error);
      onStatus('Failed to save password settings');
    }
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void load();
  });
</script>

{#if loaded}
  <div class="pw-autofill-settings">
    <h4>Password autofill</h4>
    <label class="checkbox-row">
      <input type="checkbox" bind:checked={settings.auto_fill} onchange={() => void persist()} />
      <span>Offer to fill saved passwords on login pages</span>
    </label>
    <label class="checkbox-row">
      <input type="checkbox" bind:checked={settings.auto_save} onchange={() => void persist()} />
      <span>Offer to save passwords after sign-in</span>
    </label>
  </div>
{/if}

<style>
  .pw-autofill-settings h4 {
    margin: 12px 0 8px;
    font-size: 14px;
    color: #e0e0e0;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
    font-size: 13px;
    color: #ccc;
    cursor: pointer;
  }
</style>
