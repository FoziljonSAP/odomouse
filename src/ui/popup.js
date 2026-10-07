(function () {
  'use strict';
  const U = window.OdomouseUnits;
  const UI = window.OdomouseUI;
  const { el, t } = UI;
  const api = window.odomouse;

  const SUN = '<svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true"><circle cx="10" cy="10" r="3.6"/><path d="M10 2.5v1.8M10 15.7v1.8M2.5 10h1.8M15.7 10h1.8M4.7 4.7l1.3 1.3M14 14l1.3 1.3M4.7 15.3L6 14M14 6l1.3-1.3"/></svg>';
  const MOON = '<svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" aria-hidden="true"><path d="M16 12.6A6.5 6.5 0 017.4 4a6.5 6.5 0 108.6 8.6z"/></svg>';

  let current = null;
  const odo = UI.createOdometer({
    onUnitClick: () => current && api.updateSettings({ unit: UI.nextUnit(current.settings.unit) }),
  });
  document.getElementById('odo').append(odo.root);

  document.getElementById('open-dash').addEventListener('click', () => api.openDashboard('stats'));
  document.getElementById('open-settings').addEventListener('click', () => api.openDashboard('settings'));
  document.getElementById('quit').addEventListener('click', () => api.quit());
  document.getElementById('guide-close').addEventListener('click', () => api.updateSettings({ onboardingDone: true }));
  document.getElementById('theme').addEventListener('click', () => {
    api.updateSettings({ theme: UI.isDark() ? 'light' : 'dark' });
  });

  function themeButton() {
    const dark = UI.isDark();
    const b = document.getElementById('theme');
    b.innerHTML = dark ? SUN : MOON;
    b.title = t(dark ? 'lightMode' : 'darkMode');
  }

  // ------------------------------------------------------------- guide
  function guide(p) {
    const box = document.getElementById('guide');
    const s = p.settings;
    const steps = [
      { text: t('stepAllow'), done: p.hooksRunning, action: [t('grant'), () => api.openPermissions()] },
      { text: t('stepMove'), done: p.summary.mouseMeters > 0.05 },
      { text: t('stepOpen'), done: s.seenDashboard, action: [t('open'), () => api.openDashboard('stats')] },
    ];
    const allDone = steps.every((x) => x.done);
    if (allDone && !s.onboardingDone) api.updateSettings({ onboardingDone: true });
    const show = !s.onboardingDone && !allDone;
    box.hidden = !show;
    document.querySelector('.pop').classList.toggle('guiding', show);
    if (!show) return false;
    const list = document.getElementById('guide-steps');
    list.textContent = '';
    steps.forEach((st, i) => {
      list.append(el('li', { class: st.done ? 'done' : '' },
        el('span', { class: 'step-mark', text: st.done ? '✓' : String(i + 1) }),
        el('span', { class: 't', text: st.text }),
        !st.done && st.action ? el('button', { class: 'btn', text: st.action[0], onclick: st.action[1] }) : el('span')));
    });
    return true;
  }

  // ------------------------------------------------------------- notices
  function notices(p, guiding) {
    const box = document.getElementById('notices');
    box.textContent = '';
    const add = (text, label, fn, quiet) => box.append(el('div', { class: 'notice' + (quiet ? ' quiet' : '') },
      el('p', { text }), el('button', { class: 'btn', text: label, onclick: fn })));
    if (!p.hooksRunning) {
      if (!guiding) add(t('permissionNeeded'), t('grant'), () => api.openPermissions());
      return;
    }
    const s = p.summary;
    if (s.activeMinutes >= 5 && s.mouseMeters > 2 && s.keystrokes === 0) add(t('keysNotCounted'), t('check'), () => api.openPermissions(), true);
    if (p.estimatedDisplays) add(t('distanceApprox'), t('adjust'), () => api.openDashboard('settings'), true);
  }

  // ------------------------------------------------------------- pieces
  function deltas(p) {
    const cmp = p.compare;
    const set = (id, metric, suffix) => {
      const node = document.getElementById(id);
      const info = cmp ? UI.deltaInfo(cmp.deltas[metric]) : null;
      node.textContent = info ? info.text + (suffix || '') : '';
      node.title = info ? cmp.label : '';
    };
    set('dist-d', 'mouseMeters', ` ${t('sinceYesterday')}`);
    set('keys-d', 'keystrokes');
    set('clicks-d', 'clicks');
    set('scroll-d', 'scrollMeters');
    set('active-d', 'activeMinutes');
  }

  function typing(p) {
    const s = p.summary;
    const box = document.getElementById('typing');
    box.textContent = '';
    box.append(el('span', {}, `${t('typing')} `, el('strong', { class: 'num', text: s.wpm == null ? '–' : t('wpm', { n: Math.round(s.wpm) }) })));
    if (p.comparisons.text) box.append(el('span', { text: p.comparisons.text.text }));
  }

  function session(p) {
    const box = document.getElementById('session');
    const ses = p.session || { minutes: 0, breakIn: null };
    box.hidden = !p.hooksRunning || ses.minutes < 1;
    if (box.hidden) return;
    const st = p.settings;
    const text = document.getElementById('session-text');
    text.textContent = '';
    if (!st.breakReminder) text.append(`${t('withoutBreak')} `, el('strong', { text: U.formatMinutes(ses.minutes) }));
    else if (ses.breakIn > 0) text.append(`${t('breakIn')} `, el('strong', { text: U.formatMinutes(ses.breakIn) }));
    else text.append(el('strong', { text: t('breakTime') }));
    const work = st.breakMinutes || 50;
    document.getElementById('session-bar').style.width = `${Math.min(1, ses.minutes / work) * 100}%`;
    box.classList.toggle('due', st.breakReminder && ses.minutes >= work);
    box.querySelector('.session-meter').hidden = !st.breakReminder;
  }

  function letters(s) {
    const box = document.getElementById('letters');
    box.textContent = '';
    if (!s.topLetters.length) {
      box.append(el('span', { class: 'empty', text: '–' }));
      return;
    }
    const max = s.topLetters[0].count;
    for (const { letter, count } of s.topLetters) {
      const fill = UI.seqColor(0.25 + 0.75 * (count / max));
      box.append(el('div', {
        class: 'keycap',
        style: `background:${fill.css};color:${UI.inkOn(fill)}`,
        title: `${letter}: ${U.formatCount(count)}`,
      }, el('b', { text: letter }), el('span', { text: U.formatCount(count) })));
    }
  }

  function render(p) {
    current = p;
    const s = p.summary;
    const unit = p.settings.unit;
    UI.applySettings(p.settings);
    UI.languagePicker(p.settings, api);
    themeButton();
    document.getElementById('date').textContent = UI.longDate(s.date);
    odo.update(s.mouseMeters, unit);
    document.getElementById('dist-cmp').textContent = p.comparisons.distance ? p.comparisons.distance.text : '';
    document.getElementById('keys').textContent = U.formatCount(s.keystrokes);
    const clicksEl = document.getElementById('clicks');
    clicksEl.textContent = U.formatCount(s.clicks.total);
    clicksEl.title = t('clicksLeftRight', { l: U.formatCount(s.clicks.left), r: U.formatCount(s.clicks.right) });
    document.getElementById('scroll').textContent = U.formatDistance(s.scrollMeters, unit).text;
    document.getElementById('active').textContent = U.formatMinutes(s.activeMinutes);
    const guiding = guide(p);
    notices(p, guiding);
    deltas(p);
    typing(p);
    session(p);
    letters(s);
  }

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', themeButton);
  api.onLive(render);
  api.onSettings(() => api.getLive().then(render));
  api.getLive().then(render);
})();
