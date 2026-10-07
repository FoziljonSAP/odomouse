/*
 * "How much is that?" comparisons, with references the user picks in
 * Settings. Universal module (Node + browser -> window.OdomouseFun) so the
 * settings page can preview each option with today's numbers.
 *
 * Reference sizes are rounded public figures. Text sizes: an A4 page of
 * 12 pt single-spaced text holds roughly 3 000 characters, a printed book
 * page roughly 1 800.
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory(require('./units'));
  else root.OdomouseFun = factory(root.OdomouseUnits);
})(typeof self !== 'undefined' ? self : this, function (U) {
  'use strict';

  // label: settings choice [uz, en, ru]; after a number: uz (no plural),
  // en [one, many], ru [one, few, many, fraction] (core/src/fun.rs has the same).
  const DISTANCE_REFS = [
    { key: 'banana', size: 0.18, labels: ['Banan', 'Banana', 'Банан'], uz: 'banan',
      en: ['banana', 'bananas'], ru: ['банан', 'банана', 'бананов', 'банана'] },
    { key: 'a4', size: 0.297, labels: ['A4 varaq', 'A4 sheet', 'Лист A4'], uz: 'A4 varaq',
      en: ['A4 sheet', 'A4 sheets'], ru: ['лист A4', 'листа A4', 'листов A4', 'листа A4'] },
    { key: 'bus', size: 12, labels: ['Avtobus', 'Bus', 'Автобус'], uz: 'avtobus',
      en: ['bus', 'buses'], ru: ['автобус', 'автобуса', 'автобусов', 'автобуса'] },
    { key: 'pool', size: 50, labels: ['Basseyn', 'Pool', 'Бассейн'], uz: 'olimpiya basseyni',
      en: ['Olympic pool', 'Olympic pools'],
      ru: ['олимпийский бассейн', 'олимпийских бассейна', 'олимпийских бассейнов', 'олимпийского бассейна'] },
    { key: 'football', size: 105, labels: ['Futbol maydoni', 'Football pitch', 'Футбольное поле'], uz: 'futbol maydoni',
      en: ['football pitch', 'football pitches'],
      ru: ['футбольное поле', 'футбольных поля', 'футбольных полей', 'футбольного поля'] },
    { key: 'eiffel', size: 330, labels: ['Eyfel minorasi', 'Eiffel Tower', 'Эйфелева башня'], uz: 'Eyfel minorasi',
      en: ['Eiffel Tower', 'Eiffel Towers'],
      ru: ['Эйфелева башня', 'Эйфелевы башни', 'Эйфелевых башен', 'Эйфелевой башни'] },
    { key: 'marathon', size: 42195, labels: ['Marafon', 'Marathon', 'Марафон'], uz: 'marafon',
      en: ['marathon', 'marathons'], ru: ['марафон', 'марафона', 'марафонов', 'марафона'] },
  ];

  const TEXT_REFS = [
    { key: 'sms', size: 160, labels: ['SMS', 'SMS', 'SMS'], uz: 'SMS', en: ['SMS', 'SMS'], ru: ['SMS', 'SMS', 'SMS', 'SMS'] },
    { key: 'tweet', size: 280, labels: ['Tvit', 'Tweet', 'Твит'], uz: 'tvit',
      en: ['tweet', 'tweets'], ru: ['твит', 'твита', 'твитов', 'твита'] },
    { key: 'page', size: 1800, labels: ['Kitob sahifasi', 'Book page', 'Страница книги'], uz: 'kitob sahifasi',
      en: ['book page', 'book pages'], ru: ['страница книги', 'страницы книги', 'страниц книги', 'страницы книги'] },
    { key: 'a4', size: 3000, labels: ['A4 varaq', 'A4 sheet', 'Лист A4'], uz: 'A4 varaq',
      en: ['A4 sheet', 'A4 sheets'], ru: ['лист A4', 'листа A4', 'листов A4', 'листа A4'] },
  ];

  const LANGS = ['uz', 'en', 'ru'];
  let LANG = 'uz';

  function setLanguage(lang) {
    LANG = LANGS.includes(lang) ? lang : 'uz';
    for (const r of [...DISTANCE_REFS, ...TEXT_REFS]) r.label = r.labels[LANGS.indexOf(LANG)];
  }
  setLanguage('uz');

  function unitFor(ref, number) {
    if (LANG === 'en') return number === '1' ? ref.en[0] : ref.en[1];
    if (LANG === 'ru') {
      const digits = number.replace(/[^\d.]/g, '');
      if (digits.includes('.')) return ref.ru[3];
      const n = Number(digits);
      const d10 = n % 10;
      const d100 = n % 100;
      if (d10 === 1 && d100 !== 11) return ref.ru[0];
      if (d10 >= 2 && d10 <= 4 && (d100 < 12 || d100 > 14)) return ref.ru[1];
      return ref.ru[2];
    }
    return ref.uz;
  }

  // "auto": the biggest reference that still fits at least twice
  // ("3.3 football pitches" reads better than "1.0 Eiffel Tower").
  function pick(refs, amount, key) {
    if (key && key !== 'auto') return refs.find((r) => r.key === key) || refs[0];
    let best = refs[0];
    for (const r of refs) if (amount >= 2 * r.size) best = r;
    return best;
  }

  function compare(refs, amount, key) {
    if (!(amount > 0)) return null;
    const ref = pick(refs, amount, key);
    const ratio = amount / ref.size;
    if (ratio < 0.1) return null;
    const number = U.formatNumber(ratio, ratio < 10 ? 1 : 0);
    return { key: ref.key, ratio, text: `≈ ${number} ${unitFor(ref, number)}` };
  }

  /** Cursor distance in metres -> "≈ 3.3 futbol maydoni" (or null). */
  function distanceComparison(meters, key) {
    return compare(DISTANCE_REFS, meters, key);
  }

  /** Characters typed -> "≈ 4.1 A4 varaq" (or null). */
  function textComparison(chars, key) {
    return compare(TEXT_REFS, chars, key);
  }

  function isRefKey(refs, key) {
    return key === 'auto' || refs.some((r) => r.key === key);
  }

  return { DISTANCE_REFS, TEXT_REFS, distanceComparison, textComparison, isRefKey, setLanguage };
});
