'use strict';

/*
 * Renders the UI pages in headless Chromium with the fake API and saves
 * screenshots. Fails if a page logs an error or throws. Usage:
 *   node test/ui/shoot.js <outDir> [page...]
 * pages: popup, popup-dark, popup-empty, popup-noperm, dash-stats, ...
 */

const path = require('path');
const fs = require('fs');
const { chromium } = require('playwright');

const UI = path.join(__dirname, '..', '..', 'src', 'ui');
const CORE = path.join(__dirname, '..', '..', 'src', 'core');
// units + fun first, so the fake can format comparisons like the real core
const FAKE = ['units.js', 'fun.js'].map((f) => fs.readFileSync(path.join(CORE, f), 'utf8')).join('\n')
  + '\n' + fs.readFileSync(path.join(__dirname, 'fake-api.js'), 'utf8');

const SHOTS = {
  popup: { file: 'popup.html', w: 360, h: 640, theme: 'light' },
  'popup-dark': { file: 'popup.html', w: 360, h: 640, theme: 'dark' },
  'popup-empty': { file: 'popup.html', w: 360, h: 640, theme: 'light', scenario: 'empty' },
  'popup-noperm': { file: 'popup.html', w: 360, h: 640, theme: 'dark', scenario: 'noperm' },
  'dash-stats': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', full: true },
  'dash-stats-dark': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', full: true },
  'dash-today': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', full: true, action: 'today' },
  'dash-empty': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', full: true, scenario: 'empty' },
  'dash-settings': { file: 'dashboard.html', query: 'tab=settings', w: 1040, h: 760, theme: 'light', full: true },
  'dash-narrow': { file: 'dashboard.html', w: 820, h: 600, theme: 'light', full: true },
  'popup-new': { file: 'popup.html', w: 360, h: 640, theme: 'light', scenario: 'new' },
  'popup-new-dark': { file: 'popup.html', w: 360, h: 640, theme: 'dark', scenario: 'new' },
  'dash-settings-dark': { file: 'dashboard.html', query: 'tab=settings', w: 1040, h: 760, theme: 'dark', full: true },
  'popup-due': { file: 'popup.html', w: 360, h: 640, theme: 'light', scenario: 'due' },
  'dash-appsoff': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', full: true, scenario: 'appsoff' },
  wrapped: { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', action: 'wrapped' },
  'wrapped-light': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', action: 'wrapped' },
  'wrapped-png': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', action: 'wrapped-png' },
  'win-dash-stats': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', full: true, platform: 'windows' },
  'win-dash-settings': { file: 'dashboard.html', query: 'tab=settings', w: 1040, h: 760, theme: 'light', full: true, platform: 'windows' },
  'mac-native-settings': { file: 'dashboard.html', query: 'tab=settings', w: 1040, h: 760, theme: 'dark', full: true, platform: 'mac' },
  'linux-popup': { file: 'popup.html', w: 360, h: 640, theme: 'dark', platform: 'linux' },
  // for the website (site/img): one viewport, not the whole page
  'site-dash-uz': { file: 'dashboard.html', w: 1100, h: 760, theme: 'light', platform: 'mac', lang: 'uz' },
  'site-dash-en': { file: 'dashboard.html', w: 1100, h: 760, theme: 'light', platform: 'mac', lang: 'en' },
  'site-dash-ru': { file: 'dashboard.html', w: 1100, h: 760, theme: 'light', platform: 'mac', lang: 'ru' },
  'site-popup-uz': { file: 'popup.html', w: 360, h: 640, theme: 'dark', platform: 'mac', lang: 'uz' },
  'site-popup-en': { file: 'popup.html', w: 360, h: 640, theme: 'dark', platform: 'mac', lang: 'en' },
  'site-popup-ru': { file: 'popup.html', w: 360, h: 640, theme: 'dark', platform: 'mac', lang: 'ru' },
  'site-card-uz': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', action: 'wrapped-png', platform: 'mac', lang: 'uz' },
  'site-card-en': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', action: 'wrapped-png', platform: 'mac', lang: 'en' },
  'site-card-ru': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', action: 'wrapped-png', platform: 'mac', lang: 'ru' },
  // languages and clock
  'en-dash-stats': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', full: true, platform: 'mac', lang: 'en' },
  'ru-dash-stats': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', full: true, platform: 'windows', lang: 'ru' },
  'ru-dash-today': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', full: true, platform: 'mac', lang: 'ru', action: 'today' },
  'en-dash-today-24': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', full: true, platform: 'mac', lang: 'en', clock: '24', action: 'today' },
  'uz-dash-today-12': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', full: true, platform: 'mac', lang: 'uz', clock: '12', action: 'today' },
  'en-dash-settings': { file: 'dashboard.html', query: 'tab=settings', w: 1040, h: 760, theme: 'light', full: true, platform: 'mac', lang: 'en' },
  'ru-dash-settings': { file: 'dashboard.html', query: 'tab=settings', w: 1040, h: 760, theme: 'dark', full: true, platform: 'linux', lang: 'ru' },
  'en-popup': { file: 'popup.html', w: 360, h: 640, theme: 'light', platform: 'mac', lang: 'en' },
  'ru-popup': { file: 'popup.html', w: 360, h: 640, theme: 'dark', platform: 'windows', lang: 'ru' },
  'ru-popup-new': { file: 'popup.html', w: 360, h: 640, theme: 'light', platform: 'mac', lang: 'ru', scenario: 'new' },
  'en-wrapped-png': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', action: 'wrapped-png', platform: 'mac', lang: 'en' },
  'ru-wrapped-png': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', action: 'wrapped-png', platform: 'mac', lang: 'ru' },
  'first-run-popup': { file: 'popup.html', w: 360, h: 640, theme: 'light', platform: 'mac', sysLang: 'ru', chosen: false, lang: 'auto' },
  'first-run-dash': { file: 'dashboard.html', w: 1040, h: 760, theme: 'dark', platform: 'windows', sysLang: 'en', chosen: false, lang: 'auto' },
  'first-run-picked': { file: 'dashboard.html', w: 1040, h: 760, theme: 'light', platform: 'mac', sysLang: 'en', chosen: false, lang: 'auto', action: 'pick-uz' },
};

async function shoot(browser, name, outDir) {
  const s = SHOTS[name];
  const page = await browser.newPage({ viewport: { width: s.w, height: s.h }, deviceScaleFactor: 2, colorScheme: s.theme });
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  if (s.platform) await page.addInitScript(`window.ODOMOUSE_PLATFORM = ${JSON.stringify(s.platform)}; document.addEventListener('DOMContentLoaded', () => document.documentElement.classList.add('native-chrome'));`);
  const fake = { scenario: s.scenario || 'normal', native: !!s.platform, lang: s.lang, clock: s.clock, sysLang: s.sysLang, chosen: s.chosen };
  await page.addInitScript(`window.__FAKE = ${JSON.stringify(fake)};\n${FAKE}`);
  await page.addInitScript(`document.addEventListener('DOMContentLoaded', () => document.documentElement.setAttribute('data-theme', ${JSON.stringify(s.theme)}));`);
  const url = 'file://' + path.join(UI, s.file) + (s.query ? `?${s.query}` : '');
  await page.goto(url);
  await page.waitForTimeout(500);
  if (s.action === 'today') { await page.click('[data-range="today"]'); await page.waitForTimeout(300); }
  if (s.action === 'pick-uz') { await page.click('.lang-option[lang="uz"]'); await page.waitForTimeout(400); }
  if (s.action === 'wrapped' || s.action === 'wrapped-png') { await page.click('#open-wrapped'); await page.waitForTimeout(600); }
  if (s.action === 'wrapped-png') {
    const dataUrl = await page.evaluate(() => document.querySelector('#wrapped-canvas').toDataURL('image/png'));
    fs.writeFileSync(path.join(outDir, `${name}.png`), Buffer.from(dataUrl.split(',')[1], 'base64'));
  } else {
    await page.screenshot({ path: path.join(outDir, `${name}.png`), fullPage: !!s.full });
  }
  await page.close();
  return errors;
}

(async () => {
  const outDir = process.argv[2];
  const names = process.argv.slice(3).length ? process.argv.slice(3) : Object.keys(SHOTS);
  fs.mkdirSync(outDir, { recursive: true });
  const browser = await chromium.launch();
  let bad = 0;
  for (const n of names) {
    const errs = await shoot(browser, n, outDir);
    console.log(`${errs.length ? '✗' : '✓'} ${n}${errs.length ? '\n    ' + errs.join('\n    ') : ''}`);
    if (errs.length) bad += 1;
  }
  await browser.close();
  process.exit(bad ? 1 : 0);
})();
