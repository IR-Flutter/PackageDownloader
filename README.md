# 🚀 IR-Flutter Package Downloader & Mirror Manager

ابزار خط فرمان متن‌باز، سبک و چندسکویی (Cross-Platform) برای مدیریت، تست و تغییر آنی میرورهای فلاتر، دارت و گریدل در ایران جهت دور زدن تحریم‌ها و رفع مشکلات فیلترینگ یا کندی سرعت.

شامل تمامی میرورهای رسمی، مراجع ایرانی و فهرست میرورهای ارائه‌شده در [ramineslami.ir/mirrors](https://ramineslami.ir/mirrors).

[![Release](https://img.shields.io/github/v/release/IR-Flutter/PackageDownloader?color=blue&label=Latest%20Release)](https://github.com/IR-Flutter/PackageDownloader/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build & Release](https://github.com/IR-Flutter/PackageDownloader/actions/workflows/release.yml/badge.svg)](https://github.com/IR-Flutter/PackageDownloader/actions)

🌐 **صفحه وب راهنما و دایرکتوری آنلاین:** [https://ir-flutter.github.io/PackageDownloader/](https://ir-flutter.github.io/PackageDownloader/)

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

---

## 🌐 لیست کامل میرورهای پشتیبانی‌شده در ایران

| نام میرور | پوشش سرویس | منبع / مرجع | آدرس Flutter & Dart | آدرس Gradle Maven | دستور فعال‌سازی |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **PUB-AZS** | فلاتر + دارت + گریدل | جامعه فلاتر | `https://pub-azs.ir` | `https://pub-azs.ir/api/mavens/` | `irflutter use pub-azs` |
| **Runflare (رن‌فلر)** | فلاتر + دارت + گریدل | رامین اسلامی | `https://mirror-flutter.runflare.com` | `https://mirror-maven.runflare.com/maven2/` | `irflutter use runflare` |
| **Jamko / Chrepo** | اندروید گریدل و ماون | رامین اسلامی | مخازن رسمی | `https://gradle.chrepo.ir` | `irflutter use-gradle jamko` |
| **IranServer (ایران‌سرور)** | اندروید گریدل و ماون | رامین اسلامی | مخازن رسمی | `https://maven.iranserver.com/repository/maven/` | `irflutter use-gradle iranserver` |
| **ITO National (سازمان فناوری)** | اندروید گریدل و ماون | رامین اسلامی | مخازن رسمی | `https://archive.ito.gov.ir/gradle/maven_central/` | `irflutter use-gradle ito` |
| **Flutter Community (CN)** | فلاتر + دارت | جامعه فلاتر | `https://pub.flutter-io.cn` | — | `irflutter use-flutter flutter-cn` |
| **مخازن رسمی (گوگل)** | بدون میرور | مخازن اصلی | `https://pub.dev` | مخازن اصلی Maven | `irflutter reset` |

---

## 💻 دستورات و نحوه استفاده (Usage)

### ۱. مشاهده لیست کامل میرورها:
```bash
irflutter list
```

### ۲. تست سرعت و پینگ میرورها:
```bash
irflutter test
```

### ۳. فعال‌سازی کامل میرور (فلاتر + گریدل):
```bash
irflutter use pub-azs
# یا
irflutter use runflare
```

### ۴. فعال‌سازی اختصاصی گریدل (Gradle/Maven):
اگر می‌خواهید فقط پکیج‌های اندروید گریدل از میرورهایی مثل جامکو یا ایران‌سرور دانلود شوند:
```bash
irflutter use-gradle jamko
# یا
irflutter use-gradle iranserver
```

### ۵. فعال‌سازی اختصاصی فلاتر و دارت:
```bash
irflutter use-flutter flutter-cn
```

### ۶. بازگشت به تنظیمات اصلی (غیرفعال‌سازی میرور):
```bash
irflutter reset
```

### ۷. مشاهده وضعیت فعلی سیستم:
```bash
irflutter status
```

---

## 📦 دانلود دستی فایل‌های اجرایی (Releases)

در صفحه [GitHub Releases](https://github.com/IR-Flutter/PackageDownloader/releases)، فایل‌های اجرایی کامپایل‌شده بدون نیاز به ران‌تایم Dart برای سیستم‌عامل‌های زیر در دسترس است:
* **ویندوز:** `irflutter-windows-x64.exe`
* **مک (Apple Silicon M1..M4):** `irflutter-macos-arm64`
* **مک (Intel):** `irflutter-macos-x64`
* **لینوکس:** `irflutter-linux-x64`

---

## 🛠 کامپایل دستی از سورس کد

```bash
git clone https://github.com/IR-Flutter/PackageDownloader.git
cd PackageDownloader
dart compile exe bin/irflutter.dart -o irflutter
./irflutter list
```

---

## 📄 لایسنس
این پروژه تحت لایسنس **MIT** منتشر شده است و برای تمام توسعه‌دهندگان فلاتر در ایران کاملاً رایگان و متن‌باز است.
