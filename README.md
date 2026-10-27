# 🚀 IR-Flutter Package Downloader & Mirror Manager

ابزار خط فرمان متن‌باز، فوق‌العاده سبک و چندسکویی (Cross-Platform) برای مدیریت، تست و تغییر آنی میرورهای فلاتر، دارت و گریدل در ایران جهت دور زدن تحریم‌ها و رفع مشکلات فیلترینگ یا کندی سرعت.

[![Release](https://img.shields.io/github/v/release/IR-Flutter/PackageDownloader?color=blue&label=Latest%20Release)](https://github.com/IR-Flutter/PackageDownloader/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build & Release](https://github.com/IR-Flutter/PackageDownloader/actions/workflows/release.yml/badge.svg)](https://github.com/IR-Flutter/PackageDownloader/actions)

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

## 🌐 لیست میرورهای پشتیبانی‌شده در ایران

| نام میرور | پوشش سرویس | آدرس Flutter & Dart | آدرس Gradle Maven | دستور فعال‌سازی |
| :--- | :--- | :--- | :--- | :--- |
| **PUB-AZS** | فلاتر + دارت + گریدل | `https://pub-azs.ir` | `https://pub-azs.ir/api/mavens/` | `irflutter use pub-azs` |
| **Flutter Community (CN)** | فلاتر + دارت | `https://pub.flutter-io.cn` | — | `irflutter use flutter-cn` |
| **مخازن رسمی (گوگل)** | مخازن اصلی | `https://pub.dev` | مخازن رسمی Maven | `irflutter reset` |

---

## 💻 دستورات و نحوه استفاده (Usage)

### ۱. مشاهده لیست میرورها:
```bash
irflutter list
```

### ۲. تست سرعت و پینگ میرورها:
```bash
irflutter test
```

### ۳. فعال‌سازی یک میرور:
به عنوان مثال برای فعال‌سازی میرور `pub-azs`:
```bash
irflutter use pub-azs
```
این دستور کارهای زیر را به صورت خودکار انجام می‌دهد:
* متغیرهای محیطی `PUB_HOSTED_URL` و `FLUTTER_STORAGE_BASE_URL` را در شل سیستم (`.bashrc` / `.zshrc` یا رجیستری ویندوز) ثبت می‌کند.
* فایل `init.gradle` را در مسیر `~/.gradle/init.gradle` ایجاد کرده و تمام مخازن Maven را به میرور مدنظر هدایت می‌کند.
* تنظیمات تایم‌اوت (`socketTimeout` و `connectionTimeout`) را در `~/.gradle/gradle.properties` اعمال می‌کند تا دانلود پکیج‌های سنگین قطع نشود.

### ۴. بازگشت به تنظیمات اصلی (غیرفعال‌سازی میرور):
```bash
irflutter reset
```

### ۵. مشاهده وضعیت فعلی سیستم:
```bash
irflutter status
```

---

## 📦 دانلود دستی فایل‌های اجرایی (Releases)

در صفحه [GitHub Releases](https://github.com/IR-Flutter/PackageDownloader/releases)، خروجی‌های کامپایل‌شده آماده برای انواع سیستم‌عامل‌ها موجود است:
* **ویندوز:** `irflutter-windows-x64.exe`
* **مک (Apple Silicon M1/M2/M3/M4):** `irflutter-macos-arm64`
* **مک (Intel):** `irflutter-macos-x64`
* **لینوکس:** `irflutter-linux-x64`

این فایل‌ها نیازی به نصب ران‌تایم، پایتون یا Dart روی سیستم مقصد ندارند و به صورت باینری مستقل (Standalone Binary) اجرا می‌شوند.

---

## 🛠 کامپایل دستی از سورس کد

اگر می‌خواهید خودتان سورس کد را تغییر داده و کامپایل کنید:
```bash
git clone https://github.com/IR-Flutter/PackageDownloader.git
cd PackageDownloader
dart compile exe bin/irflutter.dart -o irflutter
./irflutter list
```

---

## 📄 لایسنس
این پروژه تحت لایسنس **MIT** منتشر شده است و برای تمام توسعه‌دهندگان فلاتر در ایران کاملاً رایگان و متن‌باز است.
