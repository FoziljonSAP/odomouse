'use strict';

// The plain-JS modules the shared web UI loads (src/core): units, keyboard,
// shortcuts and comparisons, in all three languages. The counting itself is
// the Rust core (core/), tested with cargo test.
const { test, run, approx, assert } = require('./_harness');

const U = require('../src/core/units');
const KB = require('../src/core/keyboard');
const SC = require('../src/core/shortcuts');
const Fun = require('../src/core/fun');

// ------------------------------------------------------------------ units
test('units: only m, km, ft, mi; each converts exactly', () => {
  assert.deepStrictEqual(U.UNIT_OPTIONS.map((o) => o.key), ['m', 'km', 'ft', 'mi']);
  approx(U.formatDistance(1609.344, 'mi').value, 1, 1e-12);
  approx(U.formatDistance(1, 'ft').value, 3.28084, 1e-5);
  assert.strictEqual(U.formatDistance(1534, 'km').text, '1.53\u00A0km');
  assert.strictEqual(U.formatDistance(2500, 'm').text, '2\u00A0500\u00A0m');
});
test('units: settings from older versions map to a plain unit', () => {
  assert.strictEqual(U.normalizeUnit('auto-metric'), 'm');
  assert.strictEqual(U.normalizeUnit('auto-imperial'), 'ft');
  assert.strictEqual(U.normalizeUnit('yd'), 'ft');
  assert.strictEqual(U.normalizeUnit('parsec'), 'm');
  assert.strictEqual(U.formatDistance(10, 'auto-metric').unit, 'm');
});
test('units: number formatting groups thousands and trims decimals', () => {
  assert.strictEqual(U.formatNumber(1234567.891, 1), '1 234 567.9');
  assert.strictEqual(U.formatNumber(3.456), '3.46');
  assert.strictEqual(U.formatNumber(34.56), '34.6');
  assert.strictEqual(U.formatNumber(345.6), '346');
  assert.strictEqual(U.formatCount(12345), '12 345');
  assert.strictEqual(U.formatMinutes(205), '3 soat 25 daq');
  assert.strictEqual(U.formatMinutes(45), '45 daq');
  assert.strictEqual(U.formatMinutes(120), '2 soat');
  assert.strictEqual(U.formatPercent(0.0423), '4.2%');
});
test('units: unknown unit and negative/NaN input are safe', () => {
  assert.strictEqual(U.formatDistance(-5, 'm').value, 0);
  assert.strictEqual(U.formatDistance(NaN, 'km').text, '0 km');
});
test('units: minutes in every language', () => {
  U.setLanguage('en');
  assert.strictEqual(U.formatMinutes(205), '3\u00A0h 25\u00A0min');
  U.setLanguage('ru');
  assert.strictEqual(U.formatMinutes(45), '45\u00A0мин');
  assert.deepStrictEqual(U.minuteUnits(), { h: 'ч', min: 'мин' });
  U.setLanguage('xx');
  assert.strictEqual(U.formatMinutes(120), '2\u00A0soat');
});

// --------------------------------------------------------------- keyboard
test('keyboard: every keycode in the layout is unique and labelled', () => {
  const seen = new Set();
  for (const row of KB.ROWS) for (const [code] of row) {
    if (code == null) continue;
    assert.ok(!seen.has(code), `duplicate keycode ${code}`);
    seen.add(code);
  }
  assert.strictEqual(KB.labelFor(KB.K.A), 'A');
  assert.strictEqual(KB.labelFor(KB.K.Space), 'space');
  assert.ok(KB.PRINTABLE.has(KB.K.Space));
  assert.ok(!KB.PRINTABLE.has(KB.K.Tab));
  assert.strictEqual(Object.keys(KB.LETTERS).length, 26);
});
test('keyboard: right-hand key names follow the language', () => {
  KB.setLanguage('en');
  assert.ok(KB.nameFor(KB.K.ShiftRight).endsWith('(right)'));
  KB.setLanguage('ru');
  assert.ok(KB.nameFor(KB.K.ShiftRight).endsWith('(правый)'));
  KB.setLanguage('uz');
  assert.ok(KB.nameFor(KB.K.ShiftRight).endsWith("(o'ng)"));
});

// -------------------------------------------------------------- shortcuts
test('shortcuts: labels in Mac order and friendly names', () => {
  assert.strictEqual(SC.comboLabel(SC.idFor(SC.META, KB.K.C)), '⌘C');
  assert.strictEqual(SC.comboLabel(SC.idFor(SC.META | SC.SHIFT, KB.K.D4)), '⇧⌘4');
  assert.strictEqual(SC.comboLabel(SC.idFor(SC.CTRL | SC.ALT | SC.META, KB.K.Space)), '⌃⌥⌘Space');
  assert.strictEqual(SC.nameFor(SC.idFor(SC.META, KB.K.V)), 'Joylashtirish');
  assert.strictEqual(SC.nameFor(SC.idFor(SC.META, KB.K.J)), null);
  assert.ok(SC.isShortcut({ keycode: KB.K.C, metaKey: true }));
  assert.ok(!SC.isShortcut({ keycode: KB.K.C, altKey: true }), 'option+letter types a character');
  assert.ok(!SC.isShortcut({ keycode: KB.K.Meta, metaKey: true }), 'pressing ⌘ alone is not a shortcut');
});
test('shortcuts: names in English and Russian', () => {
  SC.setLanguage('en');
  assert.strictEqual(SC.nameFor(SC.idFor(SC.META, KB.K.C)), 'Copy');
  SC.setLanguage('ru');
  assert.strictEqual(SC.nameFor(SC.idFor(SC.META, KB.K.Z)), 'Отменить');
  SC.setLanguage('uz');
});

// ------------------------------------------------------------ comparisons
test('fun: auto picks the biggest reference that fits at least twice', () => {
  assert.strictEqual(Fun.distanceComparison(342, 'auto').text, '≈ 3.3 futbol maydoni');
  assert.strictEqual(Fun.distanceComparison(86.6, 'auto').text, '≈ 7.2 avtobus');
  assert.strictEqual(Fun.distanceComparison(0.3, 'auto').text, '≈ 1.7 banan');
  assert.strictEqual(Fun.distanceComparison(0, 'auto'), null);
  assert.strictEqual(Fun.textComparison(70000, 'auto').text, '≈ 23 A4 varaq');
});

test('fun: a chosen reference is always used', () => {
  assert.strictEqual(Fun.distanceComparison(342, 'banana').text, '≈ 1\u00A0900 banan');
  approx(Fun.distanceComparison(42195 * 2, 'marathon').ratio, 2, 1e-9);
  assert.strictEqual(Fun.textComparison(9000, 'a4').text, '≈ 3.0 A4 varaq');
  assert.strictEqual(Fun.textComparison(560, 'tweet').text, '≈ 2.0 tvit');
  assert.strictEqual(Fun.distanceComparison(5, 'marathon'), null, 'too small to say anything');
});

test('fun: reference keys are validated', () => {
  assert.ok(Fun.isRefKey(Fun.DISTANCE_REFS, 'auto'));
  assert.ok(Fun.isRefKey(Fun.DISTANCE_REFS, 'football'));
  assert.ok(Fun.isRefKey(Fun.TEXT_REFS, 'a4'));
  assert.ok(!Fun.isRefKey(Fun.TEXT_REFS, 'football'));
});

test('fun: English and Russian forms match the Rust core', () => {
  Fun.setLanguage('en');
  assert.strictEqual(Fun.distanceComparison(350, 'football').text, '≈ 3.3 football pitches');
  assert.strictEqual(Fun.textComparison(12300, 'a4').text, '≈ 4.1 A4 sheets');
  assert.strictEqual(Fun.DISTANCE_REFS[0].label, 'Banana');
  Fun.setLanguage('ru');
  assert.strictEqual(Fun.distanceComparison(350, 'football').text, '≈ 3.3 футбольного поля');
  assert.strictEqual(Fun.distanceComparison(2310, 'football').text, '≈ 22 футбольных поля');
  assert.strictEqual(Fun.distanceComparison(5.4, 'banana').text, '≈ 30 бананов');
  assert.strictEqual(Fun.textComparison(63000, 'a4').text, '≈ 21 лист A4');
  Fun.setLanguage('uz');
});

module.exports = run('units, keyboard, shortcuts, comparisons');
