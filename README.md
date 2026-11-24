<div align="center">

# 📌 App Top Tracker

**أداة خفيفة وحرة بلغة Rust ونظام Windows لتثبيت النوافذ فوق التطبيقات الأخرى (Always on Top) وإدارتها بسهولة عبر System Tray.**

[![Download EXE](https://img.shields.io/badge/Download-App__Top__Tracker.exe-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/iomais/app_top_tracker/releases/latest/download/app_top_tracker.exe)
[![Latest Release](https://img.shields.io/github/v/release/iomais/app_top_tracker?style=for-the-badge&color=orange)](https://github.com/iomais/app_top_tracker/releases/latest)
[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/UI-eframe%20%2F%20egui-00ADD8.svg?style=for-the-badge)]()
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6.svg?style=for-the-badge&logo=windows)]()
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)

[تحميل مباشر](#-تحميل-مباشر) • [المميزات](#-المميزات-الرئيسية) • [طريقة الاستخدام](#-طريقة-الاستخدام) • [البناء والتثبيت](#-المتطلبات-والبناء)

</div>

---

## ⚡ تحميل مباشر

لا حاجة لتثبيت Rust أو ترجمة الكود! يمكنك تحميل النسخة التنفيذية الجاهزة فوراً:

| الملف | المنصة | رابط التحميل | النسخة |
|---|---|---|---|
| `app_top_tracker.exe` | **Windows 64-bit** | [📦 تحميل ملف app_top_tracker.exe المباشر](https://github.com/iomais/app_top_tracker/releases/latest/download/app_top_tracker.exe) | أحدث إصدار (Latest Release) |

---

## 🎯 فكرة البرنامج

يتيح **App Top Tracker** عرض كافة النوافذ المرئية المفتوحة حالياً في نظام Windows، وتجميعها تلقائياً حسب اسم العملية (`Process`)، مع إمكانية تثبيت أي نافذة أو مجموعة نوافذ لتبقى دائماً في المقدمة (`HWND_TOPMOST`) أو إرجاعها للوضع الطبيعي (`HWND_NOTOPMOST`) بنقرة واحدة، بالإضافة إلى إمكانية تصغير التطبيق إلى شريط المهام (System Tray).

---

## ✨ المميزات الرئيسية

- 📌 **تثبيت فوري فوق النوافذ (`Always on Top`):** تبديل حالة النافذة بين `HWND_TOPMOST` و `HWND_NOTOPMOST` مباشرة عبر Win32 APIs.
- 🗂️ **تجميع ذكي حسب التطبيق:** تجميع النوافذ التابعة لنفس العملية مع إمكانية التحكم في كل نافذة على حدة أو تثبيت القائمة كاملة.
- 🔄 **تحديث تلقائي ويدوي:** تحديث قائمة النوافذ تلقائياً كل **2 ثانية** مع الحفاظ على التوسيع الحاضر للكتل، بالإضافة إلى زر **Refresh** للتحديث الفوري.
- 📥 **تصغير إلى System Tray:** زر **Tray** لإخفاء النافذة الرئيسية وعرض أيقونة مخصصة في شريط النظام؛ النقر على الأيقونة يعيد إظهار النافذة ويجلبها للمقدمة.
- 🔤 **دعم عناوين النوافذ العربية:** تحميل خطوط النظام القياسية (`Segoe UI` / `Tahoma` / `Arial`) داخل مكتبة `egui` وتنسيق النصوص العربية بشكل صحيح.
- 🪟 **تشغيل شفاف دون الكونسول:** استخدام `#![windows_subsystem = "windows"]` لتشغيل الواجهة الرسومية مباشرة دون ظهور نافذة CMD خلفية.

---

## 🛠️ المتطلبات والبناء

إذا كنت مطوّراً وترغب في بناء المشروع من الكود المصدري:

### المتطلبات
- بيئة **Windows**.
- تثبيت **Rust** و **Cargo** من [rust-lang.org](https://www.rust-lang.org/).

### أوامر البناء والتشغيل

1. **استنساخ المستودع:**
   ```bash
   git clone [https://github.com/iomais/app_top_tracker.git](https://github.com/iomais/app_top_tracker.git)
   cd app_top_tracker
