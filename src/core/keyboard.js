/*
 * Physical keyboard model (MacBook ANSI layout), keyed by libuiohook "VC"
 * keycodes, the ids the core stores (core/src/keyboard.rs). They name
 * *physical* keys, so counts are layout-independent:
 * on a Cyrillic or Uzbek layout the key labelled "Q" here is still the same
 * physical key. Universal module (Node + browser -> window.OdomouseKeyboard).
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory(process.env.ODOMOUSE_PLATFORM);
  else root.OdomouseKeyboard = factory(root.ODOMOUSE_PLATFORM);
})(typeof self !== 'undefined' ? self : this, function (platform) {
  'use strict';

  // 'mac' (default), 'windows' or 'linux'. The
  // native apps set window.ODOMOUSE_PLATFORM before this file loads.
  const PLATFORM = platform === 'windows' || platform === 'linux' ? platform : 'mac';
  const PC = PLATFORM !== 'mac';
  const SUPER = PLATFORM === 'windows' ? 'Win' : 'Super';

  const K = {
    Escape: 1, F1: 59, F2: 60, F3: 61, F4: 62, F5: 63, F6: 64, F7: 65, F8: 66, F9: 67, F10: 68, F11: 87, F12: 88,
    Backquote: 41, D1: 2, D2: 3, D3: 4, D4: 5, D5: 6, D6: 7, D7: 8, D8: 9, D9: 10, D0: 11, Minus: 12, Equal: 13, Backspace: 14,
    Tab: 15, Q: 16, W: 17, E: 18, R: 19, T: 20, Y: 21, U: 22, I: 23, O: 24, P: 25, BracketLeft: 26, BracketRight: 27, Backslash: 43,
    CapsLock: 58, A: 30, S: 31, D: 32, F: 33, G: 34, H: 35, J: 36, K: 37, L: 38, Semicolon: 39, Quote: 40, Enter: 28,
    Shift: 42, Z: 44, X: 45, C: 46, V: 47, B: 48, N: 49, M: 50, Comma: 51, Period: 52, Slash: 53, ShiftRight: 54,
    Ctrl: 29, Alt: 56, Meta: 3675, Space: 57, MetaRight: 3676, AltRight: 3640, CtrlRight: 3613,
    ArrowLeft: 57419, ArrowUp: 57416, ArrowDown: 57424, ArrowRight: 57421,
  };

  // [keycode | null (fn key, not reported), label, width in key units]
  const ROWS = [
    [[K.Escape, 'esc', 1.5], [K.F1, 'F1', 1], [K.F2, 'F2', 1], [K.F3, 'F3', 1], [K.F4, 'F4', 1], [K.F5, 'F5', 1],
      [K.F6, 'F6', 1], [K.F7, 'F7', 1], [K.F8, 'F8', 1], [K.F9, 'F9', 1], [K.F10, 'F10', 1], [K.F11, 'F11', 1], [K.F12, 'F12', 1]],
    [[K.Backquote, '`', 1], [K.D1, '1', 1], [K.D2, '2', 1], [K.D3, '3', 1], [K.D4, '4', 1], [K.D5, '5', 1], [K.D6, '6', 1],
      [K.D7, '7', 1], [K.D8, '8', 1], [K.D9, '9', 1], [K.D0, '0', 1], [K.Minus, '-', 1], [K.Equal, '=', 1], [K.Backspace, '⌫', 1.5]],
    [[K.Tab, '⇥', 1.5], [K.Q, 'Q', 1], [K.W, 'W', 1], [K.E, 'E', 1], [K.R, 'R', 1], [K.T, 'T', 1], [K.Y, 'Y', 1],
      [K.U, 'U', 1], [K.I, 'I', 1], [K.O, 'O', 1], [K.P, 'P', 1], [K.BracketLeft, '[', 1], [K.BracketRight, ']', 1], [K.Backslash, '\\', 1]],
    [[K.CapsLock, '⇪', 1.75], [K.A, 'A', 1], [K.S, 'S', 1], [K.D, 'D', 1], [K.F, 'F', 1], [K.G, 'G', 1], [K.H, 'H', 1],
      [K.J, 'J', 1], [K.K, 'K', 1], [K.L, 'L', 1], [K.Semicolon, ';', 1], [K.Quote, "'", 1], [K.Enter, '↩', 1.75]],
    [[K.Shift, '⇧', 2.25], [K.Z, 'Z', 1], [K.X, 'X', 1], [K.C, 'C', 1], [K.V, 'V', 1], [K.B, 'B', 1], [K.N, 'N', 1],
      [K.M, 'M', 1], [K.Comma, ',', 1], [K.Period, '.', 1], [K.Slash, '/', 1], [K.ShiftRight, '⇧', 2.25]],
    PC
      ? [[K.Ctrl, 'Ctrl', 1.25], [K.Meta, SUPER, 1.25], [K.Alt, 'Alt', 1.25], [K.Space, '', 5],
        [K.AltRight, 'Alt', 1.25], [K.CtrlRight, 'Ctrl', 1.25], [K.ArrowLeft, '←', 1], [K.ArrowUp, '↑', 1],
        [K.ArrowDown, '↓', 1], [K.ArrowRight, '→', 1]]
      : [[null, 'fn', 1], [K.Ctrl, '⌃', 1], [K.Alt, '⌥', 1], [K.Meta, '⌘', 1.25], [K.Space, '', 5],
        [K.MetaRight, '⌘', 1.25], [K.AltRight, '⌥', 1], [K.ArrowLeft, '←', 1], [K.ArrowUp, '↑', 1],
        [K.ArrowDown, '↓', 1], [K.ArrowRight, '→', 1]],
  ];

  const LETTERS = {};
  'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').forEach((ch) => { LETTERS[K[ch]] = ch; });

  // Keys that produce a character when typed (counted for WPM). Enter counts
  // like it does in most typing tests; Tab, arrows and modifiers do not.
  const PRINTABLE = new Set([
    ...Object.keys(LETTERS).map(Number),
    K.D0, K.D1, K.D2, K.D3, K.D4, K.D5, K.D6, K.D7, K.D8, K.D9,
    K.Space, K.Enter, K.Backquote, K.Minus, K.Equal, K.BracketLeft, K.BracketRight,
    K.Backslash, K.Semicolon, K.Quote, K.Comma, K.Period, K.Slash,
  ]);

  const MODIFIERS = new Set([K.Shift, K.ShiftRight, K.Ctrl, K.CtrlRight, K.Alt, K.AltRight, K.Meta, K.MetaRight, K.CapsLock]);

  const LABELS = {};
  ROWS.forEach((row) => row.forEach(([code, label]) => {
    if (code != null && !(code in LABELS)) LABELS[code] = label === '' ? 'space' : label;
  }));

  // keys that exist on a PC but are not drawn on the MacBook layout
  if (PC) {
    Object.assign(LABELS, { [K.MetaRight]: SUPER, [K.CtrlRight]: 'Ctrl', 57427: 'Del', 57415: 'Home', 57423: 'End', 57417: 'PgUp', 57425: 'PgDn' });
  }

  const RIGHT = { uz: "(o'ng)", en: '(right)', ru: '(правый)' };
  let NAMES = {};

  function setLanguage(lang) {
    const r = RIGHT[lang] || RIGHT.uz;
    NAMES = PC ? {
      [K.Escape]: 'Esc', [K.Backspace]: 'Backspace', [K.Tab]: 'Tab', [K.CapsLock]: 'Caps Lock', [K.Enter]: 'Enter',
      [K.Shift]: 'Shift', [K.ShiftRight]: `Shift ${r}`, [K.CtrlRight]: `Ctrl ${r}`, [K.AltRight]: `Alt ${r}`,
      [K.MetaRight]: `${SUPER} ${r}`, [K.Space]: 'Space',
    } : {
      [K.Escape]: 'esc', [K.Backspace]: 'delete', [K.Tab]: 'tab', [K.CapsLock]: 'caps lock', [K.Enter]: 'return',
      [K.Shift]: 'shift', [K.ShiftRight]: `shift ${r}`, [K.Ctrl]: 'control', [K.CtrlRight]: `control ${r}`,
      [K.Alt]: 'option', [K.AltRight]: `option ${r}`, [K.Meta]: 'command', [K.MetaRight]: `command ${r}`, [K.Space]: 'space',
    };
  }
  setLanguage('uz');

  /** Short label as printed on the key: 'E', '⌘', 'space'. */
  function labelFor(code) {
    return LABELS[code] || `#${code}`;
  }

  /** Label plus the key's name where the symbol alone may be unclear: '⌘ command'. */
  function nameFor(code) {
    const label = labelFor(code);
    const name = NAMES[code];
    if (!name || name === label) return label;
    return `${label} ${name}`;
  }

  return { K, ROWS, LETTERS, PRINTABLE, MODIFIERS, PLATFORM, PC, SUPER, labelFor, nameFor, setLanguage };
});
