/* Odomouse install guide: a small "video" made of HTML. A pretend screen
   (macOS, Windows or Linux) where a cursor shows every click of the first
   install, step by step, with the same steps written next to it. It plays
   when it comes into view, pauses when it leaves, and any step can be
   picked by hand. With "reduce motion" it does not start by itself.

   window.OdomouseGuide.mount(host, { os, lang, autoplay }) -> player
   player.setLang(lang), player.setOs(os), player.pause(), player.play() */
(function () {
  'use strict';

  // ------------------------------------------------------------- texts
  const TX = {
    uz: {
      play: "Ko'rsatish", pause: "To'xtatish", replay: 'Qaytadan', step: '{n}-qadam / {m}',
      today: 'Bugun', keys: 'Tugmalar', clicks: 'Kliklar',
      windows: [
        ['Faylni saqlang', "Brauzer «isn't commonly downloaded» deb ogohlantirsa, Keep (Saqlash) ni tanlang va faylni oching."],
        ['SmartScreen', "«Windows protected your PC» oynasida avval More info, keyin Run anyway ni bosing. Bu oyna ilova hali pullik sertifikat bilan imzolanmagani uchun chiqadi."],
        ["O'rnatish", "Tilni tanlab OK, keyin Next va Install ni bosing. Administrator paroli so'ralmaydi. Oxirida Finish."],
        ['Tayyor', "Bugungi hisob vazifalar panelida, soat yonida paydo bo'ladi. Windows'da boshqa ruxsat kerak emas. Uni bossangiz, kun xulosasi ochiladi."],
      ],
      mac: [
        ["Applications'ga o'tkazing", "Odomouse.dmg ni oching va Odomouse belgisini Applications papkasiga sudrab tashlang."],
        ['Birinchi ochish', "Applications'dan Odomouse'ni oching. «Odomouse Not Opened» chiqsa, Done ni bosing: bu kutilgan holat."],
        ['Open Anyway', "System Settings → Privacy & Security'ni oching, pastga tushib Open Anyway ni bosing va parol yoki Touch ID bilan tasdiqlang. macOS 14 va undan eskisida ilovani o'ng tugma bilan bosib Open ni tanlash ham yetadi."],
        ['Klaviatura ruxsati', "«Odomouse would like to receive keystrokes» oynasida Open System Settings ni bosing va Input Monitoring ro'yxatida Odomouse'ni yoqing. Hisoblagich menu bar'da paydo bo'ladi."],
      ],
      linux: [
        ["O'rnatish", "odomouse_amd64.deb ni ikki marta bosing, Install ni bosing va parolingizni kiriting. Terminalda: sudo apt install ./odomouse_amd64.deb (Ubuntu 22.04, Mint 21, Debian 12 yoki yangirog'i)."],
        ['Ochish', "Ilovalar ro'yxatidan Odomouse'ni toping va oching. Hisoblagich paneldagi belgi yonida paydo bo'ladi."],
        ["Wayland bo'lsa", "Wayland'da klaviaturani sanash uchun bir marta bajaring: sudo usermod -aG input $USER, keyin tizimdan chiqib qayta kiring. X11'da bu kerak emas."],
      ],
    },
    en: {
      play: 'Play', pause: 'Pause', replay: 'Replay', step: 'Step {n} of {m}',
      today: 'Today', keys: 'Keys', clicks: 'Clicks',
      windows: [
        ['Keep the file', 'If the browser says the file "isn\'t commonly downloaded", choose Keep, then open it.'],
        ['SmartScreen', 'In "Windows protected your PC", click More info, then Run anyway. It appears because the app is not signed with a paid certificate yet.'],
        ['Install', 'Pick a language and click OK, then Next and Install. No administrator password is needed. Finish at the end.'],
        ['Done', "Today's count appears on the taskbar next to the clock. Windows needs no other permission. Click it for a look at your day."],
      ],
      mac: [
        ['Copy to Applications', 'Open Odomouse.dmg and drag the Odomouse icon onto the Applications folder.'],
        ['First open', 'Open Odomouse from Applications. If macOS says "Odomouse Not Opened", click Done: that is expected.'],
        ['Open Anyway', 'Open System Settings → Privacy & Security, scroll down, click Open Anyway and confirm with your password or Touch ID. On macOS 14 and older, right-clicking the app and choosing Open is enough.'],
        ['Keyboard access', 'When asked "Odomouse would like to receive keystrokes", click Open System Settings and turn Odomouse on under Input Monitoring. The counter appears in the menu bar.'],
      ],
      linux: [
        ['Install', 'Double-click odomouse_amd64.deb, click Install and enter your password. In a terminal: sudo apt install ./odomouse_amd64.deb (Ubuntu 22.04, Mint 21, Debian 12 or newer).'],
        ['Open', 'Find Odomouse in your apps and open it. The counter appears next to its icon on the panel.'],
        ['On Wayland', 'To count keys on Wayland, run once: sudo usermod -aG input $USER, then log out and back in. Not needed on X11.'],
      ],
    },
    ru: {
      play: 'Показать', pause: 'Пауза', replay: 'Сначала', step: 'Шаг {n} из {m}',
      today: 'Сегодня', keys: 'Клавиши', clicks: 'Клики',
      windows: [
        ['Сохраните файл', 'Если браузер пишет, что файл «isn\'t commonly downloaded», выберите Keep (Сохранить) и откройте его.'],
        ['SmartScreen', 'В окне «Windows protected your PC» нажмите More info, затем Run anyway. Оно появляется, потому что приложение пока не подписано платным сертификатом.'],
        ['Установка', 'Выберите язык и нажмите OK, затем Next и Install. Пароль администратора не нужен. В конце — Finish.'],
        ['Готово', 'Счёт за сегодня появится на панели задач рядом с часами. Других разрешений в Windows не нужно. Нажмите на него, чтобы увидеть сводку дня.'],
      ],
      mac: [
        ['Перенесите в Applications', 'Откройте Odomouse.dmg и перетащите значок Odomouse на папку Applications.'],
        ['Первый запуск', 'Откройте Odomouse из Applications. Если macOS пишет «Odomouse Not Opened», нажмите Done: так и должно быть.'],
        ['Open Anyway', 'Откройте System Settings → Privacy & Security, прокрутите вниз, нажмите Open Anyway и подтвердите паролем или Touch ID. В macOS 14 и старше достаточно нажать на приложение правой кнопкой и выбрать Open.'],
        ['Доступ к клавиатуре', 'В окне «Odomouse would like to receive keystrokes» нажмите Open System Settings и включите Odomouse в списке Input Monitoring. Счётчик появится в строке меню.'],
      ],
      linux: [
        ['Установка', 'Дважды нажмите на odomouse_amd64.deb, нажмите Install и введите пароль. В терминале: sudo apt install ./odomouse_amd64.deb (Ubuntu 22.04, Mint 21, Debian 12 или новее).'],
        ['Запуск', 'Найдите Odomouse в списке приложений и откройте. Счётчик появится рядом со значком на панели.'],
        ['Если Wayland', 'Чтобы считать клавиши в Wayland, один раз выполните: sudo usermod -aG input $USER, затем выйдите из системы и войдите снова. В X11 не нужно.'],
      ],
    },
  };

  const ICON = 'img/icon-256.png';
  const STOP = Symbol('stop');
  const ease = (p) => (p < 0.5 ? 4 * p * p * p : 1 - Math.pow(-2 * p + 2, 3) / 2);
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);

  // ------------------------------------------------- pretend desktops
  const CLOCK = '9:41';
  const desk = {
    mac: (inner, extra = '') => `
      <div class="g-desk g-mac">
        <div class="g-menubar"><span class="g-apple"></span><b>Finder</b><span>File</span><span>Edit</span><span>View</span>
          <span class="g-spacer"></span><span class="g-mb-item" data-g="menubar-odo" hidden><img src="${ICON}" alt=""><span data-g="menubar-num">0 m</span></span>
          <span class="g-mb-dots"></span><span>${CLOCK}</span></div>
        ${inner}
        <div class="g-dock"><i></i><i></i><i></i><i></i><i></i><i class="g-dock-odo" data-g="dock-odo" hidden><img src="${ICON}" alt=""></i></div>
        ${extra}
      </div>`,
    win: (inner, extra = '') => `
      <div class="g-desk g-win">
        ${inner}
        <div class="g-taskbar">
          <span class="g-tb-icons"><i class="g-start"></i><i></i><i class="g-tb-browser"></i><i></i><i></i></span>
          <span class="g-tb-right"><span class="g-counter" data-g="counter" hidden><img src="${ICON}" alt=""><span data-g="counter-num">0 m</span></span>
          <span class="g-tb-tray">^</span><span class="g-tb-clock">${CLOCK}<small>22.10.2026</small></span></span>
        </div>
        ${extra}
      </div>`,
    linux: (inner, extra = '') => `
      <div class="g-desk g-linux">
        <div class="g-panel"><span>Activities</span><span class="g-spacer"></span><span>${CLOCK}</span><span class="g-spacer"></span>
          <span class="g-mb-item" data-g="panel-odo" hidden><img src="${ICON}" alt=""><span data-g="panel-num">0 m</span></span><span class="g-panel-sys" data-g="panel-sys"></span></div>
        ${inner}
        ${extra}
      </div>`,
  };
  const macWin = (title, body, cls = '') => `
    <div class="g-window g-macwin ${cls}"><div class="g-titlebar"><span class="g-lights"><i></i><i></i><i></i></span><span>${title}</span></div>${body}</div>`;
  const winWin = (title, body, cls = '') => `
    <div class="g-window g-winwin ${cls}"><div class="g-titlebar"><span>${title}</span><span class="g-wctl"><i>–</i><i>□</i><i>✕</i></span></div>${body}</div>`;
  const gnomeWin = (title, body, cls = '') => `
    <div class="g-window g-gnomewin ${cls}"><div class="g-titlebar"><span>${title}</span><span class="g-gctl">✕</span></div>${body}</div>`;
  const folder = (label, g = '') => `<div class="g-file" ${g ? `data-g="${g}"` : ''}><span class="g-folder"></span><span>${label}</span></div>`;
  const appIcon = (label, g = '') => `<div class="g-file" ${g ? `data-g="${g}"` : ''}><img class="g-appicon" src="${ICON}" alt=""><span>${label}</span></div>`;
  const blank = (hue) => `<div class="g-file"><span class="g-blankapp" style="--h:${hue}"></span><span>&nbsp;</span></div>`;

  // ----------------------------------------------------------- scripts
  // Each step builds its scene, then moves the cursor. `d` = its length (ms).
  const SCRIPTS = {
    mac: [
      { d: 7000, async run(g) {
        g.scene(desk.mac(macWin('Odomouse', `
          <div class="g-dmg">${appIcon('Odomouse', 'app')}<span class="g-arrow"></span>${folder('Applications', 'apps')}</div>`, 'g-center g-w56')));
        g.cursor(70, 78);
        await g.wait(500);
        await g.move('app', 900);
        await g.drag('app', 'apps', 1400);
        await g.wait(300);
        g.flash('apps');
        await g.wait(1200);
      } },
      { d: 7800, async run(g) {
        g.scene(desk.mac(macWin('Applications', `
          <div class="g-grid">${blank(210)}${blank(15)}${blank(140)}${appIcon('Odomouse', 'app')}${blank(280)}${blank(45)}${blank(190)}${blank(330)}</div>`, 'g-center g-w60')));
        g.cursor(30, 85);
        await g.wait(400);
        await g.move('app', 900);
        await g.dblclick('app');
        await g.wait(250);
        g.overlay(`<div class="g-alert g-macalert" data-g="alert">
          <span class="g-alert-icon"><img src="${ICON}" alt=""><i class="g-warn">!</i></span>
          <b>“Odomouse” Not Opened</b>
          <p>Apple could not verify “Odomouse” is free of malware that may harm your Mac or compromise your privacy.</p>
          <span class="g-btn g-blue" data-g="done">Done</span><span class="g-btn">Move to Trash</span></div>`);
        await g.wait(1300);
        await g.move('done', 800);
        await g.click('done');
        g.hide('alert');
        await g.wait(900);
      } },
      { d: 11000, async run(g) {
        const side = ['Wi‑Fi', 'Bluetooth', 'Network', 'General', 'Appearance', 'Desktop & Dock', 'Displays', 'Privacy & Security', 'Touch ID & Password']
          .map((s) => `<span ${s === 'Privacy & Security' ? 'data-g="privacy"' : ''}>${s}</span>`).join('');
        const rows = ['Location Services', 'Contacts', 'Calendars', 'Photos', 'Input Monitoring', 'Full Disk Access'].map((s) => `<span class="g-row">${s}<i>›</i></span>`).join('');
        g.scene(desk.mac(macWin('System Settings', `
          <div class="g-settings"><div class="g-side">${side}</div>
            <div class="g-pane"><div class="g-scroll" data-g="scroll">
              <h4>Privacy &amp; Security</h4><div class="g-rows">${rows}</div>
              <h5>Security</h5>
              <div class="g-rows"><span class="g-row">Allow applications from <em>App Store &amp; Known Developers</em></span>
              <span class="g-row g-blocked">“Odomouse” was blocked to protect your Mac.<span class="g-btn" data-g="anyway">Open Anyway</span></span></div>
            </div></div></div>`, 'g-center g-w72 g-h70')));
        g.cursor(20, 80);
        await g.wait(400);
        await g.move('privacy', 900);
        await g.click('privacy');
        g.add('privacy', 'g-sel');
        await g.wait(300);
        await g.scroll('scroll', 0.48, 1000);
        await g.move('anyway', 900);
        await g.click('anyway');
        await g.wait(250);
        g.overlay(`<div class="g-alert g-macalert" data-g="alert">
          <span class="g-alert-icon"><img src="${ICON}" alt=""><i class="g-warn">!</i></span>
          <b>Open “Odomouse”?</b>
          <p>Only open it if you trust where it came from.</p>
          <span class="g-btn g-blue" data-g="open">Open Anyway</span><span class="g-btn">Done</span></div>`);
        await g.wait(900);
        await g.move('open', 700);
        await g.click('open');
        g.hide('alert');
        g.overlay(`<div class="g-alert g-macalert g-auth" data-g="auth">
          <span class="g-touchid"></span><b>Privacy &amp; Security</b><p>Touch ID or enter your password to allow this.</p>
          <span class="g-field" data-g="pw"></span><span class="g-btn g-blue" data-g="ok">OK</span></div>`);
        await g.type('pw', '••••••••', 900);
        await g.move('ok', 600);
        await g.click('ok');
        g.hide('auth');
        await g.wait(700);
      } },
      { d: 10500, async run(g) {
        g.scene(desk.mac(''));
        g.cursor(50, 75);
        await g.wait(300);
        g.overlay(`<div class="g-alert g-macalert" data-g="alert">
          <span class="g-alert-icon"><img src="${ICON}" alt=""></span>
          <b>“Odomouse” would like to receive keystrokes from any application.</b>
          <p>Grant access to this application in Privacy &amp; Security settings, located in System Settings.</p>
          <span class="g-btn g-blue" data-g="open">Open System Settings</span><span class="g-btn">Deny</span></div>`);
        await g.wait(1200);
        await g.move('open', 800);
        await g.click('open');
        g.hide('alert');
        await g.wait(200);
        g.overlay(macWin('Input Monitoring', `
          <div class="g-pane g-pad"><h4>Input Monitoring</h4><p class="g-small">Allow the applications below to monitor input from your keyboard even while using other applications.</p>
          <div class="g-rows"><span class="g-row g-app"><img src="${ICON}" alt="">Odomouse<span class="g-toggle" data-g="toggle"></span></span></div></div>`, 'g-center g-w56'), 'settings');
        await g.wait(500);
        await g.move('toggle', 900);
        await g.click('toggle');
        g.add('toggle', 'g-on');
        await g.wait(700);
        g.hide('settings');
        g.show('menubar-odo');
        g.show('dock-odo');
        await g.move('menubar-odo', 900, 5);
        await g.count('menubar-num', 128.4, 1600);
        await g.wait(600);
      } },
    ],

    windows: [
      { d: 7500, async run(g) {
        g.scene(desk.win(`
          <div class="g-window g-browser g-full"><div class="g-tabs"><span class="g-tab"><img src="${ICON}" alt="">Odomouse</span></div>
            <div class="g-addr"><span>‹</span><span>›</span><span class="g-url">foziljonsap.github.io/odomouse</span><span class="g-dl-ico" data-g="dlicon">⤓</span></div>
            <div class="g-page"><div class="g-page-hero"><i></i><i></i><i></i></div></div></div>`,
        `<div class="g-flyout" data-g="fly"><span class="g-fly-head">Downloads</span>
          <div class="g-fly-item"><span class="g-exe"></span><div><b>Odomouse-Setup.exe</b>
          <p data-g="warn">Odomouse-Setup.exe isn't commonly downloaded. Make sure you trust it before you open it.</p>
          <p data-g="openfile" class="g-link" hidden>Open file</p></div></div>
          <div class="g-fly-btns" data-g="keepbtns"><span class="g-btn g-blue" data-g="keep">Keep</span><span class="g-btn">Delete</span></div></div>`));
        g.cursor(45, 60);
        await g.wait(900);
        await g.move('keep', 1000);
        await g.click('keep');
        g.hide('warn'); g.hide('keepbtns'); g.show('openfile');
        await g.wait(600);
        await g.move('openfile', 700);
        await g.click('openfile');
        await g.wait(900);
      } },
      { d: 6800, async run(g) {
        g.scene(desk.win('', `
          <div class="g-smartscreen" data-g="ss"><span class="g-ss-x">✕</span>
            <b>Windows protected your PC</b>
            <p>Microsoft Defender SmartScreen prevented an unrecognized app from starting. Running this app might put your PC at risk.</p>
            <p class="g-link" data-g="more">More info</p>
            <p data-g="details" hidden>App: Odomouse-Setup.exe<br>Publisher: Unknown publisher</p>
            <span class="g-ss-btns"><span class="g-btn" data-g="run" hidden>Run anyway</span><span class="g-btn">Don't run</span></span></div>`));
        g.cursor(70, 30);
        await g.wait(1000);
        await g.move('more', 900);
        await g.click('more');
        g.hide('more'); g.show('details'); g.show('run');
        await g.wait(900);
        await g.move('run', 900);
        await g.click('run');
        g.hide('ss');
        await g.wait(800);
      } },
      { d: 11500, async run(g) {
        g.scene(desk.win('', `
          <div class="g-window g-winwin g-center g-w30 g-lang" data-g="lang"><div class="g-titlebar"><span>Select Setup Language</span></div>
            <div class="g-pad"><p class="g-small">Select the language to use during the installation.</p><span class="g-select">English</span>
            <span class="g-row-end"><span class="g-btn g-blue" data-g="ok">OK</span><span class="g-btn">Cancel</span></span></div></div>`));
        g.cursor(40, 80);
        await g.wait(600);
        await g.move('ok', 800);
        await g.click('ok');
        g.hide('lang');
        const wiz = (title, sub, body, next, g2) => `
          <div class="g-window g-winwin g-center g-w54 g-wizard" data-g="${g2}"><div class="g-titlebar"><span>Setup - Odomouse</span><span class="g-wctl"><i>✕</i></span></div>
            <div class="g-wiz-head"><div><b>${title}</b><p>${sub}</p></div><img src="${ICON}" alt=""></div>
            <div class="g-wiz-body">${body}</div>
            <div class="g-wiz-foot"><span class="g-btn">Back</span><span class="g-btn g-blue" data-g="${g2}-next">${next}</span><span class="g-btn">Cancel</span></div></div>`;
        g.overlay(wiz('Select Additional Tasks', 'Which additional tasks should be performed?', '<span class="g-check">Create a desktop shortcut</span>', 'Next', 'w1'));
        await g.wait(700);
        await g.move('w1-next', 800);
        await g.click('w1-next');
        g.hide('w1', true);
        g.overlay(wiz('Ready to Install', 'Setup is now ready to begin installing Odomouse on your computer.', '<p class="g-small">Click Install to continue with the installation.</p>', 'Install', 'w2'));
        await g.wait(600);
        await g.move('w2-next', 600);
        await g.click('w2-next');
        g.hide('w2', true);
        g.overlay(wiz('Installing', 'Please wait while Setup installs Odomouse on your computer.', '<span class="g-progress"><i data-g="bar"></i></span>', 'Next', 'w3'));
        await g.grow('bar', 1300);
        g.hide('w3', true);
        g.overlay(wiz('Completing the Odomouse Setup Wizard', 'Setup has finished installing Odomouse on your computer.', '<span class="g-check g-checked">Start Odomouse</span>', 'Finish', 'w4'));
        await g.wait(600);
        await g.move('w4-next', 700);
        await g.click('w4-next');
        g.hide('w4');
        await g.wait(600);
      } },
      { d: 8500, async run(g) {
        g.scene(desk.win('', `<div class="g-popup" data-g="popup" hidden>
          <div class="g-pop-head"><img src="${ICON}" alt=""><b>${esc(g.tx.today)}</b></div>
          <div class="g-odo"><i>0</i><i>1</i><i>2</i><i>8</i><i class="g-red">4</i><span>m</span></div>
          <div class="g-pop-stats"><span>${esc(g.tx.keys)}<b>3 512</b></span><span>${esc(g.tx.clicks)}<b>418</b></span></div></div>`));
        g.cursor(50, 50);
        await g.wait(400);
        g.show('counter');
        g.flash('counter');
        await g.count('counter-num', 128.4, 1600);
        await g.move('counter', 1000);
        await g.click('counter');
        g.show('popup');
        await g.wait(2600);
      } },
    ],

    linux: [
      { d: 9500, async run(g) {
        g.scene(desk.linux(gnomeWin('Downloads', `
          <div class="g-grid g-grid-files"><div class="g-file" data-g="deb"><span class="g-deb"></span><span>odomouse_amd64.deb</span></div>${folder('Documents')}${folder('Pictures')}</div>`, 'g-center g-w56')));
        g.cursor(30, 80);
        await g.wait(400);
        await g.move('deb', 900);
        await g.dblclick('deb');
        g.overlay(gnomeWin('Software', `
          <div class="g-sw"><img src="${ICON}" alt=""><div><b>Odomouse</b><p class="g-small">Mouse distance and keystroke counter</p></div>
          <span class="g-btn g-green" data-g="install">Install</span></div>`, 'g-center g-w54'), 'sw');
        await g.wait(700);
        await g.move('install', 800);
        await g.click('install');
        g.overlay(`<div class="g-alert g-gnomealert" data-g="auth"><b>Authentication Required</b><p>Authentication is required to install software.</p>
          <span class="g-field" data-g="pw"></span><span class="g-btn g-blue" data-g="ok">Authenticate</span></div>`);
        await g.type('pw', '••••••••', 900);
        await g.move('ok', 600);
        await g.click('ok');
        g.hide('auth');
        g.text('install', '✓');
        await g.wait(900);
      } },
      { d: 7500, async run(g) {
        g.scene(desk.linux(`<div class="g-overview"><span class="g-search" data-g="search"></span>
          <div class="g-grid g-grid-apps" data-g="results" hidden>${appIcon('Odomouse', 'app')}</div></div>`));
        g.cursor(50, 80);
        await g.wait(400);
        await g.type('search', 'odo', 700);
        g.show('results');
        await g.move('app', 900);
        await g.click('app');
        g.hide('results');
        g.show('panel-odo');
        await g.move('panel-odo', 900, 5);
        await g.count('panel-num', 128.4, 1600);
        await g.wait(500);
      } },
      { d: 9000, async run(g) {
        g.scene(desk.linux(gnomeWin('Terminal', '<pre class="g-term"><span class="g-prompt">you@pc:~$ </span><span data-g="cmd"></span>\n<span data-g="sudo" hidden>[sudo] password for you: </span></pre>', 'g-center g-w60 g-dark'),
          `<div class="g-sysmenu" data-g="sysmenu" hidden><span>Settings</span><span>Lock</span><span data-g="logout">Log Out…</span><span>Power Off…</span></div>`));
        g.cursor(85, 85);
        await g.wait(500);
        await g.type('cmd', 'sudo usermod -aG input $USER', 1800);
        await g.wait(300);
        g.show('sudo');
        await g.wait(900);
        await g.move('panel-sys', 900, 2);
        await g.click('panel-sys');
        g.show('sysmenu');
        await g.move('logout', 700);
        await g.click('logout');
        await g.wait(800);
      } },
    ],
  };

  // ------------------------------------------------------------- player
  let uid = 0;
  class Player {
    constructor(host, opts) {
      this.host = host;
      this.os = opts.os in SCRIPTS ? opts.os : 'windows';
      this.lang = TX[opts.lang] ? opts.lang : 'en';
      this.autoplay = opts.autoplay !== false && !window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      this.token = 0;
      this.paused = !this.autoplay;
      this.cur = 0;
      this.elapsed = 0;
      this.visible = false;
      this.id = `g${++uid}`;
      this.build();
      if ('IntersectionObserver' in window) {
        this.io = new IntersectionObserver((es) => {
          for (const e of es) {
            this.visible = e.isIntersecting;
            if (!this.visible) this.setPaused(true, true);
            else if (this.autoplay && !this.userPaused) this.setPaused(false);
          }
        }, { threshold: 0.35 });
        this.io.observe(this.root);
      }
      this.start(0);
    }

    get tx() { return TX[this.lang]; }
    get steps() { return this.tx[this.os]; }

    build() {
      this.host.textContent = '';
      const root = document.createElement('div');
      root.className = 'guide';
      root.innerHTML = `
        <div class="g-stage" aria-hidden="true"><div class="g-screen"></div><div class="g-overlays"></div>
          <span class="g-cursor"><svg viewBox="0 0 20 28"><path d="M2 2v21l5.5-5.2 3.6 8.2 3.6-1.6-3.6-8h7.6z"/></svg></span>
          <button class="g-big" type="button" tabindex="-1"><svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg></button></div>
        <div class="g-bar"><button class="g-pp" type="button"></button><div class="g-segs"></div><span class="g-count"></span></div>
        <ol class="g-steps"></ol>`;
      this.host.append(root);
      this.root = root;
      this.stage = root.querySelector('.g-stage');
      this.screen = root.querySelector('.g-screen');
      this.layer = root.querySelector('.g-overlays');
      this.cur$ = root.querySelector('.g-cursor');
      this.pp = root.querySelector('.g-pp');
      this.big = root.querySelector('.g-big');
      this.pp.addEventListener('click', () => this.toggle());
      this.stage.addEventListener('click', () => this.toggle());
      this.renderChrome();
    }

    renderChrome() {
      const segs = this.root.querySelector('.g-segs');
      segs.innerHTML = this.steps.map((_, i) => `<button type="button" class="g-seg" aria-label="${esc(this.tx.step.replace('{n}', i + 1).replace('{m}', this.steps.length))}"><i></i></button>`).join('');
      [...segs.children].forEach((b, i) => b.addEventListener('click', () => this.jump(i)));
      const ol = this.root.querySelector('.g-steps');
      ol.innerHTML = this.steps.map(([title, text]) => `<li><button type="button"><b>${esc(title)}</b><span>${esc(text)}</span></button></li>`).join('');
      [...ol.querySelectorAll('button')].forEach((b, i) => b.addEventListener('click', () => this.jump(i, true)));
      this.updateChrome();
    }

    updateChrome() {
      const segs = this.root.querySelectorAll('.g-seg');
      segs.forEach((s, i) => {
        s.classList.toggle('done', i < this.cur);
        s.classList.toggle('now', i === this.cur);
        s.firstChild.style.width = i < this.cur ? '100%' : i > this.cur ? '0%' : `${Math.min(100, (this.elapsed / SCRIPTS[this.os][i].d) * 100)}%`;
      });
      this.root.querySelectorAll('.g-steps li').forEach((li, i) => {
        li.classList.toggle('now', i === this.cur);
        if (i === this.cur) li.setAttribute('aria-current', 'step'); else li.removeAttribute('aria-current');
      });
      this.root.querySelector('.g-count').textContent = this.tx.step.replace('{n}', this.cur + 1).replace('{m}', this.steps.length);
      this.pp.textContent = this.paused ? `▶ ${this.tx.play}` : `❚❚ ${this.tx.pause}`;
      this.pp.setAttribute('aria-pressed', String(!this.paused));
      this.root.classList.toggle('paused', this.paused);
    }

    setPaused(p, auto) {
      this.paused = p;
      if (!auto) this.userPaused = p;
      this.updateChrome();
    }
    toggle() {
      if (this.ended) { this.ended = false; this.setPaused(false); this.start(0); return; }
      this.setPaused(!this.paused);
    }
    play() { this.setPaused(false); }
    pause() { this.setPaused(true); }
    jump(i, focusPlay) {
      this.start(i);
      if (focusPlay || this.paused) this.setPaused(false);
    }
    setLang(lang) { if (TX[lang] && lang !== this.lang) { this.lang = lang; this.renderChrome(); this.start(this.cur); } }
    setOs(os) { if (SCRIPTS[os] && os !== this.os) { this.os = os; this.renderChrome(); this.start(0); } }

    async start(from) {
      const tok = ++this.token;
      this.ended = false;
      for (let i = from; i < SCRIPTS[this.os].length; i++) {
        this.cur = i;
        this.elapsed = 0;
        this.updateChrome();
        try {
          await SCRIPTS[this.os][i].run(this.api(tok));
          await this.api(tok).wait(Math.max(0, SCRIPTS[this.os][i].d - this.elapsed));
        } catch (e) {
          if (e === STOP) return;
          throw e;
        }
        if (tok !== this.token) return;
      }
      // the end: pause on the last frame, then start over a little later
      this.elapsed = SCRIPTS[this.os][this.cur].d;
      this.updateChrome();
      try { await this.api(tok).wait(2500); } catch (e) { return; }
      if (tok === this.token) this.start(0);
    }

    // the commands a step can use
    api(tok) {
      const P = this;
      const q = (name) => P.stage.querySelector(`[data-g="${name}"]`);
      const tick = (ms, fn) => new Promise((resolve, reject) => {
        let left = ms;
        let last = performance.now();
        const step = () => {
          if (tok !== P.token) return reject(STOP);
          const now = performance.now();
          if (!P.paused) {
            const dt = Math.min(now - last, 100);
            left -= dt;
            P.elapsed += dt;
            P.frameChrome();
          }
          last = now;
          if (fn) fn(ease(Math.min(1, Math.max(0, 1 - left / ms))));
          if (left <= 0) resolve();
          else if (P.paused) setTimeout(step, 150); // idle while paused or out of view
          else requestAnimationFrame(step);
        };
        step();
      });
      const center = (el) => {
        const s = P.stage.getBoundingClientRect();
        const r = el.getBoundingClientRect();
        return [((r.left + r.width / 2 - s.left) / s.width) * 100, ((r.top + r.height / 2 - s.top) / s.height) * 100];
      };
      const placeCursor = (x, y) => { P.cx = x; P.cy = y; P.cur$.style.left = `${x}%`; P.cur$.style.top = `${y}%`; };
      const moveTo = (x, y, ms) => {
        const [x0, y0] = [P.cx ?? 50, P.cy ?? 50];
        return tick(ms, (p) => placeCursor(x0 + (x - x0) * p, y0 + (y - y0) * p));
      };
      const ripple = () => {
        const r = document.createElement('span');
        r.className = 'g-ripple';
        r.style.left = `${P.cx}%`;
        r.style.top = `${P.cy}%`;
        P.stage.append(r);
        setTimeout(() => r.remove(), 700);
      };
      return {
        tx: P.tx,
        wait: (ms) => tick(ms),
        scene: (html) => { P.screen.innerHTML = html; P.layer.textContent = ''; },
        overlay: (html, name) => {
          const w = document.createElement('div');
          w.className = 'g-ov';
          if (name) w.dataset.g = name;
          w.innerHTML = html;
          P.layer.append(w);
        },
        cursor: (x, y) => placeCursor(x, y),
        // dy: land a little below the target (so the cursor does not cover a number)
        move: (name, ms, dy = 0) => { const el = q(name); if (!el) return tick(ms); const [x, y] = center(el); return moveTo(x, y + dy, ms); },
        click: async (name) => {
          const el = q(name);
          P.cur$.classList.add('down');
          if (el) el.classList.add('g-pressed');
          ripple();
          await tick(180);
          P.cur$.classList.remove('down');
          if (el) el.classList.remove('g-pressed');
          await tick(120);
        },
        dblclick: async (name) => {
          for (let i = 0; i < 2; i++) {
            P.cur$.classList.add('down'); ripple();
            await tick(90);
            P.cur$.classList.remove('down');
            await tick(70);
          }
          const el = q(name);
          if (el) el.classList.add('g-sel');
          await tick(200);
        },
        drag: async (name, target, ms) => {
          const el = q(name); const to = q(target);
          if (!el || !to) return tick(ms);
          const ghost = el.cloneNode(true);
          ghost.removeAttribute('data-g');
          ghost.className += ' g-ghost';
          const s = P.stage.getBoundingClientRect();
          const r = el.getBoundingClientRect();
          ghost.style.left = `${((r.left - s.left) / s.width) * 100}%`;
          ghost.style.top = `${((r.top - s.top) / s.height) * 100}%`;
          ghost.style.width = `${(r.width / s.width) * 100}%`;
          P.stage.append(ghost);
          P.cur$.classList.add('down');
          const [x0, y0] = [P.cx, P.cy];
          const [x1, y1] = center(to);
          const gx = parseFloat(ghost.style.left); const gy = parseFloat(ghost.style.top);
          await tick(ms, (p) => {
            placeCursor(x0 + (x1 - x0) * p, y0 + (y1 - y0) * p);
            ghost.style.left = `${gx + (x1 - x0) * p}%`;
            ghost.style.top = `${gy + (y1 - y0) * p}%`;
            to.classList.toggle('g-target', p > 0.7);
          });
          P.cur$.classList.remove('down');
          ghost.classList.add('g-drop');
          to.classList.remove('g-target');
          setTimeout(() => ghost.remove(), 500);
        },
        scroll: (name, frac, ms) => {
          const el = q(name);
          if (!el) return tick(ms);
          // in % of the scrolled block, so it is right at any size
          const max = Math.max(0, 1 - el.parentElement.clientHeight / el.scrollHeight) * 100;
          const to = Math.min(max, frac * 100);
          return tick(ms, (p) => { el.style.transform = `translateY(${-to * p}%)`; });
        },
        type: (name, text, ms) => {
          const el = q(name);
          return tick(ms, (p) => { if (el) el.textContent = text.slice(0, Math.round(text.length * p)); });
        },
        count: (name, to, ms) => {
          const el = q(name);
          return tick(ms, (p) => { if (el) el.textContent = `${(to * p).toFixed(1)} m`; });
        },
        grow: (name, ms) => { const el = q(name); return tick(ms, (p) => { if (el) el.style.width = `${p * 100}%`; }); },
        show: (name) => { const el = q(name); if (el) { el.hidden = false; el.classList.add('g-in'); } },
        hide: (name, now) => {
          const el = q(name);
          if (!el) return;
          if (now) { el.hidden = true; return; }
          el.classList.add('g-out');
          setTimeout(() => { el.hidden = true; }, 250);
        },
        add: (name, cls) => { const el = q(name); if (el) el.classList.add(cls); },
        text: (name, s) => { const el = q(name); if (el) el.textContent = s; },
        flash: (name) => { const el = q(name); if (el) { el.classList.remove('g-flash'); void el.offsetWidth; el.classList.add('g-flash'); } },
      };
    }

    frameChrome() {
      if (this.chromeQueued) return;
      this.chromeQueued = true;
      requestAnimationFrame(() => { this.chromeQueued = false; this.updateChrome(); });
    }
  }

  window.OdomouseGuide = {
    mount: (host, opts) => new Player(host, opts || {}),
    title: (lang, os, i) => (TX[lang] || TX.en)[os][i][0],
  };
})();
