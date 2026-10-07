# Odomouse: macOS

Menu bar ilovasi: tepada sichqoncha ikonkasi va bugungi masofa turadi. Ikonkani bossangiz, qisqa statistika chiqadi. O'ng tugma bilan bossangiz, menyu ochiladi: Statistika, Sozlamalar, Chiqish.

## Tayyor ilovani ishga tushirish

1. `Odomouse.dmg` ni oching. Ochilgan oynada **Odomouse** ni yonidagi **Applications** papkasiga sudrab tashlang.
2. Applications'dan oching. macOS "could not verify" desa: System Settings → Privacy & Security → **Open Anyway**. Ilova hali Apple sertifikati bilan imzolanmagan, shuning uchun bu bir marta so'raladi.
3. Klaviaturani sanash uchun **Input Monitoring** ruxsati kerak: menu bar'dagi belgini o'ng tugma bilan bosib, **"Klaviatura uchun ruxsat berish…"** ni tanlang va ochilgan ro'yxatda Odomouse'ni yoqing. Ruxsat berilgach, qayta ishga tushirmasdan sanash boshlanadi. Sichqoncha masofasi ruxsatsiz ham sanaladi.

## O'zingiz yig'ish (Xcode shart emas)

Command Line Tools va Rust yetarli. Homebrew o'rnatilgan bo'lsa, Command Line Tools ham allaqachon bor. Rust bo'lmasa, skript uni o'zi o'rnatadi.

```bash
sh macos/build.sh
```

Natija: `macos/dist/Odomouse.app`. `sh macos/dmg.sh` undan `Odomouse.dmg` yasaydi. Intel va Apple Silicon ikkalasi uchun yig'ish: `ARCHS="arm64 x86_64" sh macos/build.sh`.

Har safar qayta yig'ilganda macOS uni yangi ilova deb biladi. Shuning uchun Input Monitoring ruxsatini qaytadan berish kerak bo'lishi mumkin: ro'yxatdan eskisini "−" bilan o'chirib, yangisini qo'shing.

## Ma'lumotlar

`~/Library/Application Support/Odomouse/`. Eski *Mishka Tracker* statistikasi (`~/Library/Application Support/Mishka Tracker/`) birinchi ishga tushirishda shu yerga ko'chiriladi.

Menu bar to'lib, belgi ekran tepasidagi kamera qismi ortida qolsa, ilova buni sezadi va xabar beradi. Ilovani Applications'dan qayta ochsangiz, statistika oynasi ochiladi.

## Tuzilma

```
Odomouse/AppDelegate.swift   menu bar, oynalar, sozlamalar, fayl saqlash
Odomouse/InputMonitor.swift  CGEventTap (faqat kuzatadi, hech narsani o'zgartirmaydi)
Odomouse/DisplayProbe.swift  monitorlar va ularning haqiqiy o'lchami (EDID)
Odomouse/WebPage.swift       src/ui sahifalari WKWebView'da, faqat ochiq paytda yaratiladi
Odomouse/Core.swift          Rust yadro (core/) bilan bog'lanish
Resources/                   ikonkalar
build.sh                     swiftc + cargo bilan yig'ish
dmg.sh                       Odomouse.dmg
```
