# Odomouse promo video

`ad.html` is a 25-second motion piece drawn in HTML; its look is a function of
time, `render(t)`. Open it in a browser to watch it live:
`ad.html?lang=uz|en|ru&ratio=tall|wide|square` (add `&t=12` to freeze one moment).

| ratio    | size      | for                                   |
|----------|-----------|---------------------------------------|
| `tall`   | 1080×1920 | Reels, TikTok, YouTube Shorts, stories |
| `wide`   | 1920×1080 | YouTube, presentations, the website    |
| `square` | 1080×1080 | Instagram and Telegram posts           |

`music.py` composes the soundtrack (pad, bass, plucks, drums and sound effects
timed to the cuts). It is synthesised from scratch with numpy, so there is no
licence to clear.

To make the MP4 files (Python with numpy and scipy, Playwright and ffmpeg):

```bash
npm install                                   # once, for Playwright
python3 promo/music.py promo/dist/music.wav   # the soundtrack
node promo/render.js uz tall                  # -> promo/dist/odomouse-uz-tall.mp4
node promo/render.js ru wide
node promo/render.js en square
```

The files land in `promo/dist/` (not committed).

Fonts: Inter (SIL Open Font License, see `fonts/LICENSE-Inter.txt`).
