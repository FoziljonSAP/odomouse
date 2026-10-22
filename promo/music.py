"""Original background music and sound effects for the Odomouse promo.

Everything is synthesised here from sine waves and noise, so there is no
licence to worry about. The cues follow the scene times in ad.html:
whooshes into each cut, odometer ticks while the wheels roll, key clicks
over the keyboard, small pops for the chips and badges, a chime on the logo.

    python3 promo/music.py promo/dist/music.wav
"""
import sys
import wave

import numpy as np

SR = 48000
DUR = 25.0
BPM = 120
BEAT = 60 / BPM
N = int(SR * DUR)
rng = np.random.default_rng(7)  # same sound every run

# scene cuts in ad.html
CUTS = [3.8, 8.0, 12.2, 16.4, 19.8, 22.4]

t_all = np.arange(N) / SR
L = np.zeros(N)
R = np.zeros(N)


def note(name):
    """'A4' -> Hz."""
    names = {'C': -9, 'C#': -8, 'Db': -8, 'D': -7, 'D#': -6, 'Eb': -6, 'E': -5, 'F': -4, 'F#': -3,
             'Gb': -3, 'G': -2, 'G#': -1, 'Ab': -1, 'A': 0, 'A#': 1, 'Bb': 1, 'B': 2}
    n, o = name[:-1], int(name[-1])
    return 440.0 * 2 ** ((names[n] + 12 * (o - 4)) / 12)


def add(sig, start, gain=1.0, pan=0.0):
    """Mix a mono signal in at `start` seconds, panned -1..1."""
    i = int(start * SR)
    if i >= N:
        return
    sig = sig[: N - i]
    gl = gain * np.cos((pan + 1) * np.pi / 4)
    gr = gain * np.sin((pan + 1) * np.pi / 4)
    L[i:i + len(sig)] += sig * gl
    R[i:i + len(sig)] += sig * gr


def env(n, a, d, s=0.0, r=0.0, hold=None):
    """ADSR envelope of n samples (times in seconds)."""
    a, d, r = int(a * SR), int(d * SR), int(r * SR)
    hold = n - a - d - r if hold is None else int(hold * SR)
    parts = [np.linspace(0, 1, max(a, 1)), np.linspace(1, s, max(d, 1)), np.full(max(hold, 0), s), np.linspace(s, 0, max(r, 1))]
    e = np.concatenate(parts)
    return np.pad(e, (0, max(0, n - len(e))))[:n]


def lowpass(x, cutoff):
    """One-pole low-pass; cutoff may be an array (a sweep)."""
    cutoff = np.broadcast_to(cutoff, x.shape)
    a = np.exp(-2 * np.pi * cutoff / SR)
    y = np.empty_like(x)
    acc = 0.0
    for i in range(len(x)):
        acc = (1 - a[i]) * x[i] + a[i] * acc
        y[i] = acc
    return y


def lp_fast(x, cutoff):
    """Fixed-cutoff one-pole low-pass via scipy, for long signals."""
    from scipy.signal import lfilter
    a = np.exp(-2 * np.pi * cutoff / SR)
    return lfilter([1 - a], [1, -a], x)


def hp_fast(x, cutoff):
    return x - lp_fast(x, cutoff)


# ------------------------------------------------------------- harmony
# F major, one chord per bar: Fmaj7 - Cadd9/E - Dm7 - Bbmaj7
CHORDS = [
    ['F3', 'A3', 'C4', 'E4'],
    ['E3', 'G3', 'C4', 'D4'],
    ['D3', 'F3', 'A3', 'C4'],
    ['Bb2', 'D3', 'F3', 'A3'],
]
ROOTS = ['F2', 'C2', 'D2', 'Bb1']
BAR = 4 * BEAT
bars = int(np.ceil(DUR / BAR))

# pump: the pad ducks a little on every beat once the drums are in
pump = np.ones(N)
for k in range(int(DUR / BEAT) + 1):
    s = k * BEAT
    if s < 4.0 or s > 23.6:
        continue
    i = int(s * SR)
    n = int(0.35 * SR)
    pump[i:i + n] = np.minimum(pump[i:i + n], 0.55 + 0.45 * np.linspace(0, 1, n) ** 0.6)[: len(pump[i:i + n])]

# pad: detuned saws, soft low-pass, the whole piece
pad = np.zeros(N)
for b in range(bars):
    chord = CHORDS[b % 4]
    start = b * BAR
    n = int(min(BAR + 0.6, DUR - start) * SR)
    if n <= 0:
        continue
    tt = np.arange(n) / SR
    voice = np.zeros(n)
    for nm in chord:
        f = note(nm)
        for det in (-0.07, 0.0, 0.07):
            ph = rng.random() * 2 * np.pi
            ff = f * 2 ** (det / 12)
            voice += 2 * ((tt * ff + ph / (2 * np.pi)) % 1) - 1
    voice *= env(n, 0.5, 0.2, 0.9, 0.6)
    i = int(start * SR)
    pad[i:i + n] += voice[: N - i]
pad = lp_fast(pad, 900) * 0.035
pad *= np.clip(t_all / 2.5, 0, 1)          # fade in
pad *= pump
add(pad, 0, 1.0, -0.25)
add(np.roll(pad, int(0.012 * SR)), 0, 1.0, 0.25)  # a little width

# bass: from the odometer on, on beats 1 and 3 (and an 8th pickup)
for b in range(bars):
    for k, (beat, length) in enumerate([(0, 1.6), (2, 1.2), (3.5, 0.4)]):
        s = b * BAR + beat * BEAT
        if s < 4.0 or s > 23.0:
            continue
        f = note(ROOTS[b % 4])
        n = int(length * BEAT * SR)
        tt = np.arange(n) / SR
        sig = np.sin(2 * np.pi * f * tt) + 0.35 * np.sin(2 * np.pi * 2 * f * tt)
        sig *= env(n, 0.005, 0.25, 0.6, 0.08)
        add(sig, s, 0.22)

# plucks: an arpeggio of the chord, 8th notes, from the odometer on
def pluck(f, length=0.5):
    n = int(length * SR)
    tt = np.arange(n) / SR
    sig = sum((0.6 ** h) * np.sin(2 * np.pi * f * (h + 1) * tt) for h in range(5))
    return sig * np.exp(-tt * 9)

order = [0, 2, 1, 3, 2, 1, 3, 2]
for b in range(bars):
    chord = CHORDS[b % 4]
    for k in range(8):
        s = b * BAR + k * BEAT / 2
        if s < 4.0 or s > 22.3:
            continue
        nm = chord[order[k]]
        f = note(nm) * 2
        add(pluck(f), s, 0.07 if k % 2 else 0.09, pan=(-0.4 if k % 2 else 0.4))

# drums: kick on every beat 4 → 22.4, hats off-beat from 8 s, claps from 12.2 s
def kick():
    n = int(0.35 * SR)
    tt = np.arange(n) / SR
    f = 45 + 90 * np.exp(-tt * 30)
    ph = 2 * np.pi * np.cumsum(f) / SR
    return np.sin(ph) * np.exp(-tt * 9)

def hat():
    n = int(0.06 * SR)
    return hp_fast(rng.standard_normal(n), 7000) * np.exp(-np.arange(n) / SR * 70)

def clap():
    n = int(0.25 * SR)
    x = hp_fast(lp_fast(rng.standard_normal(n), 3000), 900)
    e = np.zeros(n)
    for off in (0, 0.012, 0.024):
        i = int(off * SR)
        e[i:] += np.exp(-np.arange(n - i) / SR * 30)
    return x * e

for k in range(int(DUR / BEAT) + 1):
    s = k * BEAT
    if 4.0 <= s < 22.4:
        add(kick(), s, 0.5)
    if 8.0 <= s < 22.4:
        add(hat(), s + BEAT / 2, 0.05, pan=0.3)
    if 12.2 <= s < 22.4 and k % 2 == 1:
        add(clap(), s, 0.12, pan=-0.1)

# --------------------------------------------------------------- effects
def whoosh(length=0.9):
    """Noise swept up through a low-pass: the air before a cut."""
    n = int(length * SR)
    p = np.linspace(0, 1, n)
    sweep = 250 + 4200 * p ** 2
    x = lowpass(lowpass(rng.standard_normal(n), sweep), sweep * 1.3)  # two poles: softer air
    return x * (p ** 1.6) * np.minimum(1, (1 - p) * 30)

for c in CUTS:
    w = whoosh()
    add(w, c - 0.85, 0.20, pan=-0.2)
    add(np.roll(w, 300), c - 0.85, 0.16, pan=0.2)

# soft sub "impact" on each cut
for c in CUTS:
    n = int(0.6 * SR)
    tt = np.arange(n) / SR
    add(np.sin(2 * np.pi * (60 - 25 * tt) * tt) * np.exp(-tt * 6), c, 0.18)

# odometer: mechanical ticks, fast in the middle of the roll
def tick():
    n = int(0.018 * SR)
    tt = np.arange(n) / SR
    return (np.sin(2 * np.pi * 3200 * tt) + 0.5 * hp_fast(rng.standard_normal(n), 2000)) * np.exp(-tt * 380)

s = 4.1
while s < 6.4:
    p = (s - 4.1) / 2.3
    add(tick(), s, 0.08 * (0.5 + np.sin(np.pi * p)), pan=0.1)
    s += 0.03 + 0.11 * (1 - np.sin(np.pi * p)) ** 2

# keys: a burst of soft key clicks while the heatmap lights up
def key_click():
    n = int(0.03 * SR)
    tt = np.arange(n) / SR
    x = lp_fast(rng.standard_normal(n), 2500) + 0.6 * np.sin(2 * np.pi * 900 * tt)
    return x * np.exp(-tt * 160)

s = 8.5
while s < 10.6:
    add(key_click(), s, 0.10, pan=float(rng.uniform(-0.5, 0.5)))
    s += float(rng.uniform(0.06, 0.15))

# pops for the chips and the badges
def pop(f):
    n = int(0.16 * SR)
    tt = np.arange(n) / SR
    return np.sin(2 * np.pi * f * (1 + 0.6 * np.exp(-tt * 40)) * tt) * np.exp(-tt * 22)

for i, at in enumerate([6.0, 6.35, 6.7]):
    add(pop(note(['C5', 'E5', 'G5'][i])), at + 0.05, 0.14)
for i, at in enumerate([19.9, 20.18, 20.46]):
    add(pop(note(['F5', 'A5', 'C6'][i])), at + 0.05, 0.14)

# the card flips in
add(whoosh(0.5)[::-1], 16.6, 0.10, pan=0.3)

# chime on the logo, then a long final chord
def bell(f, length=2.5):
    n = int(length * SR)
    tt = np.arange(n) / SR
    parts = [(1, 1.0, 2.2), (2.01, 0.5, 3.0), (3.0, 0.25, 4.0), (4.2, 0.15, 6.0)]
    return sum(a * np.sin(2 * np.pi * f * m * tt) * np.exp(-tt * d) for m, a, d in parts)

add(bell(note('F5')), 22.55, 0.18, pan=-0.15)
add(bell(note('C6')), 22.75, 0.12, pan=0.15)
add(bell(note('A5')), 23.05, 0.10, pan=0.0)
n = int(2.6 * SR)
tt = np.arange(n) / SR
final = sum(np.sin(2 * np.pi * note(nm) * tt) for nm in ['F2', 'C3', 'F3', 'A3', 'C4', 'E4'])
add(final * env(n, 0.05, 0.4, 0.6, 1.4), 22.4, 0.06)

# --------------------------------------------------------------- master
mix = np.stack([L, R])
mix = hp_fast(mix, 25)                         # no rumble
fade = np.clip((DUR - t_all) / 1.2, 0, 1)      # fade out over the last 1.2 s
mix *= fade
mix = np.tanh(mix * 1.4) / np.tanh(1.4)        # gentle glue
mix *= 0.89 / np.max(np.abs(mix))              # peak at -1 dBFS

out = sys.argv[1] if len(sys.argv) > 1 else 'promo/dist/music.wav'
pcm = (mix.T * 32767).astype('<i2')
with wave.open(out, 'wb') as w:
    w.setnchannels(2)
    w.setsampwidth(2)
    w.setframerate(SR)
    w.writeframes(pcm.tobytes())
print('written', out)
