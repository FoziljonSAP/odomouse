// Renders promo/ad.html into an MP4, frame by frame (needs Playwright + ffmpeg),
// with the soundtrack from promo/music.py if it has been made.
//   node promo/render.js [lang] [ratio] [out.mp4]
//   node promo/render.js uz tall        -> promo/dist/odomouse-uz-tall.mp4
//   ratio: tall (1080×1920), wide (1920×1080), square (1080×1080)
const path = require('path');
const { spawn } = require('child_process');
const { chromium } = require('playwright');

const lang = process.argv[2] || 'uz';
const ratio = process.argv[3] || 'tall';
const out = path.resolve(process.argv[4] || `promo/dist/odomouse-${lang}-${ratio}.mp4`);
const fps = 30;
const music = path.join(__dirname, 'dist', 'music.wav');
const fs = require('fs');

(async () => {
  require('fs').mkdirSync(path.dirname(out), { recursive: true });
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1920, height: 1920 }, deviceScaleFactor: 1 });
  await page.goto(`file://${path.join(__dirname, 'ad.html')}?capture&lang=${lang}&ratio=${ratio}`);
  await page.evaluate(() => window.ready);
  const duration = await page.evaluate(() => window.DURATION);
  const frames = Math.round(duration * fps);
  const audio = fs.existsSync(music) ? ['-i', music, '-c:a', 'aac', '-b:a', '192k', '-shortest'] : [];
  const ff = spawn('ffmpeg', ['-y', '-loglevel', 'error', '-f', 'image2pipe', '-framerate', String(fps), '-i', '-', ...audio,
    '-c:v', 'libx264', '-preset', 'slow', '-crf', '17', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', out], { stdio: ['pipe', 'inherit', 'inherit'] });
  const film = await page.$('#film');
  for (let f = 0; f < frames; f++) {
    await page.evaluate((t) => window.render(t), f / fps);
    const jpg = await film.screenshot({ type: 'jpeg', quality: 95 });
    if (!ff.stdin.write(jpg)) await new Promise((r) => ff.stdin.once('drain', r));
    if (f % 150 === 0) process.stdout.write(`${lang} ${ratio}: ${f}/${frames}\n`);
  }
  ff.stdin.end();
  await new Promise((r) => ff.on('close', r));
  await browser.close();
  console.log(`written ${out}`);
})();
