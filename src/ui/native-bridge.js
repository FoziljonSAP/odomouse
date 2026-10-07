/*
 * window.odomouse: the pages' way to talk to the native app, through the web
 * view's message channel (or HTTP on Linux):
 *   macOS   WKWebView  -> window.webkit.messageHandlers.odomouse.postMessage
 *   Windows WebView2   -> window.chrome.webview.postMessage
 *   Linux   WebKitGTK  -> window.webkit.messageHandlers.odomouse.postMessage
 *   Linux    browser    -> fetch('/api') + EventSource('/events') on 127.0.0.1
 * Requests are {id, method, args}; the app answers by evaluating
 * window.__odomouseReply(id, result) and pushes events with
 * window.__odomouseEmit(name, payload) (names: live, settings, displays, tab).
 * The app sets window.ODOMOUSE_PLATFORM ('mac' | 'windows' | 'linux') before
 * the page loads.
 */
(function () {
  'use strict';
  if (window.odomouse) return;

  const wk = window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.odomouse;
  const wv2 = window.chrome && window.chrome.webview;
  const http = window.ODOMOUSE_HTTP === true; // Linux: page served by the app on 127.0.0.1
  let post = null;
  if (wk) post = (msg) => wk.postMessage(msg);
  else if (wv2) post = (msg) => wv2.postMessage(msg);
  else if (http) {
    post = (msg) => {
      fetch('/api', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(msg), credentials: 'same-origin' })
        .then((r) => r.json())
        .then((res) => window.__odomouseReply(msg.id, res))
        .catch((e) => window.__odomouseReply(msg.id, { error: String(e) }));
    };
  }
  if (!post) return; // plain browser: the page shows its own empty state

  // Native windows have a real title bar: no drag strip.
  document.documentElement.classList.add('native-chrome');

  // Light / dark from the user's setting (the app sets window.ODOMOUSE_THEME
  // before load; later changes come with every payload that has settings).
  function applyTheme(theme) {
    if (theme === 'light' || theme === 'dark') document.documentElement.setAttribute('data-theme', theme);
    else if (theme) document.documentElement.removeAttribute('data-theme');
  }
  applyTheme(window.ODOMOUSE_THEME);
  const themeFrom = (x) => {
    if (!x || typeof x !== 'object') return;
    if (x.settings && x.settings.theme) applyTheme(x.settings.theme);
    else if (typeof x.theme === 'string' && 'unit' in x) applyTheme(x.theme);
  };

  let seq = 0;
  const pending = new Map();
  const listeners = { live: new Set(), settings: new Set(), displays: new Set(), tab: new Set() };

  window.__odomouseReply = function (id, result) {
    const p = pending.get(id);
    if (!p) return;
    pending.delete(id);
    themeFrom(result);
    if (result && typeof result === 'object' && typeof result.alert === 'string') window.alert(result.alert);
    if (result && typeof result === 'object' && result.error && Object.keys(result).length === 1) p.reject(new Error(result.error));
    else p.resolve(result);
  };

  window.__odomouseEmit = function (name, payload) {
    if (name === 'settings' || name === 'live') themeFrom(payload);
    const set = listeners[name];
    if (set) set.forEach((cb) => { try { cb(payload); } catch (e) { console.error(e); } });
  };

  // Over HTTP the app cannot show its own dialogs: confirm here.
  // (texts from i18n.js, which loads after this file)
  const CONFIRM = { resetData: 'confirmReset', clearApps: 'confirmApps' };

  function call(method, ...args) {
    const ask = CONFIRM[method] && window.OdomouseI18n ? window.OdomouseI18n.t(CONFIRM[method]) : null;
    if (http && ask && !window.confirm(ask)) return Promise.resolve({ ok: false });
    return new Promise((resolve, reject) => {
      const id = ++seq;
      pending.set(id, { resolve, reject });
      try {
        post({ id, method, args });
      } catch (e) {
        pending.delete(id);
        reject(e);
      }
    });
  }

  const subscribe = (name) => (cb) => {
    listeners[name].add(cb);
    return () => listeners[name].delete(cb);
  };

  if (http && window.EventSource) {
    const events = new EventSource('/events');
    events.onmessage = (e) => {
      try {
        const m = JSON.parse(e.data);
        window.__odomouseEmit(m.name, m.payload);
      } catch (_) { /* ignore */ }
    };
  }

  window.odomouse = {
    platform: window.ODOMOUSE_PLATFORM || 'mac',
    native: true,
    getLive: () => call('getLive'),
    onLive: subscribe('live'),
    onSettings: subscribe('settings'),
    onDisplays: subscribe('displays'),
    onTab: subscribe('tab'),
    getDashboard: (range) => call('getDashboard', range),
    openDashboard: (tab) => call('openDashboard', tab),
    updateSettings: (partial) => call('updateSettings', partial),
    getWrapped: (period) => call('getWrapped', period),
    saveWrapped: (dataUrl, period) => call('saveWrapped', dataUrl, period),
    copyWrapped: (dataUrl) => call('copyWrapped', dataUrl),
    exportCsv: () => call('exportCsv'),
    clearApps: () => call('clearApps'),
    resetData: () => call('resetData'),
    openPermissions: () => call('openPermissions'),
    quit: () => call('quit'),
  };
})();
