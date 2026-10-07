/*
 * Fake window.odomouse for rendering the UI pages in a plain browser (headless
 * screenshots, or opening the HTML by hand). Builds realistic data with the
 * same shapes the core sends. Injected before page scripts load.
 * window.__FAKE = { scenario: 'normal' | 'empty' | 'noperm' } picks the data.
 */
(function () {
  'use strict';
  const scenario = (window.__FAKE && window.__FAKE.scenario) || 'normal';
  let unit = (window.__FAKE && window.__FAKE.unit) || 'm';
  let theme = 'system';
  let compareDistance = 'auto';
  let compareText = 'a4';
  let onboardingDone = scenario !== 'new';
  let seenDashboard = scenario !== 'new';
  let trayShows = 'distance';
  let overrides = {};
  let breakReminder = true;
  let breakMinutes = 50;
  let trackApps = scenario !== 'appsoff';

  // deterministic pseudo-random
  let seed = 7;
  const rnd = () => { seed = (seed * 16807) % 2147483647; return (seed - 1) / 2147483646; };

  const today = '2026-10-03';
  const addDays = (key, n) => { const [y, m, d] = key.split('-').map(Number); const dt = new Date(y, m - 1, d + n, 12); return `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, '0')}-${String(dt.getDate()).padStart(2, '0')}`; };

  // English-ish letter frequencies on physical keys (uiohook codes)
  const FREQ = { 18: 12.7, 20: 9.1, 30: 8.2, 24: 7.5, 23: 7.0, 49: 6.7, 31: 6.3, 35: 6.1, 19: 6.0, 32: 4.3, 38: 4.0, 46: 2.8, 22: 2.8, 50: 2.4, 17: 2.4, 33: 2.2, 34: 2.0, 21: 2.0, 25: 1.9, 48: 1.5, 47: 1.0, 37: 0.8, 36: 0.15, 45: 0.15, 16: 0.1, 44: 0.07,
    57: 18, 14: 4.2, 28: 2.5, 42: 3.0, 3675: 2.2, 52: 1.2, 51: 1.1, 15: 0.8, 57419: 1.0, 57421: 0.9, 57416: 0.6, 57424: 0.6, 2: 0.5, 3: 0.4, 11: 0.4, 1: 0.3, 53: 0.3, 39: 0.2, 40: 0.4, 12: 0.3, 56: 0.4 };

  function keysFor(total) {
    const sum = Object.values(FREQ).reduce((a, b) => a + b, 0);
    const out = {};
    for (const [k, f] of Object.entries(FREQ)) { const n = Math.round(total * f / sum * (0.85 + rnd() * 0.3)); if (n > 0) out[k] = n; }
    return out;
  }

  function gridFor(scale) {
    const W = 32, H = 20, g = new Array(W * H).fill(0);
    const blobs = [[0.5, 0.45, 0.22, 1], [0.12, 0.08, 0.06, 0.6], [0.5, 0.96, 0.25, 0.35], [0.82, 0.3, 0.1, 0.5], [0.95, 0.03, 0.04, 0.4]];
    for (let r = 0; r < H; r++) for (let c = 0; c < W; c++) {
      const x = (c + 0.5) / W, y = (r + 0.5) / H;
      let v = 0;
      for (const [bx, by, s, w] of blobs) v += w * Math.exp(-((x - bx) ** 2 + (y - by) ** 2) / (2 * s * s));
      g[r * W + c] = Math.round(v * scale * (0.8 + rnd() * 0.4));
    }
    return g;
  }

  function hourlyFor(days) {
    const h = new Array(24).fill(0);
    for (let i = 0; i < 24; i++) {
      let v = 0;
      if (i >= 9 && i <= 12) v = 48; if (i >= 14 && i <= 18) v = 52; if (i === 13) v = 20; if (i >= 20 && i <= 23) v = 25; if (i === 8) v = 15; if (i === 1) v = 4;
      h[i] = Math.round(v * days * (0.7 + rnd() * 0.3));
    }
    return h;
  }

  function summary(days, mult) {
    if (scenario === 'empty') {
      return { date: today, mouseMeters: 0, scrollMeters: 0, keystrokes: 0, clicks: { left: 0, right: 0, middle: 0, other: 0, total: 0 },
        activeMinutes: 0, wpm: null, bestWpm: null, backspaceRatio: null, charsTyped: 0, topKeys: [], topLetters: [], peakHour: null, days, activeDays: 0,
        topShortcuts: [], shortcutTotal: 0, distinctShortcuts: 0, longestSessionMin: 0, apps: [] };
    }
    const keys = keysFor(14800 * mult);
    const letters = { 30: 'A', 48: 'B', 46: 'C', 32: 'D', 18: 'E', 33: 'F', 34: 'G', 35: 'H', 23: 'I', 36: 'J', 37: 'K', 38: 'L', 50: 'M', 49: 'N', 24: 'O', 25: 'P', 16: 'Q', 19: 'R', 31: 'S', 20: 'T', 22: 'U', 47: 'V', 17: 'W', 45: 'X', 21: 'Y', 44: 'Z' };
    const top = Object.entries(keys).map(([c, n]) => ({ code: +c, count: n })).sort((a, b) => b.count - a.count);
    return {
      date: today, mouseMeters: 342.17 * mult, scrollMeters: 61.4 * mult, keystrokes: Object.values(keys).reduce((a, b) => a + b, 0),
      clicks: { left: 1142 * mult, right: 96 * mult, middle: 0, other: 3, total: 1241 * mult }, activeMinutes: 287 * mult,
      wpm: 58.3, bestWpm: 84, backspaceRatio: 0.046, charsTyped: 11200 * mult, topKeys: top.slice(0, 10),
      topLetters: top.filter((k) => letters[k.code]).slice(0, 5).map((k) => ({ letter: letters[k.code], count: k.count })),
      peakHour: 15, days, activeDays: days, _keys: keys,
      topShortcuts: [['8:46', 143], ['8:47', 120], ['8:15', 88], ['8:44', 61], ['8:31', 40], ['8:20', 25], ['8:17', 22], ['8:33', 18], ['12:5', 12], ['8:30', 9]]
        .map(([id, c]) => ({ id, count: Math.round(c * mult) })),
      shortcutTotal: Math.round(560 * mult), distinctShortcuts: 14, longestSessionMin: 95,
      apps: scenario === 'appsoff' ? [] : [['Visual Studio Code', 130, 8200, 310, 92], ['Google Chrome', 74, 2100, 520, 140], ['Telegram', 41, 3100, 90, 25],
        ['Figma', 26, 300, 410, 70], ['Terminal', 15, 900, 12, 4], ['Finder', 7, 40, 60, 11]]
        .map(([name, m, k, c, mt]) => ({ name, minutes: Math.round(m * mult), keys: Math.round(k * mult), clicks: Math.round(c * mult), meters: mt * mult, scrollMeters: 5 })),
    };
  }

  const ACH = [
    ['dist_day_100m', 'distance', 'Isinish', 'Bir kunda kursor bilan 100 m yurish', '2026-09-30'],
    ['dist_day_500m', 'distance', 'Sichqoncha sprinteri', 'Bir kunda 500 m', null],
    ['dist_1km', 'distance', 'Birinchi kilometr', 'Jami 1 km masofa', '2026-10-02'],
    ['dist_10km', 'distance', "Uzoq yo'l", 'Jami 10 km masofa', null],
    ['dist_marathon', 'distance', 'Marafonchi', "Jami 42.195 km, ya'ni bitta marafon", null],
    ['keys_day_10k', 'keys', 'Faol barmoqlar', 'Bir kunda 10 000 ta bosish', '2026-10-01'],
    ['keys_day_30k', 'keys', 'Yozuvchi', 'Bir kunda 30 000 ta bosish', null],
    ['keys_100k', 'keys', 'Yuz ming', 'Jami 100 000 ta bosish', null],
    ['keys_1m', 'keys', 'Millioner', 'Jami 1 000 000 ta bosish', null],
    ['clicks_10k', 'clicks', 'Klikchi', 'Jami 10 000 ta klik', null],
    ['wpm_60', 'speed', 'Tezkor barmoqlar', "Yozish tezligi 60 so'z/daqiqaga yetdi", '2026-10-03'],
    ['wpm_90', 'speed', 'Chaqmoq', "Yozish tezligi 90 so'z/daqiqaga yetdi", null],
    ['accurate', 'speed', 'Aniq yozuvchi', "Bir kunda 2 000+ belgi yozib, Backspace'ni 3% dan kam bosish", null],
    ['scroll_1km', 'scroll', 'Cheksiz lenta', 'Jami 1 km scroll', null],
    ['night_owl', 'time', "Tungi boyo'g'li", 'Soat 00:00 dan 04:59 gacha ishlash', '2026-10-02'],
    ['early_bird', 'time', 'Erta turuvchi', 'Soat 05:00 dan 06:59 gacha ishlash', null],
    ['focus_4h', 'time', 'Diqqat', 'Bir kunda 4 soat faol ishlash', '2026-10-01'],
    ['streak_7', 'streak', 'Bir hafta', "Ketma-ket 7 kun faol bo'lish", null],
    ['streak_30', 'streak', 'Odat', "Ketma-ket 30 kun faol bo'lish", null],
  ].map(([id, kind, title, desc, earnedOn]) => ({ id, kind, title, desc, earnedOn: scenario === 'empty' ? null : earnedOn }));

  function displays() {
    const est = scenario === 'noperm';
    return [
      { id: '1', label: 'Built-in Retina Display', internal: true, source: overrides['1'] ? 'diagonal' : (est ? 'estimate' : 'hardware'), width: 1470, height: 956, diagonalInches: overrides['1'] || 13.6, pointsPerInch: 128.9 },
      { id: '2', label: 'DELL U2723QE', internal: false, source: overrides['2'] ? 'diagonal' : 'estimate', width: 2560, height: 1440, diagonalInches: overrides['2'] || 26.9, pointsPerInch: 109 },
    ];
  }

  const FUN = ["Kursor 3.3 × futbol maydoni masofani bosib o'tdi", 'Siz 6.2 ta kitob sahifasiga teng matn yozdingiz', "Eng tez yozishingiz: 84 so'z/daqiqa", 'Eng faol soatingiz: 15:00–15:59'];

  const F = window.__FAKE || {};
  let language = F.lang || 'auto';
  let languageChosen = F.chosen !== false;
  let clock = F.clock || 'auto';
  const sysLang = F.sysLang || 'uz';
  const lang = () => (language === 'auto' ? sysLang : language);

  function settings() {
    const s = { unit, theme, compareDistance, compareText, onboardingDone, seenDashboard, trayShows, diagonalOverrides: { ...overrides }, breakReminder, breakMinutes, trackApps };
    // the core sends these; without them the pages stay Uzbek and never ask
    if (F.native || F.lang || F.sysLang || F.chosen === false) Object.assign(s, { language, languageChosen, clock, lang: lang(), systemLang: sysLang });
    return s;
  }
  const L = (uz, en, ru) => ({ uz, en, ru })[lang()];

  // same rules as src/core/fun.js (kept tiny on purpose)
  const DIST = { auto: null, banana: ['banan', 0.18], a4: ['A4 varaq', 0.297], bus: ['avtobus', 12], pool: ['olimpiya basseyni', 50], football: ['futbol maydoni', 105], eiffel: ['Eyfel minorasi', 330], marathon: ['marafon', 42195] };
  const TEXT = { auto: null, sms: ['SMS', 160], tweet: ['tvit', 280], page: ['kitob sahifasi', 1800], a4: ['A4 varaq', 3000] };
  function cmpOf(table, amount, key) {
    if (!(amount > 0)) return null;
    let ref = table[key];
    if (!ref) { ref = null; for (const k of Object.keys(table)) if (table[k] && amount >= 2 * table[k][1]) ref = table[k]; ref = ref || Object.values(table)[1]; }
    const r = amount / ref[1];
    return { text: `≈ ${r < 10 ? r.toFixed(1) : Math.round(r)} ${ref[0]}` };
  }
  function comparisons(sum) {
    const Fun = window.OdomouseFun;
    if (Fun && Fun.setLanguage) {
      Fun.setLanguage(lang());
      return { distance: Fun.distanceComparison(sum.mouseMeters, compareDistance), text: Fun.textComparison(sum.charsTyped, compareText) };
    }
    return { distance: cmpOf(DIST, sum.mouseMeters, compareDistance), text: cmpOf(TEXT, sum.charsTyped, compareText) };
  }

  function compare(label, short) {
    if (scenario === 'empty') return null;
    return { label, short, deltas: { mouseMeters: 0.18, scrollMeters: -0.07, keystrokes: 0.12, clicks: -0.034, activeMinutes: 0.002 } };
  }

  function live() {
    const sum = scenario === 'new' ? { ...summary(1, 1), mouseMeters: 0 } : summary(1, 1);
    return { summary: sum, comparisons: comparisons(sum), settings: settings(), hooksRunning: scenario !== 'noperm' && scenario !== 'new', estimatedDisplays: scenario === 'noperm',
      compare: compare(L('Kechagi shu vaqtga nisbatan', 'Compared with yesterday at this time', 'По сравнению со вчера в это же время'), L('kecha shu vaqtga', 'vs. yesterday', 'ко вчера')),
      session: scenario === 'empty' ? { minutes: 0, breakIn: null } : (scenario === 'due' ? { minutes: 63, breakIn: 0 } : { minutes: 42, breakIn: 8 }) };
  }

  function dashboard(range) {
    const n = { today: 1, '7d': 7, '30d': 30, all: 45 }[range] || 7;
    const s = summary(n, n * 0.9);
    const series = [];
    if (range !== 'today') {
      for (let i = n - 1; i >= 0; i--) {
        const weekend = [0, 6].includes(new Date(addDays(today, -i).replace(/-/g, '/')).getDay());
        const f = scenario === 'empty' ? 0 : (weekend ? 0.25 : 0.6 + rnd() * 0.8);
        series.push({ date: addDays(today, -i), mouseMeters: 340 * f, scrollMeters: 60 * f, keystrokes: Math.round(14800 * f), clicks: Math.round(1240 * f), activeMinutes: Math.round(290 * f) });
      }
    }
    return {
      range, from: addDays(today, -(n - 1)), to: today, summary: s, comparisons: comparisons(s), series,
      compare: range === 'all' ? null : (range === 'today'
        ? compare(L('Kechagi shu vaqtga nisbatan', 'Compared with yesterday at this time', 'По сравнению со вчера в это же время'), L('kecha shu vaqtga', 'vs. yesterday', 'ко вчера'))
        : compare(L(`Oldingi ${n} kunga nisbatan`, `Compared with the previous ${n} days`, `По сравнению с предыдущими ${n} днями`), L(`oldingi ${n} kun`, `vs. previous ${n} days`, `к пред. ${n} дням`))),
      session: { minutes: 42, breakIn: 8 },
      hourly: scenario === 'empty' ? new Array(24).fill(0) : hourlyFor(n), keys: scenario === 'empty' ? {} : s._keys,
      grid: gridFor(scenario === 'empty' ? 0 : 60 * n), gridW: 32, gridH: 20, gridAspect: 1470 / 956,
      settings: settings(), displays: displays(), hooksRunning: scenario !== 'noperm',
    };
  }

  const listeners = {};
  const on = (ch) => (cb) => { (listeners[ch] = listeners[ch] || []).push(cb); return () => {}; };
  const emit = (ch, p) => (listeners[ch] || []).forEach((cb) => cb(p));

  window.odomouse = {
    platform: window.ODOMOUSE_PLATFORM || 'mac',
    native: !!(window.__FAKE && window.__FAKE.native),
    getLive: async () => live(),
    onLive: on('live'),
    onSettings: on('settings'),
    onDisplays: on('displays'),
    onAchievements: on('ach'),
    onTab: on('tab'),
    getDashboard: async (range) => dashboard(range),
    openDashboard: async (tab) => { window.__lastOpen = tab; },
    updateSettings: async (p) => {
      if (p.unit) unit = p.unit;
      if (p.trayShows) trayShows = p.trayShows;
      if (p.diagonalOverrides) overrides = { ...p.diagonalOverrides };
      if ('breakReminder' in p) breakReminder = p.breakReminder;
      if (p.breakMinutes) breakMinutes = p.breakMinutes;
      if ('trackApps' in p) trackApps = p.trackApps;
      if (p.theme) theme = p.theme;
      if (p.compareDistance) compareDistance = p.compareDistance;
      if (p.compareText) compareText = p.compareText;
      if ('onboardingDone' in p) onboardingDone = p.onboardingDone;
      if ('seenDashboard' in p) seenDashboard = p.seenDashboard;
      if (p.language) language = p.language;
      if ('languageChosen' in p) languageChosen = p.languageChosen;
      if (p.clock) clock = p.clock;
      emit('settings', settings()); emit('live', live());
      return { settings: settings(), displays: displays() };
    },
    getWrapped: async (period) => {
      const n = { week: 7, month: 30, all: 45 }[period] || 7;
      const label = { week: L('Shu hafta', 'This week', 'Эта неделя'), month: L('Shu oy', 'This month', 'Этот месяц'), all: L('Butun vaqt', 'All time', 'За всё время') }[period] || L('Shu hafta', 'This week', 'Эта неделя');
      const sum = summary(n, n * 0.85);
      return { period, label, from: addDays(today, -(n - 1)), to: today, summary: sum, comparisons: comparisons(sum), settings: settings() };
    },
    saveWrapped: async (d) => { window.__saved = d.length; return { ok: true, filePath: '/Users/foziljon/Desktop/odomouse.png' }; },
    copyWrapped: async () => ({ ok: true }),
    exportCsv: async () => ({ ok: true, filePath: '/Users/foziljon/Documents/odomouse.csv' }),
    clearApps: async () => ({ ok: true }),
    resetData: async () => ({ ok: false }),
    openPermissions: async () => { window.__perm = true; },
    quit: async () => {},
  };
})();
