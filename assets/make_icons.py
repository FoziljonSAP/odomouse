"""Draws the app icon once (1024 px) and exports every size each OS needs.
Run: python3 assets/make_icons.py   (needs Pillow)"""
import json, os
from PIL import Image, ImageDraw, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
S = 1024

def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))

def icon():
    top, bottom = (0x7d, 0xa6, 0xe0), (0x3f, 0x6f, 0xb8)
    grad = Image.new('RGB', (S, S))
    gd = ImageDraw.Draw(grad)
    for y in range(S):
        gd.line([(0, y), (S, y)], fill=lerp(top, bottom, y / S))
    mask = Image.new('L', (S, S), 0)
    m = 100  # macOS icon grid: 824 px body inside 1024
    ImageDraw.Draw(mask).rounded_rectangle([m, m, S - m, S - m], radius=185, fill=255)
    img = Image.new('RGBA', (S, S), (0, 0, 0, 0))
    shadow = Image.new('RGBA', (S, S), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle([m, m + 14, S - m, S - m + 14], radius=185, fill=(0, 0, 0, 90))
    img.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(18)))
    img.paste(grad, (0, 0), mask)

    d = ImageDraw.Draw(img)
    white = (255, 255, 255, 255)
    soft = (255, 255, 255, 120)
    # dotted path: the distance the cursor travelled
    pts = [(250, 760), (300, 700), (360, 650), (420, 615)]
    for i, (x, y) in enumerate(pts):
        r = 16 + i * 3
        d.ellipse([x - r, y - r, x + r, y + r], fill=soft)
    # mouse body
    bx0, by0, bx1, by1 = 470, 250, 760, 700
    d.rounded_rectangle([bx0, by0, bx1, by1], radius=145, fill=white)
    blue = (0x4f, 0x83, 0xcc, 255)
    cx = (bx0 + bx1) // 2
    d.line([(bx0 + 10, 430), (bx1 - 10, 430)], fill=blue, width=16)
    d.line([(cx, by0 + 10), (cx, 430)], fill=blue, width=16)
    d.rounded_rectangle([cx - 20, 315, cx + 20, 385], radius=20, fill=blue)
    return img

def main():
    big = icon()
    big.save(os.path.join(HERE, 'icon-1024.png'))

    # macOS asset catalog
    appicon = os.path.join(ROOT, 'macos', 'Odomouse', 'Assets.xcassets', 'AppIcon.appiconset')
    os.makedirs(appicon, exist_ok=True)
    images = []
    for pt in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            px = pt * scale
            name = f'icon_{pt}x{pt}' + ('@2x' if scale == 2 else '') + '.png'
            big.resize((px, px), Image.LANCZOS).save(os.path.join(appicon, name))
            images.append({'idiom': 'mac', 'size': f'{pt}x{pt}', 'scale': f'{scale}x', 'filename': name})
    with open(os.path.join(appicon, 'Contents.json'), 'w') as f:
        json.dump({'images': images, 'info': {'version': 1, 'author': 'xcode'}}, f, indent=2)
    with open(os.path.join(os.path.dirname(appicon), 'Contents.json'), 'w') as f:
        json.dump({'info': {'version': 1, 'author': 'xcode'}}, f, indent=2)

    # menu bar template image (same glyph as the Electron version)
    tray = os.path.join(os.path.dirname(appicon), 'TrayIcon.imageset')
    os.makedirs(tray, exist_ok=True)
    src = os.path.join(ROOT, 'src', 'assets')
    for n in ('trayTemplate.png', 'trayTemplate@2x.png'):
        Image.open(os.path.join(src, n)).save(os.path.join(tray, n))
    with open(os.path.join(tray, 'Contents.json'), 'w') as f:
        json.dump({'images': [
            {'idiom': 'universal', 'filename': 'trayTemplate.png', 'scale': '1x'},
            {'idiom': 'universal', 'filename': 'trayTemplate@2x.png', 'scale': '2x'},
        ], 'info': {'version': 1, 'author': 'xcode'}, 'properties': {'template-rendering-intent': 'template'}}, f, indent=2)

    # Windows .ico and Linux png: no macOS margin, the shape fills the square
    big = big.crop((92, 92, S - 92, S - 92)).resize((S, S), Image.LANCZOS)
    win = os.path.join(ROOT, 'windows', 'Odomouse', 'Assets')
    os.makedirs(win, exist_ok=True)
    big.save(os.path.join(win, 'app.ico'), sizes=[(16, 16), (20, 20), (24, 24), (32, 32), (40, 40), (48, 48), (64, 64), (256, 256)])
    lin = os.path.join(ROOT, 'linux', 'assets')
    os.makedirs(lin, exist_ok=True)
    big.resize((256, 256), Image.LANCZOS).save(os.path.join(lin, 'odomouse.png'))
    # raw ARGB32 (network byte order) for the tray item's IconPixmap
    for s in (22, 32, 48, 64):
        im = big.resize((s, s), Image.LANCZOS).convert('RGBA')
        raw = bytearray()
        for r, g, b, a in im.getdata():
            raw += bytes([a, r, g, b])
        with open(os.path.join(lin, f'icon-{s}.argb'), 'wb') as f:
            f.write(raw)
    print('icons written')

if __name__ == '__main__':
    main()
