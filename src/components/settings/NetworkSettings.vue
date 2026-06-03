<!--
  Exodus Browser — Network settings (proxy, DNS, etc.).
-->
<template>
  <section class="settings-section" data-testid="network-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <h4>{{ ui.proxySection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.proxyEnabled" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.enableProxy }}</span>
      </label>

      <template v-if="settings.proxyEnabled">
        <label>
          {{ ui.proxyType }}
          <select v-model="settings.proxyType" data-testid="proxy-type" @change="() => void persist()">
            <option value="http">HTTP</option>
            <option value="https">HTTPS</option>
            <option value="socks5">SOCKS5</option>
          </select>
        </label>
        <label>
          {{ ui.proxyHost }}
          <input v-model="settings.proxyHost" type="text" :placeholder="ui.proxyHostPlaceholder" @change="() => void persist()" />
        </label>
        <label>
          {{ ui.proxyPort }}
          <input v-model.number="settings.proxyPort" type="number" min="1" max="65535" @change="() => void persist()" />
        </label>
        <label class="checkbox-row">
          <input v-model="settings.proxyAuth" type="checkbox" @change="() => void persist()" />
          <span>{{ ui.proxyAuth }}</span>
        </label>
        <template v-if="settings.proxyAuth">
          <label>
            {{ ui.proxyUsername }}
            <input v-model="settings.proxyUsername" type="text" @change="() => void persist()" />
          </label>
          <label>
            {{ ui.proxyPassword }}
            <input v-model="settings.proxyPassword" type="password" @change="() => void persist()" />
          </label>
        </template>
      </template>

      <h4>{{ ui.dnsSection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.dnsOverHttps" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.dnsOverHttps }}</span>
      </label>

      <template v-if="settings.dnsOverHttps">
        <label>
          {{ ui.dnsProvider }}
          <select v-model="settings.dnsProvider" data-testid="dns-provider" @change="() => void persist()">
            <option value="cloudflare">Cloudflare (1.1.1.1)</option>
            <option value="google">Google (8.8.8.8)</option>
            <option value="quad9">Quad9 (9.9.9.9)</option>
            <option value="opendns">OpenDNS (208.67.222.222)</option>
            <option value="custom">{{ ui.custom }}</option>
          </select>
        </label>
        <label v-if="settings.dnsProvider === 'custom'">
          {{ ui.customDnsUrl }}
          <input v-model="settings.customDnsUrl" type="url" :placeholder="ui.customDnsPlaceholder" @change="() => void persist()" />
        </label>
      </template>

      <h4>{{ ui.connectionSection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.http2" type="checkbox" data-testid="http2" @change="() => void persist()" />
        <span>{{ ui.http2 }}</span>
      </label>
      <label class="checkbox-row">
        <input v-model="settings.http3" type="checkbox" data-testid="http3" @change="() => void persist()" />
        <span>{{ ui.http3 }}</span>
      </label>
      <label>
        {{ ui.maxConnections }}
        <input v-model.number="settings.maxConnections" type="number" min="1" max="256" data-testid="max-connections" @change="() => void persist()" />
      </label>

      <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="network-reset">
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — network settings (proxy, DNS, HTTP/2, HTTP/3).
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => networkSettingsStrings(props.uiLocale));

type NetworkSettings = {
  proxyEnabled: boolean;
  proxyType: 'http' | 'https' | 'socks5';
  proxyHost: string;
  proxyPort: number;
  proxyAuth: boolean;
  proxyUsername: string;
  proxyPassword: string;
  dnsOverHttps: boolean;
  dnsProvider: 'cloudflare' | 'google' | 'quad9' | 'opendns' | 'custom';
  customDnsUrl: string;
  http2: boolean;
  http3: boolean;
  maxConnections: number;
};

const STORAGE_KEY = 'exodus-network-settings';

const DEFAULT_SETTINGS: NetworkSettings = {
  proxyEnabled: false,
  proxyType: 'http',
  proxyHost: '',
  proxyPort: 8080,
  proxyAuth: false,
  proxyUsername: '',
  proxyPassword: '',
  dnsOverHttps: false,
  dnsProvider: 'cloudflare',
  customDnsUrl: '',
  http2: true,
  http3: true,
  maxConnections: 6,
};

const loading = ref(true);
const settings = ref<NetworkSettings>({ ...DEFAULT_SETTINGS });

function loadSettings(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      settings.value = { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.error('Failed to load network settings:', error);
    settings.value = { ...DEFAULT_SETTINGS };
  }
}

async function persist(): Promise<void> {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('Failed to save network settings:', error);
    emit('status', ui.value.saveError);
  }
}

async function resetToDefaults(): Promise<void> {
  settings.value = { ...DEFAULT_SETTINGS };
  await persist();
  emit('status', ui.value.reset);
}

onMounted(() => {
  loadSettings();
  loading.value = false;
});

function networkSettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '网络设置' : 'Network',
    hint: isZh ? '配置代理、DNS 和连接设置' : 'Configure proxy, DNS, and connection settings',
    loading: isZh ? '加载中...' : 'Loading...',
    proxySection: isZh ? '代理设置' : 'Proxy',
    enableProxy: isZh ? '启用代理' : 'Enable proxy',
    proxyType: isZh ? '代理类型' : 'Proxy type',
    proxyHost: isZh ? '代理主机' : 'Proxy host',
    proxyHostPlaceholder: isZh ? '例如：proxy.example.com' : 'e.g., proxy.example.com',
    proxyPort: isZh ? '代理端口' : 'Proxy port',
    proxyAuth: isZh ? '需要身份验证' : 'Authentication required',
    proxyUsername: isZh ? '用户名' : 'Username',
    proxyPassword: isZh ? '密码' : 'Password',
    dnsSection: isZh ? 'DNS 设置' : 'DNS',
    dnsOverHttps: isZh ? 'DNS over HTTPS' : 'DNS over HTTPS',
    dnsProvider: isZh ? 'DNS 提供商' : 'DNS provider',
    custom: isZh ? '自定义' : 'Custom',
    customDnsUrl: isZh ? '自定义 DNS URL' : 'Custom DNS URL',
    customDnsPlaceholder: isZh ? '例如：https://dns.example.com/dns-query' : 'e.g., https://dns.example.com/dns-query',
    connectionSection: isZh ? '连接设置' : 'Connection',
    http2: isZh ? '启用 HTTP/2' : 'Enable HTTP/2',
    http3: isZh ? '启用 HTTP/3' : 'Enable HTTP/3',
    maxConnections: isZh ? '最大连接数' : 'Max connections',
    reset: isZh ? '重置为默认值' : 'Reset to defaults',
    saved: isZh ? '网络设置已保存' : 'Network settings saved',
    saveError: isZh ? '保存网络设置失败' : 'Failed to save network settings',
  };
}
</script>

<style scoped>
.settings-hint {
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
  margin: 0 0 12px;
}

.loading-state {
  padding: 20px;
  text-align: center;
  color: var(--color-text-secondary, #9ca3af);
}

h4 {
  margin: 20px 0 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
  text-transform: uppercase;
}

label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

.checkbox-row {
  flex-direction: row;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

input,
select {
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-border, #404040);
  background: var(--color-bg-primary, #1a1a1a);
  color: var(--color-text-primary, #e0e0e0);
  font-size: 13px;
}

input[type="number"] {
  max-width: 120px;
}

input[type="password"] {
  font-family: monospace;
}

.nav-button {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: var(--color-bg-tertiary, #404040);
  color: #fff;
  font-size: 13px;
  margin-top: 16px;
}

.nav-button:hover {
  background: var(--color-bg-quaternary, #505050);
}

.settings-section h3 {
  margin: 0 0 8px;
  font-size: 14px;
  text-transform: uppercase;
  color: var(--color-text-secondary, #9ca3af);
}
</style>
