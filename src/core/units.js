/*
 * Distance units + number formatting. Universal module: works in Node
 * (tests) and in the browser (popup/dashboard load it with a plain <script>
 * tag and get window.OdomouseUnits).
 *
 * Formatting is hand-rolled instead of Intl so the output is identical to
 * the Rust core (core/src/units.rs): '.' for decimals, a no-break space for
 * thousands ("12 345.6").
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory();
  else root.OdomouseUnits = factory();
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  const METERS_PER = { m: 1, km: 1000, ft: 0.3048, mi: 1609.344 };

  // Four plain units, no "auto" modes: what you pick is what you see.
  const UNIT_OPTIONS = [
    { key: 'm', label: 'Metr' },
    { key: 'km', label: 'Kilometr' },
    { key: 'ft', label: 'Fut' },
    { key: 'mi', label: 'Mil' },
  ];

  // Settings saved by older versions.
  const LEGACY_UNITS = { 'auto-metric': 'm', 'auto-imperial': 'ft', yd: 'ft' };

  function normalizeUnit(key) {
    if (METERS_PER[key]) return key;
    return LEGACY_UNITS[key] || 'm';
  }

  const NBSP = ' ';


  /** Fewer decimals as the number grows: 3.42 / 34.2 / 342. */
  function autoDecimals(value) {
    const a = Math.abs(value);
    if (a === 0) return 0;
    if (a < 10) return 2;
    if (a < 100) return 1;
    return 0;
  }

  function formatNumber(value, decimals) {
    if (!Number.isFinite(value)) value = 0;
    const d = decimals == null ? autoDecimals(value) : decimals;
    const fixed = Math.abs(value).toFixed(d);
    const [intPart, frac] = fixed.split('.');
    const grouped = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, NBSP);
    const sign = value < 0 && Number(fixed) !== 0 ? '-' : '';
    return sign + grouped + (frac ? '.' + frac : '');
  }

  function formatCount(n) {
    return formatNumber(Math.round(n || 0), 0);
  }

  /**
   * @returns {{ value:number, unit:string, number:string, text:string }}
   *   value in the chosen unit, the unit symbol, the formatted number, and
   *   "number unit" ready for display.
   */
  function formatDistance(meters, unitKey, decimals) {
    const m = Math.max(0, meters || 0);
    const unit = normalizeUnit(unitKey);
    const value = m / METERS_PER[unit];
    const number = formatNumber(value, decimals);
    return { value, unit, number, text: number + NBSP + unit };
  }

  // "3 soat 25 daq" / "3 h 25 min" / "3 ч 25 мин"
  const MINUTE_UNITS = { uz: ['soat', 'daq'], en: ['h', 'min'], ru: ['ч', 'мин'] };
  let LANG = 'uz';

  function setLanguage(lang) {
    LANG = MINUTE_UNITS[lang] ? lang : 'uz';
  }

  /** Short units for chart ticks: { h: 'soat', min: 'daq' }. */
  function minuteUnits() {
    const [h, min] = MINUTE_UNITS[LANG];
    return { h, min };
  }

  /** 205 -> "3 soat 25 daq", 45 -> "45 daq", 0 -> "0 daq" (in the current language). */
  function formatMinutes(totalMinutes) {
    const mins = Math.max(0, Math.round(totalMinutes || 0));
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    const [hu, mu] = MINUTE_UNITS[LANG];
    if (h === 0) return `${m}${NBSP}${mu}`;
    if (m === 0) return `${h}${NBSP}${hu}`;
    return `${h}${NBSP}${hu} ${m}${NBSP}${mu}`;
  }

  function formatPercent(ratio, decimals) {
    return formatNumber((ratio || 0) * 100, decimals == null ? 1 : decimals) + '%';
  }

  return {
    METERS_PER,
    UNIT_OPTIONS,
    normalizeUnit,
    formatNumber,
    formatCount,
    formatDistance,
    formatMinutes,
    formatPercent,
    setLanguage,
    minuteUnits,
  };
});
