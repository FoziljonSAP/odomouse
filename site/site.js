/* odomouse.com: the page measures its visitor the way the app measures a
   day: distance (with a dashed road behind the cursor), speed, keys, clicks
   and a little key heatmap, plus road signs to drive past. It also picks the
   language and the right download. No requests anywhere, nothing stored but
   the chosen language. */
(function () {
  'use strict';

  const T = {
    uz: {
      title: "Odomouse: kursoringiz qancha yo'l bosadi?",
      h1: "Kursoringiz qancha yo'l bosadi?",
      lead: "Bu sahifa hozir sizni sanayapti: sichqonchani qimirlating, tugmalarni bosing, bosib ko'ring. Odomouse xuddi shuni kun bo'yi, barcha ilovalarda qiladi.",
      dlTitle: 'Yuklab oling:',
      promise: "Bepul va ochiq kodli. Raqamlaringiz kompyuteringizdan hech qayerga chiqmaydi.",
      panelTitle: 'Siz, shu sahifada',
      odoIdle: 'Sichqonchani qimirlating',
      odoLive: '{cmp}',
      speed: 'Tezlik',
      kmh: 'km/soat',
      top: 'eng tezi {v}',
      keys: 'Tugmalar',
      clicks: 'Kliklar',
      kbNote: "Bir nechta tugmani bosing: ko'p bosilganlari yonadi, xuddi ilovadagi issiqlik xaritasidek.",
      roadTitle: "Yo'lda.",
      roadNext: 'Keyingi belgi: {name}, yana {d}.',
      roadDone: "Hammasidan o'tdingiz. Endi butun kuningizni sanab ko'ring.",
      signs: ['Banan', 'A4 varaq', '1 metr', 'Mashina', 'Avtobus', 'Futbol maydoni'],
      unit: ['banan', 'banan', 'banan'],
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
      againTitle: "Butun kunda qancha bo'larkin?",
      againIdle: "Odomouse buni kun bo'yi, barcha ilovalarda sanaydi va kechqurun ko'rsatadi.",
      againLive: "Siz shu sahifada {dist} yurdingiz, {keys} ta tugma bosdingiz. Odomouse buni kun bo'yi, barcha ilovalarda sanaydi.",
      foot: 'Odomouse GPL-3.0 litsenziyasi ostidagi bepul dastur.',
    },
    en: {
      title: 'Odomouse: how far does your cursor travel?',
      h1: 'How far does your cursor travel?',
      lead: 'This page is counting you right now: move the mouse, press some keys, click around. Odomouse does the same all day, in every app.',
      dlTitle: 'Download:',
      promise: 'Free and open source. Your numbers never leave your computer.',
      panelTitle: 'You, on this page',
      odoIdle: 'Move the mouse',
      odoLive: '{cmp}',
      speed: 'Speed',
      kmh: 'km/h',
      top: 'top {v}',
      keys: 'Keys',
      clicks: 'Clicks',
      kbNote: 'Press a few keys: the busiest ones light up, like the heatmap in the app.',
      roadTitle: 'On the road.',
      roadNext: 'Next sign: {name}, {d} to go.',
      roadDone: "You passed them all. Now try counting a whole day.",
      signs: ['Banana', 'A4 sheet', '1 metre', 'Car', 'Bus', 'Football pitch'],
      unit: ['banana', 'bananas', 'bananas'],
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
      againTitle: 'So how far does it go in a whole day?',
      againIdle: 'Odomouse counts it all day, in every app, and shows you in the evening.',
      againLive: 'On this page you travelled {dist} and pressed {keys} keys. Odomouse counts it all day, in every app.',
      foot: 'Odomouse is free software under the GPL-3.0 licence.',
    },
    ru: {
      title: 'Odomouse: сколько проходит ваш курсор?',
      h1: 'Сколько проходит ваш курсор?',
      lead: 'Эта страница считает вас прямо сейчас: подвигайте мышью, понажимайте клавиши, покликайте. Odomouse делает то же самое весь день во всех приложениях.',
      dlTitle: 'Скачать:',
      promise: 'Бесплатно и с открытым кодом. Ваши цифры никуда не уходят с компьютера.',
      panelTitle: 'Вы, на этой странице',
      odoIdle: 'Подвигайте мышью',
      odoLive: '{cmp}',
      speed: 'Скорость',
      kmh: 'км/ч',
      top: 'максимум {v}',
      keys: 'Клавиши',
      clicks: 'Клики',
      kbNote: 'Нажмите несколько клавиш: самые частые загорятся, как тепловая карта в приложении.',
      roadTitle: 'В пути.',
      roadNext: 'Следующий знак: {name}, ещё {d}.',
      roadDone: 'Вы проехали все знаки. Теперь посчитайте целый день.',
      signs: ['Банан', 'Лист A4', '1 метр', 'Машина', 'Автобус', 'Футбольное поле'],
      unit: ['банан', 'банана', 'бананов'],
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
      againTitle: 'А сколько набежит за целый день?',
      againIdle: 'Odomouse считает это весь день во всех приложениях и показывает вечером.',
      againLive: 'На этой странице вы прошли {dist} и нажали клавиш: {keys}. Odomouse считает это весь день во всех приложениях.',
      foot: 'Odomouse — свободная программа под лицензией GPL-3.0.',
    },
  };

  const FILES = { windows: 'Odomouse-Setup.exe', mac: 'Odomouse.dmg', linux: 'odomouse_amd64.deb' };
  // where the installers are (a preview copy of the page points at the live site)
  const BASE = window.ODOMOUSE_DOWNLOADS || 'download/';
  // road signs, in metres
  const SIGNS = [0.18, 0.297, 1, 4.5, 12, 105];
  const BANANA_M = 0.18;
  // a CSS pixel is 1/96 inch: the same physical size on any screen, near enough
  const MM_PER_PX = 25.4 / 96;

  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const $ = (id) => document.getElementById(id);

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
  const t = (k, vars) => {
    let s = T[lang][k];
    if (vars) for (const [a, b] of Object.entries(vars)) s = s.split(`{${a}}`).join(String(b));
    return s;
  };

  // ------------------------------------------------------------------ OS
  function detectOs() {
    const ua = navigator.userAgent;
    const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || '';
    if (/Win/i.test(platform) || /Windows/.test(ua)) return 'windows';
    if (/Mac/i.test(platform) || /Mac OS X|iPhone|iPad/.test(ua)) return 'mac';
    if (/Linux|X11|CrOS/i.test(platform + ua) && !/Android/.test(ua)) return 'linux';
    return 'windows';
  }
  const os = detectOs();

  function renderDownloads() {
    for (const a of document.querySelectorAll('a.download[data-os]')) a.href = BASE + FILES[a.dataset.os];
  }

  // ------------------------------------------------------------- numbers
  const fmt = (n, d) => {
    const s = n.toFixed(d);
    const [i, f] = s.split('.');
    return i.replace(/\B(?=(\d{3})+(?!\d))/g, ' ') + (f ? '.' + f : '');
  };
  const distText = (m) => (m < 1000 ? `${fmt(m, m < 10 ? 2 : 1)} m` : `${fmt(m / 1000, 2)} km`);

  function bananas(m) {
    const r = m / BANANA_M;
    const shown = r < 10 ? r.toFixed(1) : String(Math.round(r));
    const u = T[lang].unit;
    let form = u[0];
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
    return `≈ ${shown} ${form}`;
  }

  // ------------------------------------------------------------ odometer
  const state = { meters: 0, keys: 0, clicks: 0, speed: 0, top: 0, perKey: {} };

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
  const dot = document.createElement('span');
  dot.className = 'dot';
  $('odo-wheels').append(...cells, dot, tenths);

  // --------------------------------------------------------- key heatmap
  const ROWS = ['QWERTYUIOP', 'ASDFGHJKL', 'ZXCVBNM'];
  const keyEls = {};
  for (const row of ROWS) {
    const r = document.createElement('div');
    r.className = 'kb-row';
    for (const ch of row) {
      const k = document.createElement('span');
      k.className = 'key';
      k.textContent = ch;
      keyEls['Key' + ch] = k;
      r.append(k);
    }
    $('kb').append(r);
  }
  {
    const r = document.createElement('div');
    r.className = 'kb-row';
    const k = document.createElement('span');
    k.className = 'key space';
    keyEls.Space = k;
    r.append(k);
    $('kb').append(r);
  }
  // the app's blue ramp, dark to light
  const RAMP = [[0x2a, 0x2d, 0x34], [0x2b, 0x3a, 0x52], [0x38, 0x5b, 0x8c], [0x4a, 0x73, 0xad], [0x66, 0x90, 0xc8], [0x8a, 0xae, 0xdb], [0xb1, 0xc9, 0xe8], [0xdf, 0xe9, 0xf7]];
  function ramp(x) {
    const p = Math.min(Math.max(x, 0), 1) * (RAMP.length - 1);
    const i = Math.min(Math.floor(p), RAMP.length - 2);
    const f = p - i;
    return RAMP[i].map((v, j) => Math.round(v + (RAMP[i + 1][j] - v) * f));
  }
  function renderKeys() {
    const max = Math.max(1, ...Object.values(state.perKey));
    for (const [code, el] of Object.entries(keyEls)) {
      const n = state.perKey[code] || 0;
      if (!n) continue;
      const c = ramp(0.2 + 0.8 * (n / max));
      el.style.background = `rgb(${c.join(',')})`;
      el.style.color = c[1] > 160 ? '#1a1c21' : '#eef2f8';
    }
  }

  // --------------------------------------------------------------- road
  const track = $('track');
  const L0 = Math.log(0.1);
  const L1 = Math.log(SIGNS[SIGNS.length - 1]);
  const pos = (m) => Math.min(1, Math.max(0, (Math.log(Math.max(m, 0.1)) - L0) / (L1 - L0)));
  const signEls = SIGNS.map((m) => {
    const s = document.createElement('span');
    s.className = 'sign' + (pos(m) > 0.9 ? ' end' : '');
    s.style.left = `${pos(m) * 100}%`;
    const b = document.createElement('b');
    const small = document.createElement('span');
    small.textContent = m < 1 ? `${Math.round(m * 100)} cm` : `${m} m`;
    s.append(b, small);
    track.append(s);
    return s;
  });

  function renderRoad() {
    const p = pos(state.meters) * 100;
    $('track-done').style.width = `${p}%`;
    $('car').style.left = `${p}%`;
    const next = SIGNS.findIndex((m) => state.meters < m);
    SIGNS.forEach((m, i) => {
      signEls[i].classList.toggle('passed', state.meters >= m);
      signEls[i].classList.toggle('next', i === next);
    });
    $('road-next').textContent = next < 0 ? t('roadDone')
      : t('roadNext', { name: T[lang].signs[next].toLowerCase(), d: distText(SIGNS[next] - state.meters) });
  }

  // --------------------------------------------------------------- render
  let frame = 0;
  function render() {
    frame = 0;
    const total = Math.min(Math.floor(state.meters * 10), 99999);
    String(Math.floor(total / 10)).padStart(4, '0').split('').map(Number).forEach((d, i) => cells[i].set(d));
    tenths.set(total % 10);
    $('odo').setAttribute('aria-label', `${(total / 10).toFixed(1)} m`);
    $('odo-note').textContent = state.meters >= 0.05 ? t('odoLive', { cmp: bananas(state.meters) }) : t('odoIdle');
    $('speed').textContent = state.speed.toFixed(1);
    $('speed-bar').style.width = `${Math.min(100, (state.speed / 12) * 100)}%`;
    $('keys').textContent = fmt(state.keys, 0);
    $('clicks').textContent = fmt(state.clicks, 0);
    renderRoad();
    $('again-note').textContent = state.meters >= 0.05 || state.keys
      ? t('againLive', { dist: distText(state.meters), keys: fmt(state.keys, 0) })
      : t('againIdle');
  }
  const schedule = () => { if (!frame) frame = requestAnimationFrame(render); };

  // ---------------------------------------------------- the dashed trail
  const hero = document.querySelector('.night');
  const canvas = $('trail');
  const ctx = canvas.getContext('2d');
  let points = [];
  let drawing = 0;
  function sizeCanvas() {
    const r = hero.getBoundingClientRect();
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = Math.round(r.width * dpr);
    canvas.height = Math.round(r.height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }
  const TRAIL_MS = 2600;
  function drawTrail() {
    drawing = 0;
    const now = performance.now();
    points = points.filter((p) => now - p.t < TRAIL_MS);
    const r = hero.getBoundingClientRect();
    ctx.clearRect(0, 0, r.width, r.height);
    ctx.lineCap = 'round';
    ctx.setLineDash([10, 9]);
    for (let i = 1; i < points.length; i++) {
      const a = points[i - 1];
      const b = points[i];
      if (b.gap) continue;
      const age = (now - b.t) / TRAIL_MS;
      ctx.strokeStyle = `rgba(130, 171, 230, ${0.85 * (1 - age)})`;
      ctx.lineWidth = 4 - age * 2;
      ctx.lineDashOffset = -b.d;
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();
    }
    if (points.length) drawing = requestAnimationFrame(drawTrail);
  }

  // --------------------------------------------------------------- input
  let last = null;
  let along = 0;
  function onMove(e) {
    const now = performance.now();
    if (last) {
      const d = Math.hypot(e.clientX - last.x, e.clientY - last.y);
      const dt = Math.max(1, now - last.t);
      if (d < 400 && dt < 500) {
        const m = (d * MM_PER_PX) / 1000;
        state.meters += m;
        // smoothed speed, km/h
        const inst = (m / (dt / 1000)) * 3.6;
        state.speed = state.speed * 0.8 + inst * 0.2;
        state.top = Math.max(state.top, state.speed);
        along += d;
      }
    }
    last = { x: e.clientX, y: e.clientY, t: now };
    if (!reduceMotion) {
      const r = hero.getBoundingClientRect();
      const inside = e.clientY >= r.top && e.clientY <= r.bottom;
      if (inside) {
        const prev = points[points.length - 1];
        points.push({ x: e.clientX - r.left, y: e.clientY - r.top, t: now, d: along, gap: !prev || now - prev.t > 300 });
        if (!drawing) drawing = requestAnimationFrame(drawTrail);
      }
    }
    schedule();
  }
  // the speed falls back to zero when the mouse rests
  setInterval(() => {
    if (last && performance.now() - last.t > 250 && state.speed > 0.05) {
      state.speed *= 0.6;
      if (state.speed < 0.05) state.speed = 0;
      schedule();
    }
  }, 120);

  window.addEventListener('pointermove', onMove, { passive: true });
  window.addEventListener('pointerleave', () => { last = null; });
  document.addEventListener('visibilitychange', () => { last = null; });
  window.addEventListener('pointerdown', () => { state.clicks += 1; schedule(); }, { passive: true });
  window.addEventListener('keydown', (e) => {
    if (e.repeat) return;
    state.keys += 1;
    if (keyEls[e.code]) {
      state.perKey[e.code] = (state.perKey[e.code] || 0) + 1;
      const el = keyEls[e.code];
      el.classList.add('hit');
      setTimeout(() => el.classList.remove('hit'), 120);
      renderKeys();
    }
    schedule();
  });
  window.addEventListener('resize', sizeCanvas);

  // ---------------------------------------------------------------- tabs
  function showOs(which) {
    for (const b of document.querySelectorAll('.os-tabs button')) {
      const on = b.dataset.os === which;
      b.setAttribute('aria-selected', String(on));
      b.tabIndex = on ? 0 : -1;
      $(`steps-${b.dataset.os}`).hidden = !on;
    }
  }
  const tabs = [...document.querySelectorAll('.os-tabs button')];
  tabs.forEach((b, i) => {
    b.addEventListener('click', () => showOs(b.dataset.os));
    b.addEventListener('keydown', (e) => {
      if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return;
      e.preventDefault();
      const next = tabs[(i + (e.key === 'ArrowRight' ? 1 : tabs.length - 1)) % tabs.length];
      next.focus();
      showOs(next.dataset.os);
    });
  });

  // ------------------------------------------------------------ language
  function applyLang() {
    document.documentElement.lang = lang;
    document.title = t('title');
    for (const n of document.querySelectorAll('[data-t]')) {
      const v = T[lang][n.dataset.t];
      if (typeof v === 'string') n.textContent = v;
    }
    for (const img of document.querySelectorAll('img[data-src]')) img.src = img.dataset.src.replace('{lang}', lang);
    for (const b of document.querySelectorAll('.langs button')) b.setAttribute('aria-pressed', String(b.dataset.lang === lang));
    SIGNS.forEach((_, i) => { signEls[i].querySelector('b').textContent = T[lang].signs[i]; });
    renderDownloads();
    render();
  }
  for (const b of document.querySelectorAll('.langs button')) {
    b.addEventListener('click', () => { lang = b.dataset.lang; store.set('lang', lang); applyLang(); });
  }

  sizeCanvas();
  showOs(os);
  applyLang();
})();
