# Odomouse: Windows

Vazifalar panelidagi (soat yonidagi) ikonka. Chap tugma bilan bosilsa, kichik oyna ochiladi. O'ng tugma bilan bosilsa, menyu chiqadi: Statistika, Sozlamalar, Chiqish. Ikonka ustiga sichqonchani olib borsangiz, bugungi masofa ko'rinadi.

Windows 11'da yangi ikonkalar avval `^` belgisi ostiga yashirinadi. Doim ko'rinib tursin desangiz, ikonkani vazifalar paneliga sudrab chiqaring.

## Yig'ish

Kerak: [Rust](https://rustup.rs) (MSVC), [.NET SDK 8](https://dotnet.microsoft.com/download), Visual Studio Build Tools (C++).

```powershell
powershell -ExecutionPolicy Bypass -File windows\build.ps1
```

Natija `windows\dist\` papkasida:

- `Odomouse-Setup.exe`: o'rnatuvchi. Ishga tushiring, **Install** ni bosing. Administrator paroli so'ralmaydi: ilova `%LOCALAPPDATA%\Programs\Odomouse\` ga o'rnatiladi, Start menyusiga qo'shiladi. O'chirish: Sozlamalar → Ilovalar → Odomouse → O'chirish (statistika saqlanib qoladi). Yig'ish uchun [Inno Setup 6](https://jrsoftware.org/isdl.php) kerak. U bo'lmasa, o'rnatuvchisiz dastur `windows\build\app\Odomouse.exe` da qoladi.

## Talablar

- .NET Framework 4.8. Windows 10 va 11 da allaqachon bor. Windows 7 va 8.1 ga alohida o'rnatiladi.
- Statistika oynasi uchun Microsoft Edge WebView2. Windows 10 va 11 da odatda bor. Agar yo'q bo'lsa, ilova yuklab olish sahifasini taklif qiladi. Hisoblash WebView2'siz ham ishlayveradi.
- Alohida ruxsat kerak emas.

## Ma'lumotlar

`%APPDATA%\Odomouse\` (`history.json`, `settings.json`). Eski *Mishka Tracker* statistikasi birinchi ishga tushirishda shu yerga ko'chiriladi, o'rnatuvchi esa eski nusxani o'chiradi.

## Tuzilma

```
Odomouse/TrayApp.cs     tray ikonka, oynalar, sozlamalar, fayl saqlash
Odomouse/InputHooks.cs  past darajadagi klaviatura va sichqoncha hook'lari (faqat kuzatadi)
Odomouse/Displays.cs    monitorlar, ularning o'lchami va DPI
Odomouse/WebWindow.cs   src/ui sahifalari WebView2'da, faqat ochiq paytda yaratiladi
Odomouse/Core.cs        Rust yadro (native\x64 yoki native\x86\odomouse_core.dll)
```
