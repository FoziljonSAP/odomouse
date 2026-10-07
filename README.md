# Odomouse

**How far does your cursor travel?** Odomouse is a small odometer for your mouse. It counts the distance the cursor travels, keystrokes, clicks, scrolling, active time and typing speed, and shows them in the menu bar or system tray and in a statistics window.

Website and downloads: **[odomouse.com](https://odomouse.com)**

- Native apps for **macOS**, **Windows** and **Linux**, idling at a few MB of memory.
- In **English, Russian and Uzbek**, light and dark, 24-hour or 12-hour clock.
- **Private by design**: it counts *which* key was pressed, never what you type. No account, no ads, no network: everything stays in one file on your computer.
- Free software under the **GPL-3.0** licence.

| | Download | First open |
|---|---|---|
| Windows 7–11 | `Odomouse-Setup.exe` | "Windows protected your PC" → More info → Run anyway |
| macOS 11+ | `Odomouse.dmg` | System Settings → Privacy & Security → Open Anyway; then allow Input Monitoring |
| Ubuntu 22.04+, Mint 21+, Debian 12+ | `odomouse_amd64.deb` | On Wayland: `sudo usermod -aG input $USER`, log out and in |

The apps are not yet signed with paid Apple/Microsoft certificates, hence the one-time question on first open.

## What it shows

- Distance in m, km, ft or mi, compared with bananas, buses, football pitches or Eiffel Towers
- Keyboard heatmap, top keys and letters, favourite shortcuts
- Typing speed (words per minute), backspace rate, clicks by button, scroll distance
- Active time by hour, per-app time (opt-in), a break reminder
- Today vs. yesterday at the same time, 7 and 30-day trends, CSV export
- A weekly "Wrapped" card to share

## How the code is organised

```
core/      Rust: all counting, storage, statistics and texts; a C API (include/odomouse_core.h)
macos/     Swift + AppKit menu bar app, built with swiftc (no Xcode project)
windows/   C# WinForms tray app (.NET Framework 4.8) with WebView2, Inno Setup installer
linux/     Rust tray app (StatusNotifierItem over D-Bus); statistics open in the browser
src/ui/    the statistics window, popup and Wrapped card: one web UI for all three apps
src/core/  small JS helpers the web UI uses (units, keyboard layout, shortcuts, comparisons)
site/      odomouse.com
test/      JS tests and the screenshot harness (test/ui/shoot.js, needs Playwright)
```

The shells only feed raw input events into the core and show what it returns, so every platform counts the same way. Each platform folder has a README with build steps.

## Building and testing

```bash
cargo test --manifest-path core/Cargo.toml     # core: units, tracker, storage, languages
sh core/tests/c_smoke.sh                        # the C API
cargo test --manifest-path linux/Cargo.toml
node test/run-all.js                            # web UI helpers
```

- macOS: `ARCHS="arm64 x86_64" sh macos/build.sh && sh macos/dmg.sh`
- Windows: `powershell -ExecutionPolicy Bypass -File windows\build.ps1`
- Linux: `sh linux/package.sh`

GitHub Actions (`.github/workflows/build.yml`) builds all three installers on every push and publishes the website with the latest ones.

## How accurate it is

- **Distance**: each monitor gets its own millimetres-per-point scale, from the size the monitor reports (EDID) when it is plausible, otherwise from an estimate you can correct by entering the diagonal in Settings. Cursor jumps between monitors are not counted.
- **Keys**: auto-repeat while a key is held counts once. A shortcut is a key pressed with ⌘/Ctrl; Option/Alt alone is not, since it types characters on a Mac.
- **Typing speed** counts only time spent typing: pauses over 2 seconds and shortcuts are left out. "Best" is the fastest 30 seconds.
- **Comparisons with yesterday** use yesterday up to the same time of day, not the whole day.
- **Saving** is atomic (write to a temporary file, then replace); a damaged file is kept as `.bak`.

## Privacy

Odomouse stores per-key counts, never the order of keys, text, passwords, window titles or screenshots. Per-app statistics are off by default; when on, only the name of the app in front is read. Nothing is sent over the network. Data lives in:

- macOS: `~/Library/Application Support/Odomouse/`
- Windows: `%APPDATA%\Odomouse\`
- Linux: `~/.local/share/odomouse/`

Odomouse was called *Mishka Tracker* before 1.0; on first start it copies statistics from the old folder.

## Licence

Copyright (C) 2026 the Odomouse authors. Odomouse is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License, version 3, as published by the Free Software Foundation. It comes with no warranty. See [LICENSE](LICENSE).

---

### O'zbekcha

Odomouse sichqoncha kursori bosib o'tgan masofani, klaviatura bosishlarini, kliklarni, faol vaqtni va yozish tezligini sanaydi. macOS, Windows va Linux uchun bepul, ochiq kodli ilova. Yozganingiz saqlanmaydi, ma'lumot kompyuteringizdan chiqmaydi. Yuklab olish: [odomouse.com](https://odomouse.com).

### Русский

Odomouse считает путь курсора мыши, нажатия клавиш, клики, активное время и скорость набора. Бесплатное приложение с открытым кодом для macOS, Windows и Linux. Набранный текст не сохраняется, данные не покидают компьютер. Скачать: [odomouse.com](https://odomouse.com).
