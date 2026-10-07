/* Shared browser helpers for popup + dashboard -> window.OdomouseUI */
(function () {
  'use strict';
  const U = window.OdomouseUnits;

  const I18n = window.OdomouseI18n;

  function parseKey(key) {
    const [y, m, d] = key.split('-').map(Number);
    return new Date(y, m - 1, d, 12);
  }
  function longDate(key) {
    return I18n.longDate(parseKey(key));
  }
  function shortDate(key) {
    const d = parseKey(key);
    return `${d.getDate()}.${String(d.getMonth() + 1).padStart(2, '0')}`;
  }
  function weekday(key) {
    return I18n.weekday(parseKey(key));
  }

  // ------------------------------------------------------------- clock
  let CLOCK = '24';

  /** setting: 'auto' | '24' | '12'; 'auto' is 12-hour in English. */
  function setClock(setting, lang) {
    CLOCK = setting === '12' || setting === '24' ? setting : (lang === 'en' ? '12' : '24');
  }
  const pad2 = (h) => String(h).padStart(2, '0');
  const ampm = (h) => (h < 12 ? 'AM' : 'PM');
  /** 14 -> "14:00" or "2:00 PM". */
  function hourText(h) {
    return CLOCK === '24' ? `${pad2(h)}:00` : `${h % 12 || 12}:00 ${ampm(h)}`;
  }
  /** Chart axis: 14 -> "14" or "2 PM". */
  function hourTick(h) {
    return CLOCK === '24' ? pad2(h) : `${h % 12 || 12} ${ampm(h)}`;
  }

  /**
   * Language and clock from a settings payload; fills the static texts.
   * Settings without `lang` (the screenshot harness): Uzbek.
   */
  function applySettings(s) {
    const lang = I18n.use((s && (s.lang || (I18n.LANGS.includes(s.language) && s.language))) || 'uz');
    setClock(s && s.clock, lang);
    I18n.apply(document);
    return lang;
  }

  /**
   * First run: a language question over the page until it is answered.
   * Only the core sends languageChosen (false): without it, never ask.
   */
  function languagePicker(s, api) {
    const old = document.getElementById('lang-pick');
    if (!s || s.languageChosen !== false) {
      if (old) old.remove();
      return false;
    }
    if (old) return true;
    const current = s.lang || 'uz';
    const box = el('div', { class: 'lang-pick', id: 'lang-pick', role: 'dialog', 'aria-modal': 'true', 'aria-labelledby': 'lang-pick-title' },
      el('div', { class: 'lang-card' },
        el('img', { src: '../assets/trayTemplate@2x.png', alt: '', class: 'lang-icon' }),
        el('h2', { id: 'lang-pick-title', text: I18n.t('chooseLanguage') }),
        el('div', { class: 'lang-options' }, ['uz', 'en', 'ru'].map((code) => el('button', {
          class: 'btn lang-option' + (code === current ? ' primary' : ''),
          lang: code,
          text: I18n.NATIVE_NAMES[code],
          onclick: () => api.updateSettings({ language: code, languageChosen: true }),
        }))),
        el('p', { class: 'note', text: I18n.t('changeLater') })));
    document.body.append(box);
    const first = box.querySelector('.lang-option.primary') || box.querySelector('.lang-option');
    if (first) first.focus();
    return true;
  }

  function el(tag, attrs, ...children) {
    const node = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs || {})) {
      if (v == null || v === false) continue;
      if (k === 'class') node.className = v;
      else if (k === 'text') node.textContent = v;
      else if (k.startsWith('on')) node.addEventListener(k.slice(2), v);
      else node.setAttribute(k, v === true ? '' : v);
    }
    for (const c of children.flat()) {
      if (c == null || c === false) continue;
      node.append(c instanceof Node ? c : document.createTextNode(String(c)));
    }
    return node;
  }

  // ------------------------------------------------------------- odometer
  const UNIT_CYCLE = ['m', 'km', 'ft', 'mi'];

  function nextUnit(unit) {
    const i = UNIT_CYCLE.indexOf(U.normalizeUnit(unit));
    return UNIT_CYCLE[(i + 1) % UNIT_CYCLE.length];
  }

  /** Digits for the wheels: integer part padded to `minInt`, plus one tenths wheel. */
  function odometerDigits(value, minInt = 5) {
    const tenthsTotal = Math.floor(Math.max(0, value) * 10 + 1e-9);
    const intPart = String(Math.floor(tenthsTotal / 10)).padStart(minInt, '0');
    return { int: intPart.split('').map(Number), tenths: tenthsTotal % 10 };
  }

  function cell(digit, extra) {
    const strip = el('span', { class: 'odo-strip' });
    for (let i = 0; i <= 9; i++) strip.append(el('span', { text: String(i) }));
    const c = el('span', { class: 'odo-cell' + (extra ? ' ' + extra : ''), 'aria-hidden': 'true' }, strip);
    c.setDigit = (d) => { strip.style.transform = `translateY(${-d * 10}%)`; }; // strip = 10 cells tall
    c.setDigit(digit);
    return c;
  }

  /**
   * Odometer for a distance in metres. Returns { root, update(meters, unitKey) }.
   * Rebuilds wheels only when the number of digits changes, otherwise rolls them.
   */
  function createOdometer({ onUnitClick } = {}) {
    const wheels = el('span', { class: 'odo-wheels' });
    const unitBtn = el('button', { class: 'odo-unit', onclick: () => onUnitClick && onUnitClick() });
    const sr = el('span', { class: 'sr-only', style: 'position:absolute;left:-9999px' });
    const root = el('div', { class: 'odo', role: 'group' }, wheels, unitBtn, sr);
    let cells = [];
    let tenthsCell = null;

    function update(meters, unitKey) {
      const f = U.formatDistance(meters, unitKey);
      const { int, tenths } = odometerDigits(f.value);
      if (cells.length !== int.length) {
        wheels.textContent = '';
        cells = int.map((d) => cell(d));
        tenthsCell = cell(tenths, 'tenths');
        wheels.append(...cells, el('span', { class: 'odo-cell sep', text: '.' }), tenthsCell);
      } else {
        int.forEach((d, i) => cells[i].setDigit(d));
        tenthsCell.setDigit(tenths);
      }
      unitBtn.textContent = f.unit;
      unitBtn.title = I18n.t('changeUnit');
      root.setAttribute('aria-label', I18n.t('distance'));
      sr.textContent = f.text;
    }
    return { root, update };
  }

  // ------------------------------------------------------------- colour
  // Sequential blue ramp from the dataviz reference palette (low -> high).
  // Softened versions of the sequential blue ramp (no neon highs).
  const RAMP_LIGHT = ['#e9ecf0', '#d3e1f3', '#adc8ec', '#86ade1', '#5f91d3', '#4777b8', '#365f96', '#284a76'];
  const RAMP_DARK = ['#2a2d33', '#2b3a52', '#30496f', '#385b8c', '#4a73ad', '#6690c8', '#8aaedb', '#b1c9e8'];

  function isDark() {
    const forced = document.documentElement.getAttribute('data-theme');
    if (forced) return forced === 'dark';
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  }

  function hexToRgb(h) {
    const n = parseInt(h.slice(1), 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  }

  /** t in 0..1 -> colour on the sequential ramp for the current theme. */
  function seqColor(t) {
    const ramp = isDark() ? RAMP_DARK : RAMP_LIGHT;
    const x = Math.min(Math.max(t, 0), 1) * (ramp.length - 1);
    const i = Math.min(Math.floor(x), ramp.length - 2);
    const f = x - i;
    const a = hexToRgb(ramp[i]);
    const b = hexToRgb(ramp[i + 1]);
    const c = a.map((v, k) => Math.round(v + (b[k] - v) * f));
    return { css: `rgb(${c[0]}, ${c[1]}, ${c[2]})`, lum: (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]) / 255 };
  }

  /** Ink that stays readable on a given fill. */
  function inkOn(fill) {
    return fill.lum > 0.55 ? '#2a2d33' : '#f8f9fb';
  }

  // ------------------------------------------------------------- tooltip
  let tip = null;
  function showTip(html, x, y) {
    if (!tip) { tip = el('div', { class: 'tooltip', role: 'tooltip' }); document.body.append(tip); }
    tip.innerHTML = html;
    tip.classList.add('show');
    const r = tip.getBoundingClientRect();
    let left = x + 12;
    let top = y - r.height - 10;
    if (left + r.width > window.innerWidth - 6) left = x - r.width - 12;
    if (top < 6) top = y + 16;
    tip.style.left = `${left}px`;
    tip.style.top = `${top}px`;
  }
  function hideTip() { if (tip) tip.classList.remove('show'); }

  function escapeHtml(s) {
    return String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  }

  /**
   * Change vs a previous period as neutral text: "▲ 12%", "▼ 4.2%", "≈ 0%".
   * More keystrokes is neither good nor bad, so no red/green.
   */
  function deltaInfo(ratio) {
    if (ratio == null || !Number.isFinite(ratio)) return null;
    const pct = Math.abs(ratio) * 100;
    if (pct < 0.5) return { text: '≈ 0%', dir: 0 };
    const shown = pct < 10 ? U.formatNumber(pct, 1) : U.formatNumber(pct, 0);
    return ratio > 0 ? { text: `▲ ${shown}%`, dir: 1 } : { text: `▼ ${shown}%`, dir: -1 };
  }

  window.OdomouseUI = {
    deltaInfo,
    longDate, shortDate, weekday, parseKey, el, escapeHtml,
    setClock, hourText, hourTick, applySettings, languagePicker, t: I18n.t, count: I18n.count,
    createOdometer, odometerDigits, nextUnit, UNIT_CYCLE,
    seqColor, inkOn, isDark, showTip, hideTip,
  };
})();
