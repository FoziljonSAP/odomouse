/*
 * Keyboard shortcuts (⌘C, ⇧⌘4 ...). A shortcut is a non-modifier key pressed
 * while ⌘ or ⌃ is held. Option alone is not a shortcut on a Mac: ⌥+letter
 * types special characters. Stored as "mask:keycode" so counts stay small
 * and layout-independent. Universal module -> window.OdomouseShortcuts.
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory(require('./keyboard'));
  else root.OdomouseShortcuts = factory(root.OdomouseKeyboard);
})(typeof self !== 'undefined' ? self : this, function (KB) {
  'use strict';

  const CTRL = 1;
  const ALT = 2;
  const SHIFT = 4;
  const META = 8;
  const K = KB.K;

  function maskOf(e) {
    return (e.ctrlKey ? CTRL : 0) | (e.altKey ? ALT : 0) | (e.shiftKey ? SHIFT : 0) | (e.metaKey ? META : 0);
  }

  function isShortcut(e) {
    return !!(e.metaKey || e.ctrlKey) && !KB.MODIFIERS.has(e.keycode);
  }

  function idFor(mask, code) {
    return `${mask}:${code}`;
  }

  function parseId(id) {
    const [mask, code] = String(id).split(':').map(Number);
    return { mask, code };
  }

  /** Mac: ⌃⌥⇧⌘ then the key ("⇧⌘4"). Windows/Linux: "Ctrl+Shift+Z". */
  function comboLabel(id) {
    const { mask, code } = parseId(id);
    const key = code === K.Space ? 'Space' : KB.labelFor(code);
    if (KB.PC) {
      const parts = [];
      if (mask & CTRL) parts.push('Ctrl');
      if (mask & META) parts.push(KB.SUPER);
      if (mask & ALT) parts.push('Alt');
      if (mask & SHIFT) parts.push('Shift');
      parts.push(key);
      return parts.join('+');
    }
    const mods = (mask & CTRL ? '⌃' : '') + (mask & ALT ? '⌥' : '') + (mask & SHIFT ? '⇧' : '') + (mask & META ? '⌘' : '');
    return mods + key;
  }

  const id = (mask, code) => idFor(mask, code);
  const NAMED = {
    [id(META, K.C)]: ['Nusxa olish', 'Copy', 'Копировать'],
    [id(META, K.V)]: ['Joylashtirish', 'Paste', 'Вставить'],
    [id(META, K.X)]: ['Kesib olish', 'Cut', 'Вырезать'],
    [id(META, K.Z)]: ['Bekor qilish', 'Undo', 'Отменить'],
    [id(META | SHIFT, K.Z)]: ['Qaytarish', 'Redo', 'Повторить'],
    [id(META, K.A)]: ['Hammasini belgilash', 'Select all', 'Выделить всё'],
    [id(META, K.S)]: ['Saqlash', 'Save', 'Сохранить'],
    [id(META, K.F)]: ['Qidirish', 'Find', 'Найти'],
    [id(META, K.T)]: ['Yangi tab', 'New tab', 'Новая вкладка'],
    [id(META, K.W)]: ['Yopish', 'Close', 'Закрыть'],
    [id(META, K.Q)]: ['Ilovadan chiqish', 'Quit app', 'Выйти из приложения'],
    [id(META, K.N)]: ['Yangi oyna', 'New window', 'Новое окно'],
    [id(META, K.R)]: ['Yangilash', 'Reload', 'Обновить'],
    [id(META, K.P)]: ['Chop etish', 'Print', 'Печать'],
    [id(META, K.L)]: ['Manzil satri', 'Address bar', 'Адресная строка'],
    [id(META, K.Tab)]: ['Ilova almashtirish', 'Switch apps', 'Переключить приложение'],
    [id(META, K.Backquote)]: ['Oyna almashtirish', 'Switch windows', 'Переключить окно'],
    [id(META, K.Space)]: ['Spotlight', 'Spotlight', 'Spotlight'],
    [id(META | SHIFT, K.D3)]: ['Ekran skrinshoti', 'Screenshot', 'Снимок экрана'],
    [id(META | SHIFT, K.D4)]: ['Qism skrinshoti', 'Screenshot of an area', 'Снимок области'],
    [id(META | SHIFT, K.D5)]: ['Skrinshot paneli', 'Screenshot toolbar', 'Панель снимков экрана'],
    [id(META, K.Backspace)]: ["Qatorni o'chirish", 'Delete line', 'Удалить строку'],
    [id(META, K.ArrowLeft)]: ['Qator boshiga', 'Start of line', 'В начало строки'],
    [id(META, K.ArrowRight)]: ['Qator oxiriga', 'End of line', 'В конец строки'],
    [id(META, K.Slash)]: ['Izohga aylantirish', 'Toggle comment', 'Закомментировать'],
    [id(CTRL, K.C)]: ["To'xtatish (Terminal)", 'Stop (Terminal)', 'Прервать (Терминал)'],
  };

  // Windows / Linux: Ctrl takes the place of ⌘.
  const PC_NAMED = {
    [id(CTRL, K.C)]: ['Nusxa olish', 'Copy', 'Копировать'],
    [id(CTRL, K.V)]: ['Joylashtirish', 'Paste', 'Вставить'],
    [id(CTRL, K.X)]: ['Kesib olish', 'Cut', 'Вырезать'],
    [id(CTRL, K.Z)]: ['Bekor qilish', 'Undo', 'Отменить'],
    [id(CTRL, K.Y)]: ['Qaytarish', 'Redo', 'Повторить'],
    [id(CTRL | SHIFT, K.Z)]: ['Qaytarish', 'Redo', 'Повторить'],
    [id(CTRL, K.A)]: ['Hammasini belgilash', 'Select all', 'Выделить всё'],
    [id(CTRL, K.S)]: ['Saqlash', 'Save', 'Сохранить'],
    [id(CTRL, K.F)]: ['Qidirish', 'Find', 'Найти'],
    [id(CTRL, K.T)]: ['Yangi tab', 'New tab', 'Новая вкладка'],
    [id(CTRL | SHIFT, K.T)]: ['Yopilgan tabni ochish', 'Reopen closed tab', 'Открыть закрытую вкладку'],
    [id(CTRL, K.W)]: ['Yopish', 'Close', 'Закрыть'],
    [id(CTRL, K.N)]: ['Yangi oyna', 'New window', 'Новое окно'],
    [id(CTRL, K.R)]: ['Yangilash', 'Reload', 'Обновить'],
    [id(CTRL, K.P)]: ['Chop etish', 'Print', 'Печать'],
    [id(CTRL, K.L)]: ['Manzil satri', 'Address bar', 'Адресная строка'],
    [id(CTRL, K.Tab)]: ['Keyingi tab', 'Next tab', 'Следующая вкладка'],
    [id(CTRL, K.Slash)]: ['Izohga aylantirish', 'Toggle comment', 'Закомментировать'],
    [id(CTRL, K.Backspace)]: ["So'zni o'chirish", 'Delete word', 'Удалить слово'],
    [id(CTRL, K.ArrowLeft)]: ["Oldingi so'z", 'Previous word', 'Предыдущее слово'],
    [id(CTRL, K.ArrowRight)]: ["Keyingi so'z", 'Next word', 'Следующее слово'],
    [id(CTRL | SHIFT, K.Escape)]: ['Vazifalar menejeri', 'Task Manager', 'Диспетчер задач'],
    [id(META, K.D)]: ['Ish stoli', 'Desktop', 'Рабочий стол'],
    [id(META, K.E)]: ['Fayllar', 'Files', 'Файлы'],
    [id(META, K.L)]: ['Qulflash', 'Lock', 'Блокировка'],
    [id(META, K.V)]: ['Bufer tarixi', 'Clipboard history', 'Журнал буфера обмена'],
    [id(META | SHIFT, K.S)]: ['Qism skrinshoti', 'Screenshot of an area', 'Снимок области'],
  };

  // [uz, en, ru]
  let LANG_INDEX = 0;
  function setLanguage(lang) {
    LANG_INDEX = Math.max(0, ['uz', 'en', 'ru'].indexOf(lang));
  }

  function nameFor(shortcutId) {
    const names = (KB.PC ? PC_NAMED : NAMED)[shortcutId];
    return names ? names[LANG_INDEX] : null;
  }

  return { CTRL, ALT, SHIFT, META, maskOf, isShortcut, idFor, parseId, comboLabel, nameFor, NAMED, setLanguage };
});
