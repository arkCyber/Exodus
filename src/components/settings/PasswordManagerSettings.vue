<!--
  Exodus Browser — password manager settings section (Settings → Autofill).
-->
<template>
  <section class="settings-section" data-testid="password-manager-panel">
    <h3>{{ ui.title }}</h3>
    <div class="toolbar">
      <input
        v-model="searchQuery"
        type="search"
        data-testid="password-manager-search"
        :placeholder="ui.searchPlaceholder"
        class="field"
      />
      <button type="button" class="nav-button secondary" data-testid="password-manager-add" @click="showAdd = true">
        {{ ui.add }}
      </button>
      <button type="button" class="nav-button secondary" data-testid="password-manager-generate" @click="showGen = true">
        {{ ui.generate }}
      </button>
    </div>
    <p v-if="filtered.length === 0" class="hint">{{ ui.empty }}</p>
    <ul v-else class="list" data-testid="password-manager-list">
      <li v-for="p in filtered" :key="p.id" class="row">
        <div>
          <strong>{{ p.site_name }}</strong>
          <span class="muted">{{ p.username }} · {{ p.url }}</span>
        </div>
        <div class="actions">
          <button type="button" class="nav-button secondary" :title="ui.copy" @click="copy(p.password)">
            {{ ui.copy }}
          </button>
          <button type="button" class="nav-button secondary danger" @click="() => void remove(p.id)">
            {{ ui.delete }}
          </button>
        </div>
      </li>
    </ul>

    <div v-if="showAdd" class="dialog-backdrop" @click.self="showAdd = false">
      <div class="dialog">
        <h4>{{ ui.addTitle }}</h4>
        <label>{{ ui.site }} <input v-model="form.site_name" class="field" /></label>
        <label>{{ ui.url }} <input v-model="form.url" class="field" /></label>
        <label>{{ ui.username }} <input v-model="form.username" class="field" /></label>
        <label>{{ ui.password }} <input v-model="form.password" type="password" class="field" /></label>
        <div class="dialog-actions">
          <button type="button" class="nav-button secondary" @click="showAdd = false">{{ ui.cancel }}</button>
          <button type="button" class="nav-button" @click="() => void save()">{{ ui.save }}</button>
        </div>
      </div>
    </div>

    <div v-if="showGen" class="dialog-backdrop" @click.self="showGen = false">
      <div class="dialog">
        <h4>{{ ui.genTitle }}</h4>
        <label>{{ ui.genLength }} {{ genLen }} <input v-model.number="genLen" type="range" min="8" max="32" /></label>
        <label class="checkbox-row"><input v-model="genSymbols" type="checkbox" /> {{ ui.genSymbols }}</label>
        <button type="button" class="nav-button" @click="() => void generate()">{{ ui.genButton }}</button>
        <p v-if="generated" class="mono">{{ generated }}</p>
        <button v-if="generated" type="button" class="nav-button secondary" @click="copy(generated)">{{ ui.copy }}</button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — saved passwords CRUD (Tauri password vault).
 */
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { PasswordEntry } from '$lib/browserTypes';
import type { AppLocale } from '@/lib/appLocale';
import { passwordManagerSettingsStrings } from '@/lib/passwordManagerSettingsUi';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{ status: [message: string] }>();

const ui = computed(() => passwordManagerSettingsStrings(props.uiLocale));

const passwords = ref<PasswordEntry[]>([]);
const searchQuery = ref('');
const showAdd = ref(false);
const showGen = ref(false);
const generated = ref('');
const genLen = ref(16);
const genSymbols = ref(true);
const form = ref({ site_name: '', url: '', username: '', password: '' });

const filtered = computed(() => {
  const list = passwords.value ?? [];
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return list;
  return list.filter(
    (p) =>
      p.url.toLowerCase().includes(q) ||
      p.username.toLowerCase().includes(q) ||
      p.site_name.toLowerCase().includes(q),
  );
});

/** Load passwords from Tauri backend. */
async function load(): Promise<void> {
  try {
    passwords.value = await invoke<PasswordEntry[]>('list_passwords');
  } catch (error) {
    console.error('list_passwords failed:', error);
    emit('status', 'Failed to load passwords');
  }
}

/** Save a new password entry. */
async function save(): Promise<void> {
  try {
    const entry: PasswordEntry = {
      id: crypto.randomUUID(),
      url: form.value.url,
      username: form.value.username,
      password: form.value.password,
      site_name: form.value.site_name,
      created_at: Date.now() / 1000,
      updated_at: Date.now() / 1000,
      use_count: 0,
    };
    await invoke('save_password', { entry });
    showAdd.value = false;
    form.value = { site_name: '', url: '', username: '', password: '' };
    await load();
    emit('status', 'Password saved');
  } catch (error) {
    console.error('save_password failed:', error);
    emit('status', 'Failed to save password');
  }
}

/** Delete a saved password. */
async function remove(id: string): Promise<void> {
  if (!confirm('Delete this password?')) return;
  try {
    await invoke('delete_password', { id });
    await load();
    emit('status', 'Password removed');
  } catch (error) {
    console.error('delete_password failed:', error);
  }
}

/** Generate a random password via Tauri. */
async function generate(): Promise<void> {
  try {
    generated.value = await invoke<string>('generate_password', {
      length: genLen.value,
      includeSymbols: genSymbols.value,
    });
  } catch (error) {
    console.error('generate_password failed:', error);
  }
}

/** Copy text to clipboard and notify. */
function copy(text: string): void {
  void navigator.clipboard.writeText(text);
  emit('status', 'Copied to clipboard');
}

onMounted(() => void load());
</script>

<style scoped>
.toolbar { display: flex; gap: 8px; margin-bottom: 12px; flex-wrap: wrap; }
.field { flex: 1; min-width: 120px; padding: 6px 8px; border-radius: 6px; border: 1px solid var(--color-border, #404040); }
.list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px; }
.row { display: flex; justify-content: space-between; gap: 8px; padding: 8px; background: var(--color-bg-secondary, #2a2a2a); border-radius: 6px; }
.muted { display: block; font-size: 12px; color: var(--color-text-secondary, #888); }
.actions { display: flex; gap: 4px; }
.dialog-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 3000; }
.dialog { background: var(--color-bg-primary, #fff); padding: 16px; border-radius: 8px; min-width: 280px; display: flex; flex-direction: column; gap: 8px; }
.dialog-actions { display: flex; justify-content: flex-end; gap: 8px; }
.mono { font-family: monospace; word-break: break-all; }
.hint { font-size: 12px; color: var(--color-text-secondary, #888); }
.nav-button { padding: 6px 12px; border: none; border-radius: 6px; cursor: pointer; background: var(--color-accent, #2563eb); color: #fff; }
.nav-button.secondary { background: var(--color-bg-tertiary, #404040); }
.nav-button.danger { background: #7f1d1d; }
.checkbox-row { display: flex; align-items: center; gap: 8px; }
.settings-section h3 { margin: 0 0 12px; font-size: 14px; text-transform: uppercase; color: var(--color-text-secondary, #9ca3af); }
</style>
