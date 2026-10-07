/* Charts for the dashboard -> window.OdomouseCharts. Plain SVG/canvas, no deps.
   Single-series charts: no legend box (the card title names the series),
   bars <= 24px with 4px rounded data-ends, hairline grid, hover tooltips. */
(function () {
  'use strict';
  const UI = window.OdomouseUI;
  const KB = window.OdomouseKeyboard;
  const U = window.OdomouseUnits;
  const SVGNS = 'http://www.w3.org/2000/svg';

  function svg(tag, attrs) {
    const n = document.createElementNS(SVGNS, tag);
    for (const [k, v] of Object.entries(attrs || {})) n.setAttribute(k, v);
    return n;
  }

  function niceTicks(max, count = 4) {
    if (!(max > 0)) return [0, 1];
    const raw = max / count;
    const mag = Math.pow(10, Math.floor(Math.log10(raw)));
    const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw) || 10 * mag;
    const ticks = [];
    for (let v = 0; v <= max + step * 0.001; v += step) ticks.push(v);
    if (ticks[ticks.length - 1] < max) ticks.push(ticks[ticks.length - 1] + step);
    return ticks;
  }

  /** Column with square base and 4px rounded top. */
  function columnPath(x, y, w, h, r) {
    if (h <= 0) return '';
    const rr = Math.min(r, w / 2, h);
    return `M${x},${y + h}V${y + rr}Q${x},${y} ${x + rr},${y}H${x + w - rr}Q${x + w},${y} ${x + w},${y + rr}V${y + h}Z`;
  }

  /**
   * @param {HTMLElement} host
   * @param {{items:Array<{label:string, value:number, tip:string, emphasis?:boolean}>,
   *          height?:number, tickFormat:(v)=>string, labelEvery?:number, ariaLabel:string}} opts
   */
  function barChart(host, opts) {
    host.textContent = '';
    const height = opts.height || 220;
    const items = opts.items;
    const draw = () => {
      host.textContent = '';
      const width = host.clientWidth || 600;
      const max = Math.max(...items.map((d) => d.value), 0);
      const ticks = niceTicks(max);
      const top = ticks[ticks.length - 1] || 1;
      const tickLabels = ticks.map(opts.tickFormat);
      const left = 12 + Math.max(...tickLabels.map((t) => t.length)) * 6.6;
      const pad = { l: left, r: 8, t: 10, b: 26 };
      const iw = width - pad.l - pad.r;
      const ih = height - pad.t - pad.b;
      const band = iw / Math.max(items.length, 1);
      const bw = Math.max(2, Math.min(24, band * 0.62));
      const root = svg('svg', { width, height, role: 'img', 'aria-label': opts.ariaLabel, class: 'chart' });

      ticks.forEach((t, i) => {
        const y = pad.t + ih - (t / top) * ih;
        root.append(svg('line', { x1: pad.l, x2: width - pad.r, y1: y, y2: y, class: i === 0 ? 'axis' : 'grid' }));
        const lbl = svg('text', { x: pad.l - 8, y: y + 4, 'text-anchor': 'end', class: 'tick' });
        lbl.textContent = tickLabels[i];
        root.append(lbl);
      });

      const every = opts.labelEvery || Math.ceil(items.length / Math.max(1, Math.floor(iw / 46)));
      items.forEach((d, i) => {
        const cx = pad.l + band * i + band / 2;
        const h = (d.value / top) * ih;
        const bar = svg('path', { d: columnPath(cx - bw / 2, pad.t + ih - h, bw, h, 4), class: d.emphasis ? 'bar emph' : 'bar' });
        root.append(bar);
        if (i % every === 0 || i === items.length - 1 && items.length < 40) {
          const xl = svg('text', { x: cx, y: height - 8, 'text-anchor': 'middle', class: 'tick' });
          xl.textContent = d.label;
          root.append(xl);
        }
        const hit = svg('rect', { x: pad.l + band * i, y: pad.t, width: band, height: ih, class: 'hit' });
        hit.addEventListener('mousemove', (e) => { bar.classList.add('hover'); UI.showTip(d.tip, e.clientX, e.clientY); });
        hit.addEventListener('mouseleave', () => { bar.classList.remove('hover'); UI.hideTip(); });
        root.append(hit);
      });
      host.append(root);
    };
    draw();
    observe(host, draw);
  }

  const observers = new WeakMap();
  function observe(host, draw) {
    const prev = observers.get(host);
    if (prev) prev.disconnect();
    let lastW = host.clientWidth;
    const ro = new ResizeObserver(() => {
      if (Math.abs(host.clientWidth - lastW) < 2) return;
      lastW = host.clientWidth;
      draw();
    });
    ro.observe(host);
    observers.set(host, ro);
  }

  /** Physical MacBook keyboard, keys coloured by press count. */
  function keyboardHeatmap(host, keys) {
    host.textContent = '';
    const counts = keys || {};
    const total = Object.values(counts).reduce((a, b) => a + b, 0);
    const max = Math.max(0, ...Object.values(counts));
    const board = UI.el('div', { class: 'kb', role: 'img', 'aria-label': UI.t('keyboardHeatmap') });
    for (const row of KB.ROWS) {
      const r = UI.el('div', { class: 'kb-row' + (row === KB.ROWS[0] ? ' fn-row' : '') });
      for (const [code, label, w] of row) {
        const n = code == null ? 0 : (counts[code] || 0);
        // sqrt scale: Space and E would otherwise wash every other key out
        const t = max > 0 ? Math.sqrt(n / max) : 0;
        const fill = n > 0 ? UI.seqColor(0.12 + 0.88 * t) : null;
        const key = UI.el('div', {
          class: 'kb-key' + (n === 0 ? ' zero' : '') + (code == null ? ' dead' : ''),
          style: `flex:${w} 1 0;` + (fill ? `background:${fill.css};color:${UI.inkOn(fill)}` : ''),
        }, UI.el('span', { text: label }));
        if (code != null) {
          const pct = total ? (n / total) * 100 : 0;
          const name = KB.nameFor(code);
          const tip = `<b>${UI.escapeHtml(name)}</b>: ${UI.count('times', n, U.formatCount(n))}` + (total ? ` (${U.formatNumber(pct, pct < 1 ? 2 : 1)}%)` : '');
          key.addEventListener('mousemove', (e) => UI.showTip(tip, e.clientX, e.clientY));
          key.addEventListener('mouseleave', UI.hideTip);
        }
        r.append(key);
      }
      board.append(r);
    }
    host.append(board, rampLegend(UI.t('less'), UI.t('more')));
  }

  function rampLegend(lo, hi) {
    const stops = [0, 0.25, 0.5, 0.75, 1].map((t) => UI.seqColor(0.12 + 0.88 * t).css).join(',');
    return UI.el('div', { class: 'ramp-legend' },
      UI.el('span', { text: lo }), UI.el('i', { style: `background:linear-gradient(90deg,${stops})` }), UI.el('span', { text: hi }));
  }

  /** Cursor dwell grid drawn inside a screen-shaped frame. */
  function gridHeatmap(host, { grid, gridW, gridH, aspect }) {
    host.textContent = '';
    const total = grid.reduce((a, b) => a + b, 0);
    const max = Math.max(0, ...grid);
    const frame = UI.el('div', { class: 'screen-frame', style: `aspect-ratio:${aspect}` });
    const canvas = UI.el('canvas', { class: 'screen-canvas', role: 'img', 'aria-label': UI.t('cursorHeatmap') });
    frame.append(canvas);
    host.append(frame, rampLegend(UI.t('lessTime'), UI.t('moreTime')));

    const draw = () => {
      const w = frame.clientWidth;
      const h = frame.clientHeight;
      if (!w || !h) return;
      const dpr = window.devicePixelRatio || 1;
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
      canvas.style.width = `${w}px`;
      canvas.style.height = `${h}px`;
      const ctx = canvas.getContext('2d');
      ctx.scale(dpr, dpr);
      const cw = w / gridW;
      const ch = h / gridH;
      const gap = 1;
      for (let r = 0; r < gridH; r++) {
        for (let c = 0; c < gridW; c++) {
          const v = grid[r * gridW + c];
          const t = max > 0 ? Math.sqrt(v / max) : 0;
          ctx.fillStyle = UI.seqColor(v > 0 ? 0.08 + 0.92 * t : 0).css;
          ctx.beginPath();
          if (ctx.roundRect) ctx.roundRect(c * cw + gap / 2, r * ch + gap / 2, cw - gap, ch - gap, 2);
          else ctx.rect(c * cw + gap / 2, r * ch + gap / 2, cw - gap, ch - gap);
          ctx.fill();
        }
      }
    };
    canvas.addEventListener('mousemove', (e) => {
      const rect = canvas.getBoundingClientRect();
      const c = Math.min(gridW - 1, Math.floor(((e.clientX - rect.left) / rect.width) * gridW));
      const r = Math.min(gridH - 1, Math.floor(((e.clientY - rect.top) / rect.height) * gridH));
      const v = grid[r * gridW + c];
      const pct = total ? (v / total) * 100 : 0;
      UI.showTip(total ? UI.t('timeHere', { p: U.formatNumber(pct, pct < 1 ? 2 : 1) }) : UI.t('noDataYet'), e.clientX, e.clientY);
    });
    canvas.addEventListener('mouseleave', UI.hideTip);
    requestAnimationFrame(draw);
    observe(frame, draw);
  }

  /**
   * Ranked list with inline bars (top shortcuts, top apps).
   * rows: [{ label: Node|string, sub?: string, value: number, display: string, tip?: string }]
   */
  function rankList(host, rows, ariaLabel) {
    host.textContent = '';
    const max = Math.max(0, ...rows.map((r) => r.value));
    const list = UI.el('ol', { class: 'rank', 'aria-label': ariaLabel });
    for (const r of rows) {
      const pct = max > 0 ? Math.max(2, (r.value / max) * 100) : 0;
      const item = UI.el('li', {},
        UI.el('div', { class: 'rank-label' }, r.label, r.sub ? UI.el('span', { class: 'rank-sub', text: r.sub }) : null),
        UI.el('div', { class: 'rank-bar' }, UI.el('i', { style: `width:${pct}%` })),
        UI.el('span', { class: 'rank-value', text: r.display }));
      if (r.tip) {
        item.addEventListener('mousemove', (e) => UI.showTip(r.tip, e.clientX, e.clientY));
        item.addEventListener('mouseleave', UI.hideTip);
      }
      list.append(item);
    }
    host.append(list);
  }

  window.OdomouseCharts = { barChart, keyboardHeatmap, gridHeatmap, rankList, niceTicks, columnPath };
})();
