import { createApp } from 'vue';
import { createRouter, createWebHashHistory } from 'vue-router';
import App from './App.vue';
import BrowserPage from './views/BrowserPage.vue';
import WasmEdgeSandboxTest from './components/WasmEdgeSandboxTest.vue';
import SiteIsolationTest from './components/SiteIsolationTest.vue';
import './app.css';
import './styles/theme.css';
import './styles/chrome-layout.css';
import './styles/sidebar-ui.css';
import { useTheme } from './composables/useTheme';
import { applyPlatformChromeClasses } from './lib/platformChrome';
import { logStartup, logStartupError } from './lib/startupLog';
import './lib/diagnosticLog';

if (import.meta.env.DEV) {
  import('./lib/e2eBridge');
}

logStartup('main.ts bootstrap start');

window.addEventListener('error', (event) => {
  logStartupError('window.error', event.error ?? event.message);
});

window.addEventListener('unhandledrejection', (event) => {
  logStartupError('unhandledrejection', event.reason);
});

try {
  applyPlatformChromeClasses();
  logStartup('platform chrome classes applied');
} catch (e) {
  logStartupError('platform chrome classes failed', e);
}

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: BrowserPage,
    },
    { path: '/chrome/settings', component: BrowserPage },
    { path: '/chrome/extensions', component: BrowserPage },
    { path: '/chrome/apps', component: BrowserPage },
    { path: '/chrome/history', component: BrowserPage },
    { path: '/chrome/bookmarks', component: BrowserPage },
    { path: '/chrome/downloads', component: BrowserPage },
    { path: '/chrome/newtab', component: BrowserPage },
    { path: '/sandbox-test', component: WasmEdgeSandboxTest },
    { path: '/site-isolation-test', component: SiteIsolationTest },
  ],
});

// Add router error handling
router.onError((error) => {
  logStartupError('router error', error);
});

router.isReady().then(() => {
  logStartup('router is ready');
}).catch((error) => {
  logStartupError('router init failed', error);
});

const app = createApp(App);
app.use(router);

logStartup('mount #app');
try {
  app.mount('#app');
  logStartup('Vue app mounted');
} catch (e) {
  logStartupError('Vue mount failed', e);
}

// Initialize theme system
try {
  const { loadTheme, setupSystemThemeListener } = useTheme();
  loadTheme();
  setupSystemThemeListener();
  logStartup('theme system initialized');
} catch (e) {
  logStartupError('theme init failed', e);
}
