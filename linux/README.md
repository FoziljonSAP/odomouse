# Odomouse: Linux

Paneldagi ikonka (StatusNotifierItem). Uni KDE, XFCE, Cinnamon, MATE va Budgie ko'rsatadi. GNOME'da "AppIndicator" kengaytmasi kerak (Ubuntu'da u o'rnatilgan holda keladi). Ikonka yonida bugungi masofa yoziladi (GNOME/Ubuntu). Boshqa muhitlarda masofa ikonka ustiga sichqonchani olib borganda ko'rinadi.

Statistika va sozlamalar brauzerda ochiladi. Chromium yoki Chrome o'rnatilgan bo'lsa, alohida oyna sifatida ochiladi. Shuning uchun ilovaning o'zi juda yengil: bo'sh turganda taxminan 6 MB xotira oladi.

## Talablar

- Ubuntu 22.04, Linux Mint 21, Debian 12 yoki ulardan yangiroq tizim (glibc 2.35+), 64-bit. Boshqa distributivlarda manbadan yig'ish mumkin (pastda).
- Statistikani ko'rish uchun istalgan brauzer.

## O'rnatish

**Ubuntu, Linux Mint, Debian, Pop!_OS:** `odomouse_<versiya>_amd64.deb` faylini ikki marta bosing, ochilgan dasturda **Install** ni bosing. Yoki terminalda:

```bash
sudo apt install ./odomouse_1.0.0_amd64.deb
```

So'ng ilovalar menyusidan **Odomouse** ni oching. O'chirish: `sudo apt remove odomouse` (statistika o'chmaydi).

**Wayland** sessiyasida klaviatura va sichqonchani sanash uchun bir marta `sudo usermod -aG input $USER`, so'ng tizimdan chiqib, qayta kiring. X11'da bu kerak emas.

## Yig'ish

```bash
curl https://sh.rustup.rs -sSf | sh   # Rust, bir marta
sh linux/package.sh                    # linux/dist/odomouse_<versiya>_amd64.deb
cargo build --release --manifest-path linux/Cargo.toml   # yoki faqat dastur: linux/target/release/odomouse
```

Tashqi Rust kutubxonalari ishlatilmaydi. X11, XRecord, XRandR va D-Bus ishga tushganda `dlopen` orqali yuklanadi.

## X11 va Wayland

- **X11**: hammasi ruxsatsiz ishlaydi. Kiritishlar XRECORD orqali olinadi, ekran o'lchamlari XRandR'dan. Masofa aniq hisoblanadi, ilovalar statistikasi ham ishlaydi.
- **Wayland**: xavfsizlik qoidasi sababli oddiy ilovalar klaviatura va sichqonchani kuzata olmaydi. Odomouse kiritishlarni to'g'ridan-to'g'ri `/dev/input` dan o'qiydi. Buning uchun foydalanuvchi `input` guruhida bo'lishi kerak:

  ```bash
  sudo usermod -aG input $USER    # so'ng tizimdan chiqib, qayta kiring
  ```

  Wayland'da cheklovlar bor:
  - Masofa taxminiy, chunki kursor tezlashuvini compositor qo'shadi.
  - Kursor issiqlik xaritasi ishlamaydi.
  - Ilovalar statistikasi ishlamaydi.
  - Touchpad'dagi barmoq harakati masofaga qo'shilmaydi.

  Tugmalar, kliklar va scroll aniq sanaladi.

## Ma'lumotlar

`~/.local/share/odomouse/` (`history.json`, `settings.json`). Eski *Mishka Tracker* (`~/.local/share/mishka-tracker/`) statistikasi birinchi ishga tushirishda shu yerga ko'chiriladi.
