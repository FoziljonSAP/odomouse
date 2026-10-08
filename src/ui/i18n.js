/*
 * Page texts in three languages -> window.OdomouseI18n.
 *
 *   I18n.use('ru')        switch (also tells units/fun/keyboard/shortcuts)
 *   I18n.t('key', {n: 3}) text with {placeholders}
 *   I18n.apply(document)  fill [data-i18n], [data-i18n-aria], [data-i18n-title]
 *
 * The core (Rust) has the native texts and the pre-formatted payload
 * pieces; this file is only what the pages draw themselves.
 */
(function () {
  'use strict';

  const LANGS = ['uz', 'en', 'ru'];
  const NATIVE_NAMES = { uz: "O'zbekcha", en: 'English', ru: 'Русский' };

  // key: [uz, en, ru]
  const D = {
    // navigation and headings
    sections: ["Bo'limlar", 'Sections', 'Разделы'],
    stats: ['Statistika', 'Statistics', 'Статистика'],
    settings: ['Sozlamalar', 'Settings', 'Настройки'],
    wrappedCard: ['Wrapped karta', 'Wrapped card', 'Карточка Wrapped'],
    period: ['Davr', 'Period', 'Период'],
    today: ['Bugun', 'Today', 'Сегодня'],
    days7: ['7 kun', '7 days', '7 дней'],
    days30: ['30 kun', '30 days', '30 дней'],
    all: ['Hammasi', 'All', 'Всё'],
    distance: ['Masofa', 'Distance', 'Расстояние'],
    days: ['Kunlar', 'Days', 'Дни'],
    hours: ['Soatlar', 'Hours', 'Часы'],
    metric: ["Ko'rsatkich", 'Metric', 'Показатель'],
    keyboard: ['Klaviatura', 'Keyboard', 'Клавиатура'],
    clicks: ['Kliklar', 'Clicks', 'Клики'],
    scroll: ['Scroll', 'Scroll', 'Прокрутка'],
    activeTime: ['Faol vaqt', 'Active time', 'Активное время'],
    table: ['Jadval', 'Table', 'Таблица'],
    chart: ['Grafik', 'Chart', 'График'],
    shortcuts: ['Yorliqlar', 'Shortcuts', 'Сочетания клавиш'],
    apps: ['Ilovalar', 'Apps', 'Приложения'],
    cursorMap: ['Kursor joyi', 'Cursor position', 'Где был курсор'],
    topKeys: ['Top tugmalar', 'Top keys', 'Частые клавиши'],
    topLetters: ['Top harflar', 'Top letters', 'Частые буквы'],
    hour: ['Soat', 'Hour', 'Час'],
    date: ['Sana', 'Date', 'Дата'],
    week: ['Hafta', 'Week', 'Неделя'],
    month: ['Oy', 'Month', 'Месяц'],
    allTime: ['Hammasi', 'All time', 'Всё время'],
    savePng: ['PNG saqlash', 'Save PNG', 'Сохранить PNG'],
    copy: ['Nusxa olish', 'Copy', 'Копировать'],
    close: ['Yopish', 'Close', 'Закрыть'],
    quit: ['Chiqish', 'Quit', 'Выйти'],
    open: ['Ochish', 'Open', 'Открыть'],
    gettingStarted: ['Boshlash', 'Getting started', 'Начало'],
    switchTheme: ['Rejimni almashtirish', 'Switch appearance', 'Сменить тему'],
    lightMode: ["Yorug' rejim", 'Light mode', 'Светлая тема'],
    darkMode: ['Tungi rejim', 'Dark mode', 'Тёмная тема'],
    changeUnit: ['Birlikni almashtirish', 'Change unit', 'Сменить единицу'],

    // notices and guide
    permissionNeeded: ['Ruxsat kerak', 'Permission needed', 'Нужен доступ'],
    grant: ['Berish', 'Allow', 'Разрешить'],
    distanceApprox: ['Masofa taxminiy', 'Distance is approximate', 'Расстояние приблизительное'],
    adjust: ['Sozlash', 'Adjust', 'Настроить'],
    keysNotCounted: ['Klaviatura sanalmayapti', "Keys aren't being counted", 'Клавиши не считаются'],
    check: ['Tekshirish', 'Check', 'Проверить'],
    stepAllow: ['Ruxsat bering', 'Allow access', 'Дайте доступ'],
    stepMove: ['Sichqonchani qimirlating', 'Move the mouse', 'Подвигайте мышью'],
    stepOpen: ['Statistikani oching', 'Open statistics', 'Откройте статистику'],

    // stats
    funDistance: ['Masofa {v}', 'Distance {v}', 'Расстояние {v}'],
    funText: ['Yozilgan matn {v}', 'Text typed {v}', 'Набрано текста {v}'],
    funPeak: ['Eng faol soat {v}', 'Busiest hour {v}', 'Самый активный час {v}'],
    funLongest: ["Eng uzun to'xtovsiz ish {v}", 'Longest stretch without a break {v}', 'Самая долгая работа без перерыва {v}'],
    noDataYet: ["Hali ma'lumot yo'q", 'No data yet', 'Пока нет данных'],
    noData: ["Ma'lumot yo'q", 'No data', 'Нет данных'],
    clicksLeftRight: ["chap {l}, o'ng {r}", 'left {l}, right {r}', 'левая {l}, правая {r}'],
    typing: ['Yozish', 'Typing', 'Набор текста'],
    speed: ['Tezlik', 'Speed', 'Скорость'],
    best: ['Eng tezi', 'Best', 'Рекорд'],
    bestTitle: ['30 soniyalik eng tez yozish', 'Fastest 30 seconds of typing', 'Самые быстрые 30 секунд набора'],
    backspaceTitle: ['Har 100 belgiga nechta Backspace', 'Backspaces per 100 characters', 'Сколько Backspace на 100 символов'],
    characters: ['Belgilar', 'Characters', 'Символы'],
    wpm: ["{n} so'z/daq", '{n} wpm', '{n} сл/мин'],
    fastestTyping: ['Eng tez yozish', 'Fastest typing', 'Самый быстрый набор'],
    shortcutsEmpty: [
      "Hali yo'q. {ex} kabi yorliqlar shu yerda sanaladi.",
      'None yet. Shortcuts like {ex} are counted here.',
      'Пока нет. Здесь считаются сочетания вроде {ex}.',
    ],
    appsIntro: [
      'Qaysi ilovada qancha vaqt ishlaganingiz. Faqat ilova nomi saqlanadi.',
      'How long you worked in each app. Only app names are stored.',
      'Сколько времени вы провели в каждом приложении. Сохраняются только названия.',
    ],
    turnOn: ['Yoqish', 'Turn on', 'Включить'],
    collecting: ["Ma'lumot yig'ilmoqda", 'Collecting data', 'Собираем данные'],
    appTip: ['{k} tugma, {c} klik', '{k} keys, {c} clicks', 'нажатий: {k}, кликов: {c}'],
    keyboardHeatmap: ['Klaviatura issiqlik xaritasi', 'Keyboard heatmap', 'Тепловая карта клавиатуры'],
    cursorHeatmap: ['Kursor issiqlik xaritasi', 'Cursor heatmap', 'Тепловая карта курсора'],
    less: ['Kam', 'Less', 'Меньше'],
    more: ["Ko'p", 'More', 'Больше'],
    lessTime: ['Kam vaqt', 'Less time', 'Меньше времени'],
    moreTime: ["Ko'p vaqt", 'More time', 'Больше времени'],
    timeHere: ['Bu joyda vaqtning <b>{p}%</b>', '<b>{p}%</b> of the time here', 'Здесь <b>{p}%</b> времени'],
    sinceYesterday: ['kechagidan', 'vs. yesterday', 'ко вчера'],
    withoutBreak: ["To'xtovsiz", 'Without a break', 'Без перерыва'],
    breakIn: ['Tanaffusgacha', 'Break in', 'До перерыва'],
    breakTime: ['Tanaffus vaqti', 'Time for a break', 'Пора сделать перерыв'],

    // wrapped
    cursorTraveled: ["Kursorim bosib o'tgan yo'l", 'How far my cursor traveled', 'Путь моего курсора'],
    favShortcut: ['Sevimli yorliq', 'Favorite shortcut', 'Любимое сочетание'],
    savedTo: ['Saqlandi: {p}', 'Saved: {p}', 'Сохранено: {p}'],
    imageCopied: [
      'Rasm nusxalandi. Telegram yoki boshqa ilovaga joylashtiring.',
      'Image copied. Paste it into Telegram or another app.',
      'Картинка скопирована. Вставьте её в Telegram или другое приложение.',
    ],

    // settings
    auto: ['Avto', 'Auto', 'Авто'],
    appearance: ["Ko'rinish", 'Appearance', 'Оформление'],
    themeSystem: ['Tizim', 'System', 'Системная'],
    themeLight: ["Yorug'", 'Light', 'Светлая'],
    themeDark: ['Tungi', 'Dark', 'Тёмная'],
    language: ['Til', 'Language', 'Язык'],
    languageSystem: ['Tizim tili', 'System', 'Как в системе'],
    clock: ['Soat formati', 'Clock', 'Формат времени'],
    clock24: ['24 soat', '24-hour', '24 часа'],
    clock12: ['12 soat (AM/PM)', '12-hour (AM/PM)', '12 часов (AM/PM)'],
    units: ['Birlik', 'Units', 'Единицы'],
    comparisons: ['Taqqoslash', 'Comparisons', 'Сравнения'],
    textTyped: ['Yozilgan matn', 'Text typed', 'Набранный текст'],
    iconOnly: ['Faqat ikonka', 'Icon only', 'Только значок'],
    trayMac: ['Menu bar', 'Menu bar', 'Строка меню'],
    trayWindows: ['Vazifalar paneli', 'Taskbar', 'Панель задач'],
    trayLinux: ['Panel', 'Panel', 'Панель'],
    trayHoverNote: [
      "Raqam vazifalar panelida, soat yonida turadi. Uni sichqoncha bilan istalgan joyga sudrab qo'yish mumkin.",
      'The number sits on the taskbar next to the clock. You can drag it anywhere with the mouse.',
      'Число видно на панели задач рядом с часами. Его можно перетащить мышью в любое место.',
    ],
    launchAtLogin: ['Kompyuter yonganda ishga tushirish', 'Start when the computer starts', 'Запускать при включении компьютера'],
    breakReminder: ['Tanaffus eslatmasi', 'Break reminder', 'Напоминание о перерыве'],
    remind: ['Eslatish', 'Remind me', 'Напоминать'],
    duration: ['Davomiylik', 'Duration', 'Длительность'],
    breakNote: [
      '5 daqiqa sichqoncha va klaviaturaga tegmasangiz, tanaffus hisoblanadi.',
      'Not touching the mouse and keyboard for 5 minutes counts as a break.',
      '5 минут без мыши и клавиатуры считаются перерывом.',
    ],
    trackApps: ['Qaysi ilovada ishlaganimni sanash', 'Count which apps I use', 'Считать, в каких приложениях я работаю'],
    appNameOnly: ['Faqat ilova nomi saqlanadi.', 'Only the app name is stored.', 'Сохраняется только название приложения.'],
    deleteAppData: ["Ilovalar ma'lumotini o'chirish", 'Delete app data', 'Удалить данные приложений'],
    deleted: ["O'chirildi", 'Deleted', 'Удалено'],
    displays: ['Ekranlar', 'Displays', 'Экраны'],
    displaysNote: [
      "Masofa ekran o'lchamidan hisoblanadi. Taxminiy bo'lsa, diagonalni kiriting.",
      "Distance is calculated from the screen size. If it's approximate, enter the diagonal.",
      'Расстояние считается по размеру экрана. Если оно приблизительное, укажите диагональ.',
    ],
    inches: ['{label}, dyuym', '{label}, inches', '{label}, дюймы'],
    save: ['Saqlash', 'Save', 'Сохранить'],
    sourceHardware: ['Aniq', 'Exact', 'Точно'],
    sourceDiagonal: ["Qo'lda", 'Manual', 'Вручную'],
    sourceEstimate: ['Taxminiy', 'Approximate', 'Примерно'],
    data: ["Ma'lumotlar", 'Data', 'Данные'],
    exportCsv: ['CSV eksport', 'Export CSV', 'Экспорт CSV'],
    saved: ['Saqlandi', 'Saved', 'Сохранено'],
    deleteAll: ["Hammasini o'chirish", 'Delete everything', 'Удалить всё'],
    privacyNote: [
      'Hammasi faqat {device}. Yozgan matningiz saqlanmaydi.',
      'Everything stays {device}. What you type is never stored.',
      'Всё хранится только {device}. Набранный текст не сохраняется.',
    ],
    onThisMac: ["shu Mac'da", 'on this Mac', 'на этом Mac'],
    onThisComputer: ['shu kompyuterda', 'on this computer', 'на этом компьютере'],
    help: ['Yordam', 'Help', 'Помощь'],
    showGuide: ["Qo'llanmani qayta ko'rsatish", 'Show the guide again', 'Показать подсказки снова'],
    permissions: ['Ruxsatlar', 'Permissions', 'Разрешения'],
    givePermission: ['Ruxsat berish', 'Give permission', 'Дать разрешение'],

    confirmReset: [
      "Barcha statistika o'chirilsinmi? Bu amalni qaytarib bo'lmaydi.",
      "Delete all statistics? This can't be undone.",
      'Удалить всю статистику? Это действие нельзя отменить.',
    ],
    confirmApps: [
      "Ilovalar bo'yicha statistika o'chirilsinmi? Umumiy statistika qoladi.",
      'Delete per-app statistics? Overall statistics stay.',
      'Удалить статистику по приложениям? Общая статистика останется.',
    ],

    // first run
    chooseLanguage: ['Tilni tanlang', 'Choose a language', 'Выберите язык'],
    changeLater: [
      "Keyinroq Sozlamalarda o'zgartirish mumkin.",
      'You can change it later in Settings.',
      'Потом его можно изменить в Настройках.',
    ],
  };

  // Counted nouns. uz: one form; en: [one, many]; ru: [one, few, many].
  const COUNTED = {
    times: ['marta', ['time', 'times'], ['раз', 'раза', 'раз']],
    activeDays: ['faol kun', ['active day', 'active days'], ['активный день', 'активных дня', 'активных дней']],
  };

  const MONTHS = {
    uz: ['yanvar', 'fevral', 'mart', 'aprel', 'may', 'iyun', 'iyul', 'avgust', 'sentyabr', 'oktyabr', 'noyabr', 'dekabr'],
    en: ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'],
    // genitive: "6 октября"
    ru: ['января', 'февраля', 'марта', 'апреля', 'мая', 'июня', 'июля', 'августа', 'сентября', 'октября', 'ноября', 'декабря'],
  };
  const WEEKDAYS = {
    uz: ['Ya', 'Du', 'Se', 'Ch', 'Pa', 'Ju', 'Sh'],
    en: ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'],
    ru: ['Вс', 'Пн', 'Вт', 'Ср', 'Чт', 'Пт', 'Сб'],
  };

  let LANG = 'uz';
  const idx = () => LANGS.indexOf(LANG);

  function t(key, vars) {
    const entry = D[key];
    let s = entry ? entry[idx()] : key;
    if (vars) for (const [k, v] of Object.entries(vars)) s = s.split(`{${k}}`).join(String(v));
    return s;
  }

  /** Russian form index for a displayed number: 0 one, 1 few, 2 many. */
  function ruIndex(number) {
    const n = Math.floor(Math.abs(Number(String(number).replace(/[^\d.]/g, ''))) || 0);
    const d10 = n % 10;
    const d100 = n % 100;
    if (d10 === 1 && d100 !== 11) return 0;
    if (d10 >= 2 && d10 <= 4 && (d100 < 12 || d100 > 14)) return 1;
    return 2;
  }

  /** "{shown} noun": 3 -> "3 marta" / "3 times" / "3 раза". `shown` is the formatted number. */
  function count(key, n, shown) {
    const [uz, en, ru] = COUNTED[key];
    const s = shown == null ? String(n) : shown;
    if (LANG === 'uz') return `${s} ${uz}`;
    if (LANG === 'en') return `${s} ${Number(n) === 1 ? en[0] : en[1]}`;
    return `${s} ${ru[ruIndex(n)]}`;
  }

  function longDate(d) {
    const day = d.getDate();
    const m = MONTHS[LANG][d.getMonth()];
    if (LANG === 'en') return `${m.slice(0, 3)} ${day}`;
    if (LANG === 'ru') return `${day} ${m}`;
    return `${day}-${m}`;
  }

  function weekday(d) {
    return WEEKDAYS[LANG][d.getDay()];
  }

  function use(lang) {
    LANG = LANGS.includes(lang) ? lang : 'uz';
    document.documentElement.lang = LANG;
    for (const mod of [window.OdomouseUnits, window.OdomouseFun, window.OdomouseKeyboard, window.OdomouseShortcuts]) {
      if (mod && mod.setLanguage) mod.setLanguage(LANG);
    }
    return LANG;
  }

  /** Static texts in the HTML. */
  function apply(root) {
    const r = root || document;
    for (const n of r.querySelectorAll('[data-i18n]')) n.textContent = t(n.dataset.i18n);
    for (const n of r.querySelectorAll('[data-i18n-aria]')) n.setAttribute('aria-label', t(n.dataset.i18nAria));
    for (const n of r.querySelectorAll('[data-i18n-title]')) n.title = t(n.dataset.i18nTitle);
  }

  window.OdomouseI18n = {
    LANGS, NATIVE_NAMES, use, t, count, apply, longDate, weekday,
    lang: () => LANG,
  };
})();
