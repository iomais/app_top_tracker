<div align="center">

# 📌 App Top Tracker

**أداة خفيفة بلغة Rust لواجهة Windows لتثبيت النوافذ فوق التطبيقات الأخرى (Always on Top) وإدارتها عبر System Tray.**

[![Download EXE](https://img.shields.io/badge/Download-App__Top__Tracker.exe-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/iomais/app_top_tracker/releases/latest/download/app_top_tracker.exe)

[![Latest Release](https://img.shields.io/github/v/release/iomais/app_top_tracker?style=for-the-badge&color=orange)](https://github.com/iomais/app_top_tracker/releases/latest)
[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/UI-eframe%20%2F%20egui-00ADD8.svg?style=for-the-badge)]()

</div>

---

## ⚡ تحميل مباشر

| المنصة | رابط التحميل المباشر | النسخة |
|---|---|---|
| **Windows 64-bit** | [📦 تحميل ملف app_top_tracker.exe](https://github.com/iomais/app_top_tracker/releases/latest/download/app_top_tracker.exe) | أحدث إصدار (Release) |

---


<div align="center">
[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/UI-eframe%20%2F%20egui-00ADD8.svg?style=for-the-badge)]()
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6.svg?style=for-the-badge&logo=windows)]()

</div>

---

## 🎯 فكرة البرنامج

يتيح **App Top Tracker** عرض النوافذ المرئية المفتوحة حالياً في نظام Windows، وتجميعها تلقائياً حسب اسم العملية (Process)، مع إمكانية تثبيت أي نافذة أو مجموعة نوافذ لتبقى دائماً في المقدمة (`HWND_TOPMOST`) أو إرجاعها للوضع الطبيعي (`HWND_NOTOPMOST`) بنقرة واحدة، بالإضافة إلى إخفاء النافذة الرئيسية في شريط المهام (System Tray) واستعادتها بسهولة.

---

## ✨ المميزات الرئيسية

- 📌 **تثبيت فوري فوق النوافذ (`Always on Top`):** تبديل حالة النافذة بين `HWND_TOPMOST` و `HWND_NOTOPMOST` مباشرة عبر `SetWindowPos`.
- 🗂️ **تجميع ذكي حسب التطبيق:** يجمع النوافذ التابعة لنفس العملية (`Process`) ويعرض عدد النوافذ، مع إمكانية توسيع القائمة للتحكم بكل نافذة على حدة أو تثبيت المجموعة كاملة.
- 🔄 **تحديث تلقائي ويدوي:** تحديث قائمة النوافذ تلقائياً كل **2 ثانية** (مع الحفاظ على حالة التوسيع `expanded` لكل مجموعة)، بالإضافة إلى زر **Refresh** للتحديث الفوري.
- 📥 **تصغير إلى System Tray:** زر **Tray** لإخفاء نافذة التطبيق الرئيسية وعرض أيقونة مخصصة في شريط النظام؛ النقر على الأيقونة يعيد إظهار النافذة ويجلبها للمقدمة.
- 🔤 **دعم عناوين النوافذ العربية:** تحميل خطوط النظام (`Segoe UI` / `Tahoma` / `Arial`) في `egui` مع تشكيل الحروف العربية وضبط اتجاه المقاطع العربية.
- 🪟 **إخفاء نافذة الكونسول الافتراضية:** استخدام `#![windows_subsystem = "windows"]` لتشغيل الواجهة الرسومية مباشرة دون نافذة طرفية خلفية.

---

## 🛠️ المتطلبات والبناء

### المتطلبات
- نظام **Windows** (يعتمد على واجهات `windows` crate و Win32 APIs).
- تثبيت **Rust** و **Cargo** من [rust-lang.org](https://www.rust-lang.org/).

### التشغيل والبناء

لتشغيل البرنامج في وضع التطوير:
```bash
cargo run
