(function () {
  'use strict';
  const U = window.OdomouseUnits;
  const UI = window.OdomouseUI;
  const C = window.OdomouseCharts;
  const KB = window.OdomouseKeyboard;
  const SC = window.OdomouseShortcuts;
  const Fun = window.OdomouseFun;
  const { el } = UI;
  const api = window.odomouse;
  const PLATFORM = KB.PLATFORM; // 'mac' | 'windows' | 'linux'
  const NATIVE = !!(api && api.native);
  const { t } = UI;
  const trayName = () => t({ mac: 'trayMac', windows: 'trayWindows', linux: 'trayLinux' }[PLATFORM]);
  const device = () => t(PLATFORM === 'mac' ? 'onThisMac' : 'onThisComputer');

  const state = {
    // ?tab=… (Windows, Linux) or window.ODOMOUSE_TAB (macOS: file URLs there cannot carry a query)
    tab: (new URLSearchParams(location.search).get('tab') || window.ODOMOUSE_TAB) === 'settings' ? 'settings' : 'stats',
    range: '7d',
    metric: 'mouseMeters',
    showTable: false,
    data: null,
    settings: null,
  };

  const odo = UI.createOdometer({
    onUnitClick: () => state.settings && api.updateSettings({ unit: UI.nextUnit(state.settings.unit) }),
  });
  document.getElementById('dash-odo').append(odo.root);

  const unit = () => (state.settings ? state.settings.unit : 'm');

  const METRICS = {
    mouseMeters: { title: 'distance', fmt: (v) => U.formatDistance(v, unit()).text, tick: (v) => U.formatDistance(v, unit(), 0).text },
    keystrokes: { title: 'keyboard', fmt: (v) => U.formatCount(v), tick: (v) => U.formatCount(v) },
    clicks: { title: 'clicks', fmt: (v) => U.formatCount(v), tick: (v) => U.formatCount(v) },
    activeMinutes: { title: 'activeTime', fmt: (v) => U.formatMinutes(v), tick: (v) => `${Math.round(v)} ${U.minuteUnits().min}` },
  };

  function setPressed(groupId, attr, value) {
    for (const b of document.querySelectorAll(`#${groupId} button`)) b.setAttribute('aria-pressed', String(b.dataset[attr] === value));
  }

  function rangeText(d) {
    if (d.range === 'today') return UI.longDate(d.to);
    return `${UI.longDate(d.from)} – ${UI.longDate(d.to)}`;
  }

  // ---------------------------------------------------------------- tabs
  function showTab(tab) {
    state.tab = tab === 'settings' ? 'settings' : 'stats';
    for (const b of document.querySelectorAll('.nav-item')) {
      if (b.dataset.tab === state.tab) b.setAttribute('aria-current', 'page'); else b.removeAttribute('aria-current');
    }
    document.getElementById('tab-stats').hidden = state.tab !== 'stats';
    document.getElementById('tab-settings').hidden = state.tab !== 'settings';
    load();
  }

  // ---------------------------------------------------------------- stats
  function renderStats(d) {
    const s = d.summary;
    document.getElementById('range-dates').textContent = rangeText(d);
    setPressed('range', 'range', d.range);
    setPressed('metric', 'metric', state.metric);

    const notices = document.getElementById('stats-notices');
    notices.textContent = '';
    if (!d.hooksRunning) {
      notices.append(el('div', { class: 'notice' }, el('p', { text: t('permissionNeeded') }),
        el('button', { class: 'btn', text: t('grant'), onclick: () => api.openPermissions() })));
    }
    if (d.displays.some((x) => x.source === 'estimate')) {
      notices.append(el('div', { class: 'notice quiet' }, el('p', { text: t('distanceApprox') }),
        el('button', { class: 'btn', text: t('adjust'), onclick: () => showTab('settings') })));
    }

    odo.update(s.mouseMeters, unit());
    const cmp = d.compare;
    const deltaText = (metric) => {
      const info = cmp ? UI.deltaInfo(cmp.deltas[metric]) : null;
      return info ? `${info.text} ${cmp.short}` : '';
    };
    document.getElementById('hero-delta').textContent = deltaText('mouseMeters');

    const fun = document.getElementById('fun-list');
    fun.textContent = '';
    const lines = [];
    if (d.comparisons.distance) lines.push(t('funDistance', { v: d.comparisons.distance.text }));
    if (d.comparisons.text) lines.push(t('funText', { v: d.comparisons.text.text }));
    if (s.peakHour != null) lines.push(t('funPeak', { v: UI.hourText(s.peakHour) }));
    if (s.longestSessionMin >= 30) lines.push(t('funLongest', { v: U.formatMinutes(s.longestSessionMin) }));
    for (const l of lines) fun.append(el('li', { text: l }));
    if (!lines.length) fun.append(el('li', { class: 'empty', text: t('noDataYet') }));

    const kpis = document.getElementById('kpis');
    kpis.textContent = '';
    const kpi = (label, value, metric, title) => el('div', { class: 'kpi', title: title || '' },
      el('p', { class: 'label', text: label }), el('p', { class: 'value', text: value }),
      el('p', { class: 'delta', text: deltaText(metric) || ' ', title: cmp ? cmp.label : '' }));
    kpis.append(
      kpi(t('keyboard'), U.formatCount(s.keystrokes), 'keystrokes'),
      kpi(t('clicks'), U.formatCount(s.clicks.total), 'clicks', t('clicksLeftRight', { l: U.formatCount(s.clicks.left), r: U.formatCount(s.clicks.right) })),
      kpi(t('scroll'), U.formatDistance(s.scrollMeters, unit()).text, 'scrollMeters'),
      kpi(t('activeTime'), U.formatMinutes(s.activeMinutes), 'activeMinutes'),
      typingStrip(s),
    );

    renderTrend(d);
    C.keyboardHeatmap(document.getElementById('keyboard'), d.keys);
    renderShortcuts(d);
    renderApps(d);
    C.gridHeatmap(document.getElementById('cursor-map'), d);
    renderSide(d);
  }

  function typingStrip(s) {
    const item = (label, value, title) => el('div', { class: 'ts-item', title: title || '' },
      el('span', { class: 'label', text: label }), el('strong', { text: value }));
    return el('div', { class: 'kpi typing-strip' },
      el('p', { class: 'ts-title', text: t('typing') }),
      item(t('speed'), s.wpm == null ? '–' : t('wpm', { n: Math.round(s.wpm) })),
      item(t('best'), s.bestWpm ? t('wpm', { n: s.bestWpm }) : '–', t('bestTitle')),
      item('Backspace', s.backspaceRatio != null ? U.formatPercent(s.backspaceRatio) : '–', t('backspaceTitle')),
      item(t('characters'), U.formatCount(s.charsTyped)));
  }

  /** Active-minute series: whole hours on the axis once values pass 2 h. */
  function minuteScale(items) {
    const max = Math.max(0, ...items.map((i) => i.value));
    if (max >= 120) {
      return {
        items: items.map((i) => ({ ...i, value: i.value / 60 })),
        tick: (v) => (v === 0 ? '0' : `${U.formatNumber(v, Number.isInteger(v) ? 0 : 1)} ${U.minuteUnits().h}`),
      };
    }
    return { items, tick: (v) => (v === 0 ? '0' : `${Math.round(v)} ${U.minuteUnits().min}`) };
  }

  function hourItems(hourly) {
    return hourly.map((v, h) => ({
      label: UI.hourTick(h),
      value: v,
      tip: `<b>${UI.hourText(h)}</b><br>${U.formatMinutes(v)}`,
    }));
  }

  function emptyChart(host) {
    host.textContent = '';
    host.append(el('div', { class: 'empty-chart', text: t('noData') }));
  }

  function renderTrend(d) {
    const host = document.getElementById('trend-chart');
    const tableHost = document.getElementById('trend-table');
    const isToday = d.range === 'today';
    document.getElementById('metric').hidden = isToday;
    document.getElementById('trend-title').textContent = t(isToday ? 'hours' : 'days');
    const m = METRICS[state.metric];

    let items;
    let tick;
    if (isToday) {
      ({ items, tick } = minuteScale(hourItems(d.hourly)));
    } else {
      items = d.series.map((p, i) => ({
        label: d.series.length > 14 ? UI.shortDate(p.date) : `${UI.weekday(p.date)} ${UI.shortDate(p.date)}`,
        value: p[state.metric],
        emphasis: i === d.series.length - 1,
        tip: `<b>${UI.longDate(p.date)}</b><br>${m.fmt(p[state.metric])}`,
      }));
      tick = m.tick;
      if (state.metric === 'activeMinutes') ({ items, tick } = minuteScale(items));
    }
    if (items.every((i) => i.value === 0)) emptyChart(host);
    else C.barChart(host, { items, tickFormat: tick, ariaLabel: t(isToday ? 'hours' : m.title), labelEvery: isToday ? 3 : undefined });

    tableHost.hidden = !state.showTable;
    host.hidden = state.showTable;
    document.getElementById('toggle-table').textContent = t(state.showTable ? 'chart' : 'table');
    if (state.showTable) {
      const rows = isToday
        ? d.hourly.map((v, h) => [UI.hourText(h), U.formatMinutes(v)])
        : d.series.slice().reverse().map((p) => [UI.longDate(p.date), U.formatDistance(p.mouseMeters, unit()).text,
          U.formatCount(p.keystrokes), U.formatCount(p.clicks), U.formatMinutes(p.activeMinutes)]);
      const head = (isToday ? ['hour', 'activeTime'] : ['date', 'distance', 'keyboard', 'clicks', 'activeTime']).map((k) => t(k));
      tableHost.textContent = '';
      tableHost.append(el('table', {},
        el('thead', {}, el('tr', {}, head.map((h) => el('th', { text: h })))),
        el('tbody', {}, rows.map((r) => el('tr', {}, r.map((c) => el('td', { text: c })))))));
    }
  }

  function renderSide(d) {
    const host = document.getElementById('side-chart');
    host.textContent = '';
    if (d.range === 'today') {
      document.getElementById('side-title').textContent = t('topKeys');
      const top = d.summary.topKeys;
      if (!top.length) { emptyChart(host); return; }
      C.barChart(host, {
        items: top.map((k) => ({ label: KB.labelFor(k.code), value: k.count, tip: `<b>${UI.escapeHtml(KB.nameFor(k.code))}</b>: ${U.formatCount(k.count)}` })),
        tickFormat: (v) => U.formatCount(v), ariaLabel: t('topKeys'), labelEvery: 1,
      });
      return;
    }
    document.getElementById('side-title').textContent = t('hours');
    if (d.hourly.every((v) => v === 0)) { emptyChart(host); return; }
    const scaled = minuteScale(hourItems(d.hourly));
    C.barChart(host, { items: scaled.items, tickFormat: scaled.tick, ariaLabel: t('hours'), labelEvery: 3 });
  }

  function comboNode(id) {
    const label = SC.comboLabel(id);
    if (KB.PC) {
      return el('span', { class: 'combo', 'aria-label': label }, label.split('+').map((part) => el('kbd', { text: part })));
    }
    const mods = label.match(/^[⌃⌥⇧⌘]*/)[0];
    const key = label.slice(mods.length);
    return el('span', { class: 'combo', 'aria-label': label }, [...mods].map((m) => el('kbd', { text: m })), el('kbd', { text: key }));
  }

  function renderShortcuts(d) {
    const host = document.getElementById('shortcuts');
    const s = d.summary;
    document.getElementById('shortcuts-sub').textContent = s.shortcutTotal ? UI.count('times', s.shortcutTotal, U.formatCount(s.shortcutTotal)) : '';
    if (!s.topShortcuts.length) {
      host.textContent = '';
      host.append(el('div', { class: 'empty-block' }, el('p', { text: t('shortcutsEmpty', { ex: KB.PC ? 'Ctrl+C, Ctrl+V' : '⌘C, ⌘V' }) })));
      return;
    }
    C.rankList(host, s.topShortcuts.map((x) => {
      const name = SC.nameFor(x.id);
      return {
        label: el('span', { class: 'rank-label' }, comboNode(x.id), name ? el('span', { class: 'rank-sub', text: name }) : null),
        value: x.count,
        display: U.formatCount(x.count),
        tip: `<b>${UI.escapeHtml(SC.comboLabel(x.id))}</b>${name ? ` ${UI.escapeHtml(name)}` : ''}: ${U.formatCount(x.count)}`,
      };
    }), t('shortcuts'));
  }

  function renderApps(d) {
    const host = document.getElementById('apps');
    host.textContent = '';
    const s = d.summary;
    if (!d.settings.trackApps && !s.apps.length) {
      host.append(el('div', { class: 'empty-block' },
        el('p', { text: t('appsIntro') }),
        el('button', { class: 'btn', text: t('turnOn'), onclick: () => api.updateSettings({ trackApps: true }) })));
      return;
    }
    if (!s.apps.length) {
      host.append(el('div', { class: 'empty-block' }, el('p', { text: t('collecting') })));
      return;
    }
    C.rankList(host, s.apps.slice(0, 8).map((a) => ({
      label: el('span', { class: 'rank-label' }, el('span', { class: 'app-name', text: a.name })),
      value: a.minutes,
      display: U.formatMinutes(a.minutes),
      tip: `<b>${UI.escapeHtml(a.name)}</b><br>${U.formatMinutes(a.minutes)}<br>${t('appTip', { k: U.formatCount(a.keys), c: U.formatCount(a.clicks) })}`,
    })), t('apps'));
  }

  // ------------------------------------------------------------- settings
  const SOURCE_TEXT = { hardware: 'sourceHardware', diagonal: 'sourceDiagonal', estimate: 'sourceEstimate' };

  function group(title, ...children) {
    return el('section', { class: 'set-group' }, el('h2', { text: title }), ...children);
  }

  function segmented(label, options, value, onPick) {
    const g = el('div', { class: 'segmented', role: 'group', 'aria-label': label });
    for (const [key, text] of options) {
      g.append(el('button', { text, 'aria-pressed': String(value === key), onclick: () => onPick(key) }));
    }
    return g;
  }

  function choiceGrid(name, refs, value, preview, onPick) {
    const grid = el('div', { class: 'choices', role: 'radiogroup', 'aria-label': name });
    for (const r of [{ key: 'auto', label: t('auto') }, ...refs]) {
      const p = preview(r.key);
      grid.append(el('label', { class: 'choice' },
        el('input', { type: 'radio', name, value: r.key, checked: value === r.key, onchange: () => onPick(r.key) }),
        el('span', { text: r.label }),
        el('small', { text: p ? p.text.replace(/^≈ /, '') : '' })));
    }
    return grid;
  }

  function renderSettings(d) {
    const s = d.settings;
    const host = document.getElementById('settings');
    host.textContent = '';
    const wrap = el('div', { class: 'settings' });
    const set = (partial) => api.updateSettings(partial);

    wrap.append(group(t('language'),
      segmented(t('language'), [['auto', t('languageSystem')], ...['uz', 'en', 'ru'].map((c) => [c, window.OdomouseI18n.NATIVE_NAMES[c]])],
        s.language || 'auto', (k) => set({ language: k, languageChosen: true }))));

    wrap.append(group(t('appearance'),
      segmented(t('appearance'), [['system', t('themeSystem')], ['light', t('themeLight')], ['dark', t('themeDark')]], s.theme, (k) => set({ theme: k }))));

    wrap.append(group(t('clock'),
      segmented(t('clock'), [['auto', t('auto')], ['24', t('clock24')], ['12', t('clock12')]], s.clock || 'auto', (k) => set({ clock: k })),
      el('p', { class: 'note', text: `${UI.hourText(9)} · ${UI.hourText(21)}` })));

    wrap.append(group(t('units'),
      segmented(t('units'), U.UNIT_OPTIONS.map((o) => [o.key, o.key]), s.unit, (k) => set({ unit: k }))));

    const meters = Math.max(d.summary.mouseMeters, 120);
    const chars = Math.max(d.summary.charsTyped, 4000);
    wrap.append(group(t('comparisons'),
      el('p', { class: 'label', text: t('distance') }),
      choiceGrid('cmpDist', Fun.DISTANCE_REFS, s.compareDistance, (k) => Fun.distanceComparison(meters, k), (k) => set({ compareDistance: k })),
      el('p', { class: 'label', style: 'margin-top:14px', text: t('textTyped') }),
      choiceGrid('cmpText', Fun.TEXT_REFS, s.compareText, (k) => Fun.textComparison(chars, k), (k) => set({ compareText: k }))));

    const trayGroup = group(trayName(),
      segmented(trayName(), [['distance', t('distance')], ['keys', t('keyboard')], ['icon', t('iconOnly')]], s.trayShows, (k) => set({ trayShows: k })));
    if (PLATFORM === 'windows') {
      trayGroup.append(el('p', { class: 'note', text: t('trayHoverNote') }));
    }
    if (NATIVE) {
      trayGroup.append(
        el('label', { class: 'check', style: 'margin-top:12px' },
          el('input', { type: 'checkbox', checked: !!s.launchAtLogin, onchange: (e) => set({ launchAtLogin: e.target.checked }) }), t('launchAtLogin')));
    }
    wrap.append(trayGroup);

    wrap.append(group(t('breakReminder'),
      el('div', { class: 'break-row' },
        el('label', { class: 'check' },
          el('input', { type: 'checkbox', checked: s.breakReminder, onchange: (e) => set({ breakReminder: e.target.checked }) }), t('remind')),
        segmented(t('duration'), [25, 50, 60, 90].map((m) => [m, `${m} ${U.minuteUnits().min}`]), s.breakMinutes, (k) => set({ breakMinutes: k }))),
      el('p', { class: 'note', text: t('breakNote') })));

    const appsStatus = el('span', { class: 'inline-status', role: 'status' });
    wrap.append(group(t('apps'),
      el('label', { class: 'check' },
        el('input', { type: 'checkbox', checked: s.trackApps, onchange: (e) => set({ trackApps: e.target.checked }) }), t('trackApps')),
      el('p', { class: 'note', text: t('appNameOnly') }),
      el('div', { class: 'row-actions', style: 'margin-top:10px' },
        el('button', { class: 'btn', text: t('deleteAppData'), onclick: () => api.clearApps().then((r) => { if (r.ok) appsStatus.textContent = t('deleted'); }) }),
        appsStatus)));

    const displaysBox = group(t('displays'), el('p', { class: 'note', style: 'margin:0 0 4px', text: t('displaysNote') }));
    for (const disp of d.displays) {
      const input = el('input', { type: 'number', min: '5', max: '120', step: '0.1', value: disp.diagonalInches.toFixed(1), 'aria-label': t('inches', { label: disp.label }) });
      const status = el('span', { class: 'inline-status' });
      const save = () => {
        const v = Number(input.value);
        if (!(v >= 5 && v <= 120)) { status.textContent = '5–120'; return; }
        set({ diagonalOverrides: { ...s.diagonalOverrides, [disp.id]: v } });
      };
      const reset = () => {
        const next = { ...s.diagonalOverrides };
        delete next[disp.id];
        set({ diagonalOverrides: next });
      };
      displaysBox.append(el('div', { class: 'display-row' },
        el('div', {},
          el('h3', {}, disp.label, el('span', { class: `src ${disp.source}`, text: t(SOURCE_TEXT[disp.source]) })),
          el('p', { text: `${disp.diagonalInches.toFixed(1)}″, ${disp.width} × ${disp.height}` })),
        el('div', { class: 'diag' },
          input, el('span', { text: '″' }),
          el('button', { class: 'btn', text: t('save'), onclick: save }),
          s.diagonalOverrides[disp.id] ? el('button', { class: 'link-btn', text: t('auto'), onclick: reset }) : null,
          status)));
    }
    wrap.append(displaysBox);

    const dataStatus = el('span', { class: 'inline-status', role: 'status' });
    wrap.append(group(t('data'),
      el('div', { class: 'row-actions' },
        el('button', { class: 'btn', text: t('exportCsv'), onclick: () => api.exportCsv().then((r) => { dataStatus.textContent = r.ok ? t('saved') : ''; }) }),
        el('button', { class: 'btn danger', text: t('deleteAll'), onclick: () => api.resetData().then((r) => { if (r.ok) { dataStatus.textContent = t('deleted'); load(); } }) }),
        dataStatus),
      el('p', { class: 'note', text: t('privacyNote', { device: device() }) })));

    wrap.append(group(t('help'),
      el('div', { class: 'row-actions' },
        el('button', { class: 'btn', text: t('showGuide'), onclick: () => set({ onboardingDone: false, seenDashboard: false }) }),
        PLATFORM === 'windows' && d.hooksRunning ? null
          : el('button', { class: 'btn', text: t(d.hooksRunning ? 'permissions' : 'givePermission'), onclick: () => api.openPermissions() }))));

    host.append(wrap);
  }

  // -------------------------------------------------------------- loading
  let loading = false;
  async function load() {
    if (loading) return;
    loading = true;
    try {
      const d = await api.getDashboard(state.range);
      state.data = d;
      state.settings = d.settings;
      UI.applySettings(d.settings);
      UI.languagePicker(d.settings, api);
      if (state.tab === 'stats') renderStats(d); else renderSettings(d);
    } finally {
      loading = false;
    }
  }

  setInterval(() => {
    if (document.visibilityState === 'visible' && state.tab === 'stats' && !document.querySelector('.tooltip.show')) load();
  }, 10000);

  for (const b of document.querySelectorAll('.nav-item')) b.addEventListener('click', () => showTab(b.dataset.tab));
  for (const b of document.querySelectorAll('#range button')) b.addEventListener('click', () => { state.range = b.dataset.range; load(); });
  for (const b of document.querySelectorAll('#metric button')) b.addEventListener('click', () => { state.metric = b.dataset.metric; if (state.data) renderStats(state.data); });
  document.getElementById('toggle-table').addEventListener('click', () => { state.showTable = !state.showTable; if (state.data) renderTrend(state.data); });

  api.onTab((tab) => showTab(tab));
  api.onSettings((s) => { state.settings = s; load(); });
  api.onDisplays(() => { if (state.tab === 'settings') load(); });

  window.OdomouseWrapped.init(api);
  showTab(state.tab);
})();
