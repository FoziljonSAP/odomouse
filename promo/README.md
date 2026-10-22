# Odomouse promo video

`ad.html` is a 25-second vertical (1080×1920) motion piece drawn in HTML; its
look is a function of time, `render(t)`. Open it in a browser to watch it live
(`ad.html?lang=uz|en|ru`, or `?t=12` to freeze one moment).

To make the MP4 files (Playwright and ffmpeg are needed):

```bash
npm install          # once, for Playwright
node promo/render.js uz
node promo/render.js en
node promo/render.js ru
```

The videos land in `promo/dist/` (not committed). There is no sound: add music
from a royalty-free library in your video editor.

Fonts: Inter (SIL Open Font License, see `fonts/LICENSE-Inter.txt`).
