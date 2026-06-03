/**
 * Exodus Sample Net Rules — MV3 background (declarativeNetRequest + webRequest).
 *
 * Registers rules ONLY for the fake host `exodus-blocked.test` so real browsing is unaffected.
 * Hand-test: navigate to https://exodus-blocked.test/ — navigation should be blocked or redirected.
 */
(function () {
  'use strict';

  const LOG_PREFIX = '[Exodus sample-net-rules/bg]';
  /** Fake host used exclusively for dev net-rule tests (not a real site). */
  const BLOCK_HOST = 'exodus-blocked.test';
  const DNR_RULE_ID = 9001;
  const STORAGE_KEY = 'netRulesBootAt';

  /**
   * Timestamped debug log at service worker entry.
   * @param {string} phase - Phase label.
   * @param {...unknown} args - Log payload.
   */
  function tsLog(phase, ...args) {
    try {
      console.debug(LOG_PREFIX, new Date().toISOString(), phase, ...args);
    } catch (_) {
      /* ignore */
    }
  }

  /**
   * Run a Chrome API call with centralized error logging.
   * @param {string} label - Operation name.
   * @param {() => void} fn - API body.
   */
  function safeApi(label, fn) {
    try {
      fn();
    } catch (err) {
      tsLog('error', label, err);
    }
  }

  /**
   * Install declarativeNetRequest dynamic rules (block fake test host).
   */
  function installDnrRules() {
    if (!chrome.declarativeNetRequest?.updateDynamicRules) {
      tsLog('dnr.skip', 'declarativeNetRequest unavailable');
      return;
    }
    safeApi('dnr.updateDynamicRules', () => {
      chrome.declarativeNetRequest.updateDynamicRules({
        removeRuleIds: [DNR_RULE_ID],
        addRules: [
          {
            id: DNR_RULE_ID,
            priority: 1,
            action: { type: 'block' },
            condition: {
              urlFilter: '*' + BLOCK_HOST + '*',
              resourceTypes: ['main_frame', 'sub_frame'],
            },
          },
        ],
      });
      tsLog('dnr', 'block rule installed for', BLOCK_HOST);
    });
  }

  /**
   * Register webRequest blocking listener for the same fake host (flush → host store).
   */
  function installWebRequestBlock() {
    if (!chrome.webRequest?.onBeforeRequest?.addListener) {
      tsLog('webRequest.skip', 'webRequest unavailable');
      return;
    }
    safeApi('webRequest.onBeforeRequest', () => {
      chrome.webRequest.onBeforeRequest.addListener(
        function (_details) {
          return { cancel: true };
        },
        { urls: ['*://' + BLOCK_HOST + '/*'], types: ['main_frame'] },
        ['blocking']
      );
      tsLog('webRequest', 'onBeforeRequest blocking listener registered');
    });
  }

  tsLog('boot');

  if (!chrome?.runtime?.id) {
    tsLog('fatal', 'chrome.runtime.id unavailable');
    return;
  }

  if (chrome.storage?.local?.set) {
    chrome.storage.local.set({ [STORAGE_KEY]: Date.now() }, () => {
      if (chrome.runtime?.lastError) {
        tsLog('storage.err', chrome.runtime.lastError);
      }
    });
  }

  installDnrRules();
  installWebRequestBlock();

  if (chrome.runtime?.onInstalled?.addListener) {
    chrome.runtime.onInstalled.addListener((details) => {
      tsLog('onInstalled', details);
      installDnrRules();
      installWebRequestBlock();
    });
  }
})();
