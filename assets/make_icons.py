"""Odomouse icons, drawn once and exported in every size each OS needs.

The mark: a classic corded mouse whose scroll wheel is the odometer's red
tenths drum, and whose cord is a dashed road, the distance travelled.

Run: python3 assets/make_icons.py   (needs Pillow)
"""
import math
import os
import struct
from PIL import Image, ImageDraw, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
S = 1024          # master size
K = 4             # supersampling for smooth edges

INK_TOP = (0x31, 0x35, 0x3d)
INK_BOTTOM = (0x1c, 0x1e, 0x23)
BODY = (0xf2, 0xf1, 0xec)
SEAM = (0xc9, 0xc8, 0xc1)
RED = (0xc9, 0x4a, 0x41)
RED_DARK = (0x96, 0x2e, 0x29)
ROAD = (0x82, 0xab, 0xe6)


def lerp(a, b, t):
    return tuple(int(round(a[i] + (b[i] - a[i]) * t)) for i in range(3))


def bezier(p0, p1, p2, p3, n=400):
    pts = []
    for i in range(n + 1):
        t = i / n
        u = 1 - t
        pts.append((u ** 3 * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t ** 3 * p3[0],
                    u ** 3 * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t ** 3 * p3[1]))
    return pts


def dashes(path, dash, gap):
    """Split a polyline into dash segments by arc length."""
    out, cur, acc, on = [], [path[0]], 0.0, True
    for a, b in zip(path, path[1:]):
        seg = math.dist(a, b)
        while seg > 0:
            left = (dash if on else gap) - acc
            if seg < left:
                acc += seg
                if on:
                    cur.append(b)
                seg = 0
            else:
                t = left / seg
                p = (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
                if on:
                    cur.append(p)
                    out.append(cur)
                cur = [p]
                a, seg, acc, on = p, seg - left, 0.0, not on
    if on and len(cur) > 1:
        out.append(cur)
    return out


# Geometry in 1024 space (macOS grid: body 824 px with a 100 px margin).
MOUSE = (452, 372, 728, 812)          # x0, y0, x1, y1
MOUSE_R = 138
WHEEL = (566, 430, 614, 520)          # red drum
CORD = ((590, 372), (590, 250), (430, 190), (300, 272))
CORD_TAIL = ((300, 272), (205, 332), (196, 470), (262, 600))


def draw_mark(d, s, mono=False, short=False):
    """The mouse and its road. `s` scales 1024-space coordinates.
    `short`: only the start of the cord (small menu bar sizes)."""
    sc = lambda v: v * s
    body = (0, 0, 0, 255) if mono else BODY + (255,)
    # dashed cord, thick near the mouse, thinner and fainter as it goes
    path = bezier(*CORD) + bezier(*CORD_TAIL)[1:]
    path = [(sc(x), sc(y)) for x, y in path]
    segs = dashes(path, sc(46), sc(30))
    if short:
        segs = segs[:3]
    for i, seg in enumerate(segs):
        w = max(sc(30 - i * 1.8), sc(16))
        if mono:
            col = (0, 0, 0, 255)
        else:
            # fade into the housing colour (opaque, so overlaps stay clean)
            y = seg[len(seg) // 2][1] / (S * s)
            col = lerp(ROAD, lerp(INK_TOP, INK_BOTTOM, y), min(0.75, i / (len(segs) + 1))) + (255,)
        # each dash as a chain of round dots: smooth caps and joins
        step = w / 6
        for a, b in zip(seg, seg[1:]):
            n = max(1, int(math.dist(a, b) / step))
            for k in range(n + 1):
                x = a[0] + (b[0] - a[0]) * k / n
                y2 = a[1] + (b[1] - a[1]) * k / n
                d.ellipse([x - w / 2, y2 - w / 2, x + w / 2, y2 + w / 2], fill=col)
    # body
    x0, y0, x1, y1 = (sc(v) for v in MOUSE)
    d.rounded_rectangle([x0, y0, x1, y1], radius=sc(MOUSE_R), fill=body)
    if not mono:
        cx = (x0 + x1) / 2
        # seam between the buttons, and under them
        d.line([(cx, y0 + sc(8)), (cx, sc(WHEEL[1]) - sc(10))], fill=SEAM + (255,), width=int(sc(8)))
        d.line([(cx, sc(WHEEL[3]) + sc(10)), (cx, sc(586))], fill=SEAM + (255,), width=int(sc(8)))
        d.line([(x0 + sc(6), sc(586)), (x1 - sc(6), sc(586))], fill=SEAM + (255,), width=int(sc(8)))
    # the red drum
    wx0, wy0, wx1, wy1 = (sc(v) for v in WHEEL)
    if mono:
        # template images are one colour: cut the drum out of the body
        d.rounded_rectangle([wx0 - sc(10), wy0 - sc(10), wx1 + sc(10), wy1 + sc(10)], radius=sc(34), fill=(0, 0, 0, 0))
        d.rounded_rectangle([wx0, wy0, wx1, wy1], radius=sc(24), fill=(0, 0, 0, 255))
    else:
        h = wy1 - wy0
        for i in range(int(h)):
            t = i / h
            col = lerp(RED_DARK, RED, 1 - abs(t - 0.5) * 2)
            d.line([(wx0, wy0 + i), (wx1, wy0 + i)], fill=col + (255,))
        mask = Image.new('L', d.im.size, 0)
        ImageDraw.Draw(mask).rounded_rectangle([wx0, wy0, wx1, wy1], radius=sc(24), fill=255)
        return mask
    return None


def app_icon():
    big = S * K
    img = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    m = 100 * K
    # shadow
    shadow = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle([m, m + 16 * K, big - m, big - m + 16 * K], radius=185 * K, fill=(0, 0, 0, 110))
    img.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(20 * K)))
    # housing: the odometer's dark wheel colour
    grad = Image.new('RGB', (big, big))
    gd = ImageDraw.Draw(grad)
    for y in range(big):
        gd.line([(0, y), (big, y)], fill=lerp(INK_TOP, INK_BOTTOM, y / big))
    mask = Image.new('L', (big, big), 0)
    ImageDraw.Draw(mask).rounded_rectangle([m, m, big - m, big - m], radius=185 * K, fill=255)
    img.paste(grad, (0, 0), mask)
    # faint inner edge
    ImageDraw.Draw(img).rounded_rectangle([m + 2 * K, m + 2 * K, big - m - 2 * K, big - m - 2 * K], radius=183 * K,
                                          outline=(255, 255, 255, 22), width=3 * K)
    layer = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    wheel_mask = draw_mark(d, K)
    # the drum was drawn as full-width stripes: keep only its rounded shape
    drum = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    wx0, wy0, wx1, wy1 = (v * K for v in WHEEL)
    drum.paste(layer.crop((wx0, wy0, wx1, wy1)), (wx0, wy0))
    body_only = layer.copy()
    ImageDraw.Draw(body_only).rectangle([wx0, wy0, wx1, wy1], fill=BODY + (255,))
    img.alpha_composite(body_only)
    clear = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    clear.paste(drum, (0, 0), wheel_mask)
    img.alpha_composite(clear)
    return img.resize((S, S), Image.LANCZOS)


def tray_glyph(px):
    """Monochrome mark for the menu bar / panel (template image)."""
    big = 1024
    img = Image.new('RGBA', (big, big), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    draw_mark(d, 1, mono=True, short=px < 48)
    # crop to the mark with a little air, keep it square
    box = img.getbbox()
    side = max(box[2] - box[0], box[3] - box[1]) + 40
    cx, cy = (box[0] + box[2]) / 2, (box[1] + box[3]) / 2
    img = img.crop((int(cx - side / 2), int(cy - side / 2), int(cx + side / 2), int(cy + side / 2)))
    return img.resize((px, px), Image.LANCZOS)


def argb(img):
    """StatusNotifierItem pixmap: ARGB32 in network byte order."""
    out = bytearray()
    for r, g, b, a in img.getdata():
        out += struct.pack('>BBBB', a, r, g, b)
    return bytes(out)


def main():
    icon = app_icon()
    icon.save(os.path.join(HERE, 'icon-1024.png'))

    # macOS
    res = os.path.join(ROOT, 'macos', 'Resources')
    icon.save(os.path.join(res, 'AppIcon.icns'), sizes=[(16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512), (1024, 1024)])
    tray_glyph(18).save(os.path.join(res, 'TrayIcon.png'))
    tray_glyph(36).save(os.path.join(res, 'TrayIcon@2x.png'))
    # the same glyph in the pages (sidebar brand, language picker)
    tray_glyph(18).save(os.path.join(ROOT, 'src', 'assets', 'trayTemplate.png'))
    tray_glyph(36).save(os.path.join(ROOT, 'src', 'assets', 'trayTemplate@2x.png'))

    # Windows and Linux: no macOS margin, the tile fills the square
    full = icon.crop((92, 92, S - 92, S - 92)).resize((S, S), Image.LANCZOS)
    full.save(os.path.join(ROOT, 'windows', 'Odomouse', 'Assets', 'app.ico'),
              sizes=[(16, 16), (20, 20), (24, 24), (32, 32), (40, 40), (48, 48), (64, 64), (128, 128), (256, 256)])
    full.resize((256, 256), Image.LANCZOS).save(os.path.join(ROOT, 'linux', 'assets', 'odomouse.png'))
    for px in (22, 32, 48, 64):
        with open(os.path.join(ROOT, 'linux', 'assets', f'icon-{px}.argb'), 'wb') as f:
            f.write(argb(full.resize((px, px), Image.LANCZOS)))

    # website
    site = os.path.join(ROOT, 'site')
    icon.resize((256, 256), Image.LANCZOS).save(os.path.join(site, 'img', 'icon-256.png'), optimize=True)
    full.resize((180, 180), Image.LANCZOS).save(os.path.join(site, 'img', 'apple-touch-icon.png'), optimize=True)
    full.resize((64, 64), Image.LANCZOS).save(os.path.join(site, 'favicon.png'), optimize=True)
    print('icons written')


if __name__ == '__main__':
    main()
