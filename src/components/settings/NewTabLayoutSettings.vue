<!--
  Exodus Browser — new tab page layout reset (Settings → New tab page).
-->
<template>
  <section id="settings-section-ntp-layout" class="settings-section" data-testid="new-tab-layout-settings">
    <h3>New tab page</h3>
    <p class="settings-hint">
      Top sites (8-tile grid) and quick-link chips. Restore bundled defaults after customization.
    </p>
    <button type="button" class="nav-button secondary" @click="handleRestoreDefaults" data-testid="ntp-restore-defaults">
      Restore default layout
    </button>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — reset NTP top sites and quick links to first-run defaults.
 */
import { resetAllNtpLayout } from '@/lib/ntpLayoutStore';

const emit = defineEmits<{
  status: [message: string];
  ntpLayoutReset: [];
}>();

/** Wipe NTP layout storage and notify shell to refresh the new tab page. */
function handleRestoreDefaults(): void {
  try {
    resetAllNtpLayout();
    emit('ntpLayoutReset');
    emit('status', 'New tab page layout restored to defaults');
  } catch (error) {
    console.error('[NewTabLayoutSettings] restore defaults failed:', error);
    emit('status', 'Failed to restore new tab page layout');
  }
}
</script>

<style scoped>
.settings-hint {
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
  margin: 0 0 12px;
}
.nav-button.secondary {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: var(--color-bg-tertiary, #404040);
  color: #fff;
}
.settings-section h3 {
  margin: 0 0 12px;
  font-size: 14px;
  text-transform: uppercase;
  color: var(--color-text-secondary, #9ca3af);
}
</style>
