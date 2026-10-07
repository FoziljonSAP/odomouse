/* odomouse.com: language, the right download for this computer, and a live
   odometer of the visitor's own cursor on this page. No requests anywhere. */
(function () {
  'use strict';

  const T = {
    uz: {
      title: "Odomouse: kursoringiz qancha yo'l bosadi?",
      h1: "Kursoringiz qancha yo'l bosadi?",
      lead: "Sichqonchani qimirlatib ko'ring: hisoblagich shu sahifadagi yo'lni sanayapti. Odomouse esa buni kun bo'yi, barcha ilovalarda sanaydi, yana klaviatura bosishlari, kliklar va yozish tezligini ham.",
      odoIdle: "Sichqonchani sahifa ustida qimirlating",
      odoLive: "Shu sahifada: {cmp}",
      dlWindows: 'Windows uchun yuklab olish',
      dlMac: 'macOS uchun yuklab olish',
      dlLinux: 'Linux uchun yuklab olish',
      others: 'Boshqa tizimlar uchun:',
      promise: "Bepul va ochiq kodli. Raqamlaringiz kompyuteringizdan hech qayerga chiqmaydi.",
      showTitle: "Kun oxirida nimalarni ko'rasiz",
      f1: "Masofa metrda, yana futbol maydoni yoki Eyfel minorasida",
      f2: "Klaviatura issiqlik xaritasi: qaysi tugmalarni ko'p bosasiz",
      f3: "Yozish tezligi, kliklar, faol vaqt va tanaffus eslatmasi",
      f4: "Sevimli klaviatura yorliqlari va qaysi ilovada qancha ishlaganingiz",
      f5: "Do'stlarga ulashish uchun haftalik karta",
      langsNote: "O'zbek, rus va ingliz tillarida. Yorug' va tungi ko'rinish.",
      cardTitle: 'Haftangiz bitta kartada',
      cardText: "Odomouse haftangizni bitta kartaga chizadi: kursor qancha yo'l bosgani, tugmalar, kliklar va eng tez yozishingiz. Uni rasm qilib saqlang yoki to'g'ridan-to'g'ri Telegram'ga joylang.",
      privacyTitle: "Tugmalarni sanaydi. Nima yozganingizni o'qimaydi.",
      privacy1: "Odomouse faqat qaysi tugma bosilganini qayd qiladi: sanash va issiqlik xaritasini bo'yash uchun. Matn, parol yoki ekran rasmlarini hech qachon saqlamaydi.",
      privacy2: "Hisob ochish, reklama yoki internetga yuborish yo'q: barcha raqamlar kompyuteringizdagi bitta faylda turadi. Ularni Sozlamalarda eksport qilishingiz yoki o'chirishingiz mumkin.",
      privacy3: "Kod GPL-3.0 litsenziyasi ostida ochiq, shuning uchun buni istalgan kishi tekshira oladi.",
      codeLink: "Kodni GitHub'da ko'rish",
      firstTitle: 'Birinchi marta ochish',
      firstNote: "Ilova hali Apple yoki Microsoft'ning pullik sertifikati bilan imzolanmagan. Shuning uchun tizim bir marta unga ishonasizmi deb so'raydi.",
      w1: "Odomouse-Setup.exe ni ishga tushirib, Install ni bosing. Administrator paroli so'ralmaydi.",
      w2: "«Windows protected your PC» chiqsa, avval More info, keyin Run anyway ni bosing.",
      w3: "Sichqoncha belgisi soat yonida, ko'pincha ^ ichida paydo bo'ladi. Hisoblagich doim ko'rinib tursin desangiz, uni vazifalar paneliga sudrab chiqaring.",
      m1: "Odomouse.dmg ni oching va Odomouse'ni Applications papkasiga sudrab tashlang.",
      m2: "Applications'dan oching. macOS ilovani tekshira olmadim desa, System Settings → Privacy & Security bo'limida Open Anyway ni bosing.",
      m3: "Klaviaturani sanash uchun Input Monitoring ruxsatini bering: menu bar'dagi belgini o'ng tugma bilan bosib, «Klaviatura uchun ruxsat berish» ni tanlang.",
      l1: "odomouse_amd64.deb ni ikki marta bosing yoki terminalda: sudo apt install ./odomouse_amd64.deb",
      l2: "Ilovalar menyusidan Odomouse'ni oching. Ubuntu 22.04, Linux Mint 21, Debian 12 yoki yangirog'i kerak.",
      l3: "Wayland'da bir marta bajaring: sudo usermod -aG input $USER, so'ng tizimdan chiqib, qayta kiring.",
      foot: 'Odomouse GPL-3.0 litsenziyasi ostidagi bepul dastur.',
      unit: ['banan', 'banan', 'banan'],
    },
    en: {
      title: 'Odomouse: how far does your cursor travel?',
      h1: 'How far does your cursor travel?',
      lead: 'Move your mouse: the counter is measuring this page. Odomouse measures it all day, in every app, along with keystrokes, clicks and typing speed.',
      odoIdle: 'Move the mouse over the page',
      odoLive: 'On this page: {cmp}',
      dlWindows: 'Download for Windows',
      dlMac: 'Download for macOS',
      dlLinux: 'Download for Linux',
      others: 'Also for',
      promise: 'Free and open source. Your numbers never leave your computer.',
      showTitle: 'What you see at the end of the day',
      f1: 'Distance in metres, and in football pitches or Eiffel Towers',
      f2: 'A keyboard heatmap: which keys you press most',
      f3: 'Typing speed, clicks, active time and a break reminder',
      f4: 'Your favourite shortcuts and the apps you spent time in',
      f5: 'A weekly card to share with friends',
      langsNote: 'In English, Russian and Uzbek. Light and dark.',
      cardTitle: 'Your week on one card',
      cardText: 'Odomouse draws your week on one card: how far the cursor went, keys, clicks and your fastest typing. Save it as a picture or paste it straight into a chat.',
      privacyTitle: 'It counts keys. It does not read what you type.',
      privacy1: 'Odomouse notes which key was pressed so it can count it and colour the heatmap. It never stores text, passwords or screenshots.',
      privacy2: 'There is no account, no advertising and nothing is sent anywhere: all the numbers sit in one file on your computer, and you can export or delete them in Settings.',
      privacy3: 'The code is open under the GPL-3.0 licence, so anyone can check this.',
      codeLink: 'Read the code on GitHub',
      firstTitle: 'Opening it for the first time',
      firstNote: 'The app is not signed with a paid Apple or Microsoft certificate yet, so the system asks once whether you trust it.',
      w1: 'Run Odomouse-Setup.exe and click Install. No administrator password is needed.',
      w2: 'If you see "Windows protected your PC", click More info, then Run anyway.',
      w3: 'The mouse icon appears by the clock, often inside the ^ menu. Drag it onto the taskbar to keep the counter in sight.',
      m1: 'Open Odomouse.dmg and drag Odomouse into the Applications folder.',
      m2: 'Open it from Applications. If macOS says it could not verify the app, go to System Settings → Privacy & Security and click Open Anyway.',
      m3: 'To count keys, allow Input Monitoring: right-click the icon in the menu bar and choose "Allow keyboard access".',
      l1: 'Double-click odomouse_amd64.deb, or run: sudo apt install ./odomouse_amd64.deb',
      l2: 'Open Odomouse from the app menu. It needs Ubuntu 22.04, Linux Mint 21, Debian 12 or newer.',
      l3: 'On Wayland, run once: sudo usermod -aG input $USER, then log out and back in.',
      foot: 'Odomouse is free software under the GPL-3.0 licence.',
      unit: ['banana', 'bananas', 'bananas'],
    },
    ru: {
      title: 'Odomouse: сколько проходит ваш курсор?',
      h1: 'Сколько проходит ваш курсор?',
      lead: 'Подвигайте мышью: счётчик меряет путь на этой странице. Odomouse считает его весь день во всех приложениях, а ещё нажатия клавиш, клики и скорость набора.',
      odoIdle: 'Подвигайте мышью над страницей',
      odoLive: 'На этой странице: {cmp}',
      dlWindows: 'Скачать для Windows',
      dlMac: 'Скачать для macOS',
      dlLinux: 'Скачать для Linux',
      others: 'Для других систем:',
      promise: 'Бесплатно и с открытым кодом. Ваши цифры никуда не уходят с компьютера.',
      showTitle: 'Что вы увидите в конце дня',
      f1: 'Расстояние в метрах, а ещё в футбольных полях или Эйфелевых башнях',
      f2: 'Тепловая карта клавиатуры: какие клавиши вы нажимаете чаще',
      f3: 'Скорость набора, клики, активное время и напоминание о перерыве',
      f4: 'Любимые сочетания клавиш и время в каждом приложении',
      f5: 'Карточка недели, чтобы поделиться с друзьями',
      langsNote: 'На русском, английском и узбекском. Светлая и тёмная тема.',
      cardTitle: 'Ваша неделя на одной карточке',
      cardText: 'Odomouse рисует вашу неделю на одной карточке: путь курсора, нажатия, клики и самый быстрый набор. Сохраните её картинкой или вставьте прямо в Telegram.',
      privacyTitle: 'Считает нажатия. Не читает, что вы пишете.',
      privacy1: 'Odomouse отмечает, какая клавиша нажата, чтобы посчитать её и раскрасить тепловую карту. Он никогда не сохраняет текст, пароли или снимки экрана.',
      privacy2: 'Нет аккаунта, рекламы и отправки данных: все цифры лежат в одном файле на вашем компьютере, их можно выгрузить или удалить в Настройках.',
      privacy3: 'Код открыт под лицензией GPL-3.0, так что это может проверить любой.',
      codeLink: 'Код на GitHub',
      firstTitle: 'Первый запуск',
      firstNote: 'Приложение пока не подписано платным сертификатом Apple или Microsoft, поэтому система один раз спросит, доверяете ли вы ему.',
      w1: 'Запустите Odomouse-Setup.exe и нажмите Install. Пароль администратора не нужен.',
      w2: 'Если появится «Windows protected your PC», нажмите More info, затем Run anyway.',
      w3: 'Значок мыши появится возле часов, часто в меню ^. Перетащите его на панель задач, чтобы счётчик всегда был виден.',
      m1: 'Откройте Odomouse.dmg и перетащите Odomouse в папку Applications.',
      m2: 'Откройте его из Applications. Если macOS не смогла проверить приложение, зайдите в System Settings → Privacy & Security и нажмите Open Anyway.',
      m3: 'Чтобы считать клавиши, разрешите Input Monitoring: нажмите правой кнопкой на значок в строке меню и выберите «Разрешить доступ к клавиатуре».',
      l1: 'Дважды нажмите на odomouse_amd64.deb или выполните: sudo apt install ./odomouse_amd64.deb',
      l2: 'Откройте Odomouse из меню приложений. Нужна Ubuntu 22.04, Linux Mint 21, Debian 12 или новее.',
      l3: 'В Wayland один раз выполните: sudo usermod -aG input $USER, затем выйдите из системы и войдите снова.',
      foot: 'Odomouse — свободная программа под лицензией GPL-3.0.',
      // one, few, many / fraction uses "few" (банана)
      unit: ['банан', 'банана', 'бананов'],
    },
  };

  const FILES = {
    windows: 'Odomouse-Setup.exe',
    mac: 'Odomouse.dmg',
    linux: 'odomouse_amd64.deb',
  };
  const OS_NAMES = { windows: 'Windows', mac: 'macOS', linux: 'Linux' };

  // ------------------------------------------------------------ language
  const store = {
    get: (k) => { try { return localStorage.getItem(k); } catch (e) { return null; } },
    set: (k, v) => { try { localStorage.setItem(k, v); } catch (e) { /* private mode */ } },
  };

  function pickLang() {
    const q = new URLSearchParams(location.search).get('lang');
    if (T[q]) return q;
    const saved = store.get('lang');
    if (T[saved]) return saved;
    for (const l of navigator.languages || [navigator.language || '']) {
      const c = String(l).slice(0, 2).toLowerCase();
      if (T[c]) return c;
    }
    return 'en';
  }

  let lang = pickLang();

  // --------------------------------------------------------------- OS
  function detectOs() {
    const ua = navigator.userAgent;
    const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || '';
    if (/Win/i.test(platform) || /Windows/.test(ua)) return 'windows';
    if (/Mac/i.test(platform) || /Mac OS X|iPhone|iPad/.test(ua)) return 'mac';
    if (/Linux|X11|CrOS/i.test(platform + ua) && !/Android/.test(ua)) return 'linux';
    return 'windows';
  }
  const os = detectOs();

  function renderDownload() {
    const t = T[lang];
    const btn = document.getElementById('download');
    btn.href = `download/${FILES[os]}`;
    document.getElementById('download-label').textContent = t[{ windows: 'dlWindows', mac: 'dlMac', linux: 'dlLinux' }[os]];
    document.getElementById('download-file').textContent = FILES[os];
    const others = document.getElementById('other-links');
    others.textContent = '';
    for (const o of ['windows', 'mac', 'linux']) {
      if (o === os) continue;
      const a = document.createElement('a');
      a.className = 'link';
      a.href = `download/${FILES[o]}`;
      a.textContent = OS_NAMES[o];
      others.append(a);
    }
  }

  function applyLang() {
    const t = T[lang];
    document.documentElement.lang = lang;
    document.title = t.title;
    for (const n of document.querySelectorAll('[data-t]')) {
      const v = t[n.dataset.t];
      if (v != null) n.textContent = v;
    }
    for (const img of document.querySelectorAll('img[data-src]')) img.src = img.dataset.src.replace('{lang}', lang);
    for (const b of document.querySelectorAll('.langs button')) b.setAttribute('aria-pressed', String(b.dataset.lang === lang));
    renderDownload();
    renderNote();
  }

  for (const b of document.querySelectorAll('.langs button')) {
    b.addEventListener('click', () => {
      lang = b.dataset.lang;
      store.set('lang', lang);
      applyLang();
    });
  }

  // ------------------------------------------------------------ odometer
  // A CSS pixel is 1/96 inch: the same physical size on any screen, near enough.
  const MM_PER_PX = 25.4 / 96;
  const BANANA_M = 0.18;
  let meters = 0;
  let last = null;

  const wheels = document.getElementById('odo-wheels');
  const cells = [];
  function cell(red) {
    const c = document.createElement('span');
    c.className = 'cell' + (red ? ' red' : '');
    const strip = document.createElement('span');
    strip.className = 'strip';
    for (let i = 0; i <= 9; i++) {
      const d = document.createElement('span');
      d.textContent = String(i);
      strip.append(d);
    }
    c.append(strip);
    c.set = (n) => { strip.style.transform = `translateY(${-n * 10}%)`; };
    return c;
  }
  for (let i = 0; i < 4; i++) cells.push(cell(false));
  const tenths = cell(true);
  wheels.append(...cells);
  const dot = document.createElement('span');
  dot.className = 'dot';
  wheels.append(dot, tenths);

  function renderOdo() {
    const total = Math.min(Math.floor(meters * 10), 99999);
    const int = String(Math.floor(total / 10)).padStart(4, '0').split('').map(Number);
    int.forEach((d, i) => cells[i].set(d));
    tenths.set(total % 10);
    document.getElementById('odo').setAttribute('aria-label', `${(total / 10).toFixed(1)} m`);
  }

  function bananas() {
    const r = meters / BANANA_M;
    const shown = r < 10 ? r.toFixed(1) : String(Math.round(r));
    const u = T[lang].unit;
    let form = u[2];
    if (lang === 'en') form = shown === '1' ? u[0] : u[1];
    if (lang === 'ru') {
      if (shown.includes('.')) form = u[1];
      else {
        const n = Number(shown);
        const d10 = n % 10;
        const d100 = n % 100;
        form = d10 === 1 && d100 !== 11 ? u[0] : (d10 >= 2 && d10 <= 4 && (d100 < 12 || d100 > 14) ? u[1] : u[2]);
      }
    }
    if (lang === 'uz') form = u[0];
    return `≈ ${shown} ${form}`;
  }

  function renderNote() {
    const note = document.getElementById('odo-note');
    note.textContent = meters >= 0.05 ? T[lang].odoLive.replace('{cmp}', bananas()) : T[lang].odoIdle;
  }

  let frame = 0;
  function onMove(e) {
    if (last) {
      const dx = e.clientX - last.x;
      const dy = e.clientY - last.y;
      const d = Math.hypot(dx, dy);
      if (d < 400) meters += (d * MM_PER_PX) / 1000; // ignore jumps (window switches)
    }
    last = { x: e.clientX, y: e.clientY };
    if (!frame) {
      frame = requestAnimationFrame(() => {
        frame = 0;
        renderOdo();
        renderNote();
      });
    }
  }
  window.addEventListener('pointermove', onMove, { passive: true });
  window.addEventListener('pointerleave', () => { last = null; });
  document.addEventListener('visibilitychange', () => { last = null; });

  // -------------------------------------------------------------- OS tabs
  function showOs(which) {
    for (const b of document.querySelectorAll('.os-tabs button')) {
      const on = b.dataset.os === which;
      b.setAttribute('aria-selected', String(on));
      b.tabIndex = on ? 0 : -1;
      document.getElementById(`steps-${b.dataset.os}`).hidden = !on;
    }
  }
  const tabs = [...document.querySelectorAll('.os-tabs button')];
  tabs.forEach((b, i) => {
    b.addEventListener('click', () => showOs(b.dataset.os));
    b.addEventListener('keydown', (e) => {
      if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return;
      const next = tabs[(i + (e.key === 'ArrowRight' ? 1 : tabs.length - 1)) % tabs.length];
      next.focus();
      showOs(next.dataset.os);
    });
  });

  showOs(os);
  renderOdo();
  applyLang();
})();
