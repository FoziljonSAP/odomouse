// Renders promo/ad.html into an MP4, frame by frame (needs Playwright + ffmpeg).
//   node promo/render.js [lang] [out.mp4] [fps]
//   node promo/render.js uz dist/odomouse-uz.mp4
const path = require('path');
const { spawn } = require('child_process');
const { chromium } = require('playwright');

const lang = process.argv[2] || 'uz';
const out = path.resolve(process.argv[3] || `promo/dist/odomouse-${lang}.mp4`);
const fps = Number(process.argv[4] || 30);

(async () => {
  require('fs').mkdirSync(path.dirname(out), { recursive: true });
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1080, height: 1920 }, deviceScaleFactor: 1 });
  await page.goto(`file://${path.join(__dirname, 'ad.html')}?capture&lang=${lang}`);
  await page.evaluate(() => window.ready);
  const duration = await page.evaluate(() => window.DURATION);
  const frames = Math.round(duration * fps);
  const ff = spawn('ffmpeg', ['-y', '-loglevel', 'error', '-f', 'image2pipe', '-framerate', String(fps), '-i', '-',
    '-c:v', 'libx264', '-preset', 'slow', '-crf', '17', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', out], { stdio: ['pipe', 'inherit', 'inherit'] });
  const film = await page.$('#film');
  for (let f = 0; f < frames; f++) {
    await page.evaluate((t) => window.render(t), f / fps);
    const jpg = await film.screenshot({ type: 'jpeg', quality: 95 });
    if (!ff.stdin.write(jpg)) await new Promise((r) => ff.stdin.once('drain', r));
    if (f % 150 === 0) process.stdout.write(`${lang}: ${f}/${frames}\n`);
  }
  ff.stdin.end();
  await new Promise((r) => ff.on('close', r));
  await browser.close();
  console.log(`written ${out}`);
})();
