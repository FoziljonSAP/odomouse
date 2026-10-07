/* Shareable "Wrapped" card drawn on a 1080x1350 canvas (4:5, fits Instagram
   and Telegram previews) -> window.OdomouseWrapped. The card is always dark:
   it's a poster, not part of the app chrome. */
(function () {
  'use strict';
  const U = window.OdomouseUnits;
  const UI = window.OdomouseUI;

  const SITE = 'odomouse.com';
  const W = 1080;
  const H = 1350;
  const PAD = 84;
  const SANS = '-apple-system, "SF Pro Display", BlinkMacSystemFont, system-ui, "Segoe UI", sans-serif';
  const MONO = '"SF Mono", ui-monospace, Menlo, Consolas, monospace';
  const INK = '#f3f3ef';
  const MUTED = '#9a9ca3';
  const LINE = 'rgba(255,255,255,0.10)';
  const RED = '#d8403f';
  const BLUE_RAMP = ['#1c5cab', '#2a78d6', '#3987e5', '#5598e7', '#86b6ef'];

  function luminance(hex) {
    const n = parseInt(hex.slice(1), 16);
    return (0.2126 * ((n >> 16) & 255) + 0.7152 * ((n >> 8) & 255) + 0.0722 * (n & 255)) / 255;
  }

  function roundRect(ctx, x, y, w, h, r) {
    ctx.beginPath();
    ctx.moveTo(x + r, y);
    ctx.arcTo(x + w, y, x + w, y + h, r);
    ctx.arcTo(x + w, y + h, x, y + h, r);
    ctx.arcTo(x, y + h, x, y, r);
    ctx.arcTo(x, y, x + w, y, r);
    ctx.closePath();
  }

  function text(ctx, str, x, y, font, color, align = 'left') {
    ctx.font = font;
    ctx.fillStyle = color;
    ctx.textAlign = align;
    ctx.textBaseline = 'alphabetic';
    ctx.fillText(str, x, y);
    return ctx.measureText(str).width;
  }

  function fitText(ctx, str, maxW, weight, size, minSize) {
    let s = size;
    ctx.font = `${weight} ${s}px ${SANS}`;
    while (s > minSize && ctx.measureText(str).width > maxW) {
      s -= 2;
      ctx.font = `${weight} ${s}px ${SANS}`;
    }
    return `${weight} ${s}px ${SANS}`;
  }

  function mouseGlyph(ctx, x, y, h) {
    const w = h * 0.62;
    ctx.save();
    ctx.strokeStyle = INK;
    ctx.lineWidth = h * 0.1;
    roundRect(ctx, x, y, w, h, w / 2);
    ctx.stroke();
    ctx.fillStyle = INK;
    roundRect(ctx, x + w / 2 - h * 0.05, y + h * 0.17, h * 0.1, h * 0.2, h * 0.05);
    ctx.fill();
    ctx.restore();
  }

  function odometer(ctx, value, unitLabel, x, y, maxW) {
    const { int, tenths } = UI.odometerDigits(value);
    ctx.font = `600 64px ${SANS}`;
    const unitW = ctx.measureText(unitLabel).width + 28;
    const sepW = 30;
    const gap = 10;
    const n = int.length + 1;
    const cw = Math.min(96, (maxW - unitW - sepW - gap * (n + 1) - 24) / n);
    const ch = cw * 1.42;
    const total = cw * n + gap * (n - 1) + sepW + 24;
    // housing
    ctx.fillStyle = '#0b0c0e';
    roundRect(ctx, x, y, total, ch + 24, 18);
    ctx.fill();
    let cx = x + 12;
    const cells = [...int.map((d) => ({ d })), { sep: true }, { d: tenths, red: true }];
    for (const c of cells) {
      if (c.sep) {
        text(ctx, '.', cx + sepW / 2, y + 12 + ch * 0.82, `700 ${cw}px ${MONO}`, '#8c8d93', 'center');
        cx += sepW;
        continue;
      }
      const g = ctx.createLinearGradient(0, y + 12, 0, y + 12 + ch);
      if (c.red) { g.addColorStop(0, '#9e2827'); g.addColorStop(0.5, RED); g.addColorStop(1, '#9e2827'); }
      else { g.addColorStop(0, '#24252a'); g.addColorStop(0.5, '#3a3b42'); g.addColorStop(1, '#24252a'); }
      ctx.fillStyle = g;
      roundRect(ctx, cx, y + 12, cw, ch, 8);
      ctx.fill();
      text(ctx, String(c.d), cx + cw / 2, y + 12 + ch * 0.72, `600 ${Math.round(cw * 1.02)}px ${MONO}`, '#ffffff', 'center');
      // drum shading
      const s = ctx.createLinearGradient(0, y + 12, 0, y + 12 + ch);
      s.addColorStop(0, 'rgba(0,0,0,0.45)'); s.addColorStop(0.28, 'rgba(0,0,0,0)');
      s.addColorStop(0.72, 'rgba(0,0,0,0)'); s.addColorStop(1, 'rgba(0,0,0,0.45)');
      ctx.fillStyle = s;
      roundRect(ctx, cx, y + 12, cw, ch, 8);
      ctx.fill();
      cx += cw + gap;
    }
    text(ctx, unitLabel, x + total + 24, y + 12 + ch * 0.72, `600 64px ${SANS}`, INK);
    return ch + 24;
  }

  function draw(canvas, data) {
    const ctx = canvas.getContext('2d');
    canvas.width = W;
    canvas.height = H;
    const s = data.summary;
    const unit = data.settings.unit;

    // background
    const bg = ctx.createLinearGradient(0, 0, W, H);
    bg.addColorStop(0, '#16171c');
    bg.addColorStop(1, '#0e0f12');
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, W, H);

    // header
    mouseGlyph(ctx, PAD, 78, 40);
    text(ctx, 'Odomouse', PAD + 42, 110, `600 30px ${SANS}`, INK);
    text(ctx, data.label, W - PAD, 110, `600 30px ${SANS}`, INK, 'right');
    const dates = data.from === data.to ? UI.longDate(data.to) : `${UI.longDate(data.from)} – ${UI.longDate(data.to)}`;
    text(ctx, dates, W - PAD, 150, `400 26px ${SANS}`, MUTED, 'right');

    // hero
    text(ctx, UI.t('cursorTraveled'), PAD, 290, fitText(ctx, UI.t('cursorTraveled'), W - PAD * 2, 700, 52, 36), INK);
    const f = U.formatDistance(s.mouseMeters, unit);
    // the one decorative stroke: a dashed road running through the odometer
    ctx.save();
    ctx.strokeStyle = 'rgba(57,135,229,0.22)';
    ctx.lineWidth = 3;
    ctx.setLineDash([26, 22]);
    ctx.beginPath();
    ctx.moveTo(-20, 470);
    ctx.bezierCurveTo(320, 380, 720, 520, W + 20, 410);
    ctx.stroke();
    ctx.restore();
    const oh = odometer(ctx, f.value, f.unit, PAD, 330, W - PAD * 2);
    const cmp = data.comparisons && data.comparisons.distance;
    if (cmp) {
      const line = cmp.text;
      ctx.fillStyle = '#3987e5';
      ctx.fillRect(PAD, 330 + oh + 40, 6, 46);
      text(ctx, line, PAD + 26, 330 + oh + 78, fitText(ctx, line, W - PAD * 2 - 26, 600, 40, 26), INK);
    }

    // stats 2x2
    const top = 330 + oh + 140;
    const colW = (W - PAD * 2) / 2;
    const rowH = 150;
    const stats = [
      [UI.t('keyboard'), U.formatCount(s.keystrokes)],
      [UI.t('clicks'), U.formatCount(s.clicks.total)],
      [UI.t('activeTime'), U.formatMinutes(s.activeMinutes)],
      s.bestWpm ? [UI.t('fastestTyping'), UI.t('wpm', { n: s.bestWpm })] : [UI.t('scroll'), U.formatDistance(s.scrollMeters, unit).text],
    ];
    ctx.fillStyle = LINE;
    ctx.fillRect(PAD, top, W - PAD * 2, 2);
    ctx.fillRect(PAD, top + rowH, W - PAD * 2, 2);
    ctx.fillRect(PAD, top + rowH * 2, W - PAD * 2, 2);
    ctx.fillRect(PAD + colW, top + 24, 2, rowH * 2 - 48);
    stats.forEach(([label, value], i) => {
      const x = PAD + (i % 2) * colW + (i % 2 ? 36 : 0);
      const y = top + Math.floor(i / 2) * rowH;
      text(ctx, label, x, y + 50, `400 26px ${SANS}`, MUTED);
      text(ctx, value, x, y + 116, fitText(ctx, value, colW - 48, 600, 58, 34), INK);
    });

    // top letters as keycaps
    const ly = top + rowH * 2 + 70;
    text(ctx, UI.t('topLetters'), PAD, ly, `400 26px ${SANS}`, MUTED);
    const letters = s.topLetters.slice(0, 5);
    const kw = (W - PAD * 2 - 4 * 18) / 5;
    const kh = 118;
    if (!letters.length) {
      text(ctx, UI.t('noDataYet'), PAD, ly + 70, `400 30px ${SANS}`, MUTED);
    }
    letters.forEach((l, i) => {
      const x = PAD + i * (kw + 18);
      const y = ly + 26;
      // most-pressed key is the brightest on this dark poster
      const fill = BLUE_RAMP[Math.max(0, 4 - i)];
      ctx.fillStyle = fill;
      roundRect(ctx, x, y, kw, kh, 14);
      ctx.fill();
      ctx.fillStyle = 'rgba(0,0,0,0.22)';
      ctx.fillRect(x + 6, y + kh - 6, kw - 12, 4);
      const ink = luminance(fill) > 0.5 ? '#0b0c0e' : '#ffffff';
      text(ctx, l.letter, x + 20, y + 60, `700 46px ${SANS}`, ink);
      text(ctx, U.formatCount(l.count), x + 20, y + 98, `500 24px ${SANS}`, ink);
    });

    // favourite shortcut
    const fav = s.topShortcuts && s.topShortcuts[0];
    if (fav && window.OdomouseShortcuts) {
      const SC = window.OdomouseShortcuts;
      const name = SC.nameFor(fav.id);
      const y = ly + 26 + kh + 58;
      const lw = text(ctx, `${UI.t('favShortcut')} `, PAD, y, `400 28px ${SANS}`, MUTED);
      const value = `${SC.comboLabel(fav.id)}${name ? ` ${name.toLowerCase()}` : ''}, ${UI.count('times', fav.count, U.formatCount(fav.count))}`;
      text(ctx, value, PAD + lw + 4, y, fitText(ctx, value, W - PAD * 2 - lw - 4, 600, 28, 20), INK);
    }

    // footer
    const fy = H - 64;
    ctx.fillStyle = LINE;
    ctx.fillRect(PAD, fy - 48, W - PAD * 2, 2);
    const foot = UI.count('activeDays', s.activeDays || 0);
    text(ctx, foot, PAD, fy, `500 26px ${SANS}`, INK);
    // where people who see a shared card can get the app
    text(ctx, SITE, W - PAD, fy, `500 26px ${SANS}`, MUTED, 'right');

  }

  function init(api) {
    const dialog = document.getElementById('wrapped');
    const canvas = document.getElementById('wrapped-canvas');
    const status = document.getElementById('wrapped-status');
    let period = 'week';

    async function render() {
      for (const b of document.querySelectorAll('#wrapped-period button')) b.setAttribute('aria-pressed', String(b.dataset.period === period));
      status.textContent = '';
      draw(canvas, await api.getWrapped(period));
    }

    document.getElementById('open-wrapped').addEventListener('click', () => {
      if (!dialog.open) dialog.showModal();
      render();
    });
    for (const b of document.querySelectorAll('#wrapped-period button')) {
      b.addEventListener('click', () => { period = b.dataset.period; render(); });
    }
    document.getElementById('wrapped-save').addEventListener('click', async () => {
      const r = await api.saveWrapped(canvas.toDataURL('image/png'), period);
      status.textContent = r.ok ? UI.t('savedTo', { p: r.filePath }) : '';
    });
    document.getElementById('wrapped-copy').addEventListener('click', async () => {
      const r = await api.copyWrapped(canvas.toDataURL('image/png'));
      status.textContent = r.ok ? UI.t('imageCopied') : '';
    });
    document.getElementById('wrapped-close').addEventListener('click', () => dialog.close());
  }

  window.OdomouseWrapped = { init, draw };
})();
