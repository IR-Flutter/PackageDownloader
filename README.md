# 🚀 PackageDownloader 2.0 (`pdown`)

ابزار بومی، فوق‌سریع، چندسکویی (Cross-Platform) و بدون نیاز به پیش‌نیاز (Zero-Dependency) به زبان Rust برای مدیریت، تست و تغییر آنی میرورهای توسعه‌دهندگان در ایران (**فلاتر**، **دارت**، **گریدل** و **اندروید**) جهت دور زدن تحریم‌ها و رفع مشکلات فیلترینگ یا کندی سرعت.

[![Release](https://img.shields.io/github/v/release/IR-Flutter/PackageDownloader?color=blue&label=Latest%20Release)](https://github.com/IR-Flutter/PackageDownloader/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/IR-Flutter/PackageDownloader/actions/workflows/ci.yml/badge.svg)](https://github.com/IR-Flutter/PackageDownloader/actions)
[![Build & Release](https://github.com/IR-Flutter/PackageDownloader/actions/workflows/release.yml/badge.svg)](https://github.com/IR-Flutter/PackageDownloader/actions)

🌐 **صفحه وب راهنما و دایرکتوری آنلاین:** [https://ir-flutter.github.io/PackageDownloader/](https://ir-flutter.github.io/PackageDownloader/)

---

## ⚡ تفاوت‌های کلیدی و پیشرفت‌های نسخه ۲.۰

* **بازنویسی کامل با Rust:** بدون نیاز به نصب Dart، Flutter یا جاوا روی سیستم کاربر. اجرای آنی در چند میلی‌ثانیه.
* **رجیستری پویا با امضای دیجیتال (Ed25519):** به‌روزرسانی خودکار لیست میرورها بدون نیاز به بیلد و دانلود مجدد برنامه، با اعتبارسنجی رمزنگاری‌شده و حفاظت در برابر بازگشت به عقب (Anti-Rollback).
* **تراکنش‌های امن و بازگشت‌پذیر (Safe Transactions):** ایجاد پشتیبان خودکار (SHA-256) قبل از تغییر هر فایل، قابلیت پیش‌نمایش تغییرات با `--dry-run`، و بازگردانی خودکار در صورت بروز هرگونه خطا.
* **جداسازی هوشمند اکوسیستم‌ها:** جلوگیری از ادغام اشتباه مخازن؛ تفکیک باینری‌های موتور فلاتر (`flutter_infra_release`) از پکیج‌های Pub، و تفکیک Google Maven از Maven Central در Gradle.
* **تست چندمرحله‌ای واقعی:** سنجش سلامت واقعی مخازن بر اساس دانلود نمونه پکیج‌ها و متادیتاها (نه صرفاً پینگ صفحه اصلی).

---

## ⚡ نصب سریع با یک دستور (One-Line Install)

### 🍏 مک و 🐧 لینوکس (macOS / Linux):
```bash
curl -fsSL https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/install.sh | bash
```

### 🪟 ویندوز (PowerShell):
```powershell
irm https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/install.ps1 | iex
```

> **نکته:** دستور `irflutter` همچنان به عنوان نام مستعار (Alias) برای سازگاری کامل با اسکریپت‌های قبلی شما کار می‌کند.

---

## 🌐 لیست میرورهای تأییدشده و پوشش سرویس‌ها

| شناسه میرور | نام میرور | Flutter Storage | Dart / Pub | Gradle Maven | Android Google Maven | دستور فعال‌سازی |
| :--- | :--- | :---: | :---: | :---: | :---: | :--- |
| `pub-azs` | **PUB-AZS** | ❌ (عدم پشتیبانی) | ✅ | ✅ | ✅ | `pdown use pub-azs` |
| `runflare` | **Runflare (رن‌فلر)** | ✅ | ✅ | ✅ (Maven Central) | ❌ | `pdown use runflare` |
| `flutter-cn` | **Flutter Community (CN)** | ✅ | ✅ | ❌ | ❌ | `pdown use flutter-cn` |
| `jamko` | **Jamko / Chrepo** | ❌ | ❌ | ✅ | ✅ | `pdown use jamko --only gradle` |
| `iranserver` | **IranServer (ایران‌سرور)** | ❌ | ❌ | ✅ | ❌ | `pdown use iranserver --only gradle` |
| `official` | **مخازن رسمی گوگل** | پیش‌فرض | پیش‌فرض | پیش‌فرض | پیش‌فرض | `pdown reset` |

> [!NOTE]
> با توجه به عدم ارائه مخزن معتبر و تأییدشده برای CocoaPods CDN در ایران، اکوسیستم iOS برای محافظت از پروژه‌های توسعه‌دهندگان به‌صورت پیش‌فرض غیرفعال است.

---

## 💻 دستورات و نحوه استفاده (Usage)

### ۱. مشاهده لیست میرورها و قابلیت‌ها:
```bash
pdown list
# یا فیلتر براساس اکوسیستم:
pdown list --provider flutter
```

### ۲. تست سرعت زنده و سلامت میرورها (Benchmark):
```bash
pdown test
# یا تست یک میرور خاص با تعیین تایم‌اوت:
pdown test --mirror pub-azs --timeout 5
```

### ۳. فعال‌سازی هوشمند میرور:
```bash
pdown use runflare
# یا فعال‌سازی فقط برای یک اکوسیستم خاص:
pdown use pub-azs --only dart
# یا فعال‌سازی فقط برای گریدل:
pdown use jamko --only gradle
```

### ۴. پیش‌نمایش تغییرات قبل از اعمال (Dry-Run):
```bash
pdown use runflare --dry-run
```

### ۵. بازگشت به تنظیمات اصلی و حذف تمام میرورها:
```bash
pdown reset
```

### ۶. عیب‌یابی و بررسی وضعیت سیستم (Doctor):
```bash
pdown doctor
```

### ۷. مشاهده وضعیت فعلی متغیرها و فایل‌های کانفیگ:
```bash
pdown status
```

### ۸. بررسی و به‌روزرسانی رجیستری میرورها از گیت‌هاب:
```bash
pdown registry status
pdown registry check
pdown registry update
```

### ۹. مدیریت پروفایل‌ها (Profiles):
```bash
pdown profile list
pdown profile use iran
pdown profile create work-project
```

### ۱۰. حالت خروجی JSON (جهت اتوماسیون و IDEها):
```bash
pdown list --json
pdown status --json
pdown doctor --json
```

---

## 📚 مستندات تکمیلی

* [معماری نرم‌افزار (Software Architecture)](docs/architecture/SOFTWARE_ARCHITECTURE.md)
* [مشخصات فنی و طراحی سیستم (Technical Design)](docs/architecture/TECHNICAL_DESIGN.md)
* [گزارش ممیزی کد و ارزیابی میرورها (Repository Audit)](docs/audit/REPOSITORY_AUDIT.md)
* [مشخصات رجیستری پویا (Registry Specification)](docs/REGISTRY_SPECIFICATION.md)
* [امنیت و راستی‌آزمایی رمزنگاری رجیستری (Registry Security)](docs/REGISTRY_SECURITY.md)
* [راهنمای اکوسیستم‌ها و پرووایدرها (Providers Guide)](docs/PROVIDERS.md)
* [راهنمای جامع دستورات (Usage Guide)](docs/USAGE.md)
* [راهنمای نصب و بیلد (Installation)](docs/INSTALLATION.md)
* [عیب‌یابی و حل مشکلات (Troubleshooting)](docs/TROUBLESHOOTING.md)
* [نقشه راه توسعه (Roadmap)](docs/ROADMAP.md)

---

## 📄 لایسنس
این پروژه تحت لایسنس **MIT** منتشر شده است و برای تمام توسعه‌دهندگان فلاتر و اندروید کاملاً رایگان و متن‌باز است.
