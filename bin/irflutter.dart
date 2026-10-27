import 'dart:io';
import '../lib/config/mirrors.dart';
import '../lib/core/env_manager.dart';
import '../lib/core/gradle_manager.dart';
import '../lib/core/network.dart';

void main(List<String> args) async {
  if (args.isEmpty || args.contains('--help') || args.contains('-h')) {
    _printHelp();
    return;
  }

  final command = args[0].toLowerCase();
  switch (command) {
    case 'list':
    case 'ls':
      await _listMirrors();
      break;

    case 'use':
    case 'set':
    case 'activate':
      final mirrorId = args.length > 1 ? args[1] : 'pub-azs';
      await _activateMirror(mirrorId);
      break;

    case 'use-gradle':
      if (args.length < 2) {
        print('لطفاً شناسه میرور گریدل را وارد کنید. مثلاً: irflutter use-gradle jamko');
        exit(1);
      }
      await _activateGradleOnly(args[1]);
      break;

    case 'use-flutter':
      if (args.length < 2) {
        print('لطفاً شناسه میرور فلاتر را وارد کنید. مثلاً: irflutter use-flutter pub-azs');
        exit(1);
      }
      await _activateFlutterOnly(args[1]);
      break;

    case 'reset':
    case 'deactivate':
    case 'off':
      await _deactivateMirror();
      break;

    case 'status':
      _showStatus();
      break;

    case 'test':
    case 'ping':
      await _pingMirrors();
      break;

    case 'version':
    case '-v':
      print('irflutter v1.1.0 (IR-Flutter Community)');
      break;

    default:
      print('دستور نامعتبر: $command');
      print('برای مشاهده راهنما: irflutter help');
      exit(1);
  }
}

void _printHelp() {
  print('''
================================================================
🚀 IR-Flutter Mirror CLI (مدیریت میرورهای فلاتر، دارت و گریدل)
================================================================
دستورات:
  irflutter list                نمایش لیست تمام میرورهای موجود
  irflutter test                تست سرعت و در دسترس بودن میرورها (Ping)
  irflutter use <نام_میرور>      فعال‌سازی میرور (فلاتر + گریدل)
  irflutter use-flutter <شناسه>  فعال‌سازی میرور فقط برای فلاتر و دارت
  irflutter use-gradle <شناسه>   فعال‌سازی میرور فقط برای گریدل و ماون
  irflutter reset               غیرفعال‌سازی میرور و بازگشت به تنظیمات اصلی گوگل
  irflutter status              بررسی وضعیت متغیرهای محیطی و فایل‌های تنظیمات
  irflutter version             نمایش نسخه برنامه

نمونه‌ها:
  irflutter use pub-azs
  irflutter use runflare
  irflutter use-gradle jamko
  irflutter use-gradle iranserver
  irflutter use-flutter flutter-cn
  irflutter reset
''');
}

Future<void> _listMirrors() async {
  final mirrors = await MirrorRegistry.getAllMirrors();
  print('\n📋 لیست میرورهای در دسترس فلاتر و گریدل (به همراه میرورهای معرفی‌شده در رامین اسلامی):\n');

  for (final m in mirrors) {
    String coverage;
    if (m.supportsFlutter && m.supportsGradle) {
      coverage = '⚡ فلاتر + دارت + گریدل (پوشش کامل)';
    } else if (m.supportsFlutter) {
      coverage = '🎯 فقط فلاتر و دارت';
    } else {
      coverage = '☕ فقط اندروید گریدل و ماون';
    }

    print('🔹 [${m.id}] ${m.name} ($coverage)');
    print('   توضیحات: ${m.description}');
    print('   وب‌سایت:  ${m.website}');
    if (m.supportsFlutter) {
      print('   Flutter:  ${m.flutterStorageUrl}');
      print('   Pub:      ${m.pubHostedUrl}');
    }
    if (m.mavenUrl != null) {
      print('   Maven:    ${m.mavenUrl}');
    }
    print('   ----------------------------------------');
  }
  print('\nبرای فعال‌سازی هر میرور: irflutter use <شناسه_میرور>');
}

Future<void> _pingMirrors() async {
  final mirrors = await MirrorRegistry.getAllMirrors();
  print('\n⚡ در حال تست پینگ و بررسی در دسترس بودن میرورها...\n');

  for (final m in mirrors) {
    final targetUrl = m.supportsFlutter ? m.pubHostedUrl : (m.mavenUrl ?? m.website);
    stdout.write('🔍 تست [${m.id}] ($targetUrl) ... ');
    final result = await NetworkBenchmark.testMirror(m.id, targetUrl);
    if (result.isReachable) {
      print('✅ آنلاین | تاخیر: ${result.latencyMs}ms');
    } else {
      print('❌ ناموفق / فیلتر (${result.error ?? "خطا"})');
    }
  }
  print('\n');
}

Future<void> _activateMirror(String id) async {
  final mirror = await MirrorRegistry.findById(id);
  if (mirror == null) {
    print('❌ میروری با شناسه "$id" پیدا نشد.');
    print('برای مشاهده شناسه‌ها دستور irflutter list را اجرا کنید.');
    exit(1);
  }

  print('\n⚙️ در حال فعال‌سازی میرور: ${mirror.name} ...');

  // 1. تنظیم Environment Variables
  if (mirror.supportsFlutter) {
    await EnvManager.setVariables(
      pubHostedUrl: mirror.pubHostedUrl,
      flutterStorageUrl: mirror.flutterStorageUrl,
    );
    print('✅ متغیرهای محیطی PUB_HOSTED_URL و FLUTTER_STORAGE_BASE_URL تنظیم شدند.');
  }

  // 2. تنظیم Gradle
  if (mirror.mavenUrl != null) {
    await GradleManager.configureGradle(mavenUrl: mirror.mavenUrl!);
    print('✅ اسکریپت بازنویسی مخازن Gradle (init.gradle) و تایم‌اوت‌ها اعمال شدند.');
  } else {
    await GradleManager.restoreGradle();
    print('ℹ️ این میرور مخزن اختصاصی Gradle ندارد، تنظیمات گریدل دست‌نخورده باقی ماند.');
  }

  print('\n🎉 میرور "${mirror.name}" با موفقیت فعال شد!');
  print('📌 نکته: لطفاً ترمینال یا IDE خود (VS Code / Android Studio) را یک‌بار ببندید و دوباره باز کنید.');
}

Future<void> _activateGradleOnly(String id) async {
  final mirror = await MirrorRegistry.findById(id);
  if (mirror == null || mirror.mavenUrl == null) {
    print('❌ میرور "$id" فاقد آدرس Maven/Gradle معتبر است.');
    exit(1);
  }

  print('\n⚙️ در حال فعال‌سازی اختصاصی گریدل برای میرور: ${mirror.name} ...');
  await GradleManager.configureGradle(mavenUrl: mirror.mavenUrl!);
  print('✅ اسکریپت بازنویسی مخازن Gradle روی ${mirror.mavenUrl} تنظیم شد.');
  print('🎉 گریدل با موفقیت تنظیم شد!');
}

Future<void> _activateFlutterOnly(String id) async {
  final mirror = await MirrorRegistry.findById(id);
  if (mirror == null || !mirror.supportsFlutter) {
    print('❌ میرور "$id" فاقد آدرس فلاتر/دارت معتبر است.');
    exit(1);
  }

  print('\n⚙️ در حال فعال‌سازی اختصاصی فلاتر برای میرور: ${mirror.name} ...');
  await EnvManager.setVariables(
    pubHostedUrl: mirror.pubHostedUrl,
    flutterStorageUrl: mirror.flutterStorageUrl,
  );
  print('✅ متغیرهای محیطی فلاتر روی ${mirror.pubHostedUrl} تنظیم شدند.');
  print('🎉 فلاتر با موفقیت تنظیم شد!');
}

Future<void> _deactivateMirror() async {
  print('\n🔄 در حال غیرفعال‌سازی میرورها و بازگشت به تنظیمات اصلی...');

  await EnvManager.removeVariables();
  await GradleManager.restoreGradle();

  print('✅ متغیرهای محیطی پاک شدند.');
  print('✅ تنظیمات Gradle به حالت رسمی گوگل و مخازن پیش‌فرض بازگردانده شدند.');
  print('\n✨ تمام تنظیمات به حالت اولیه بازگشت!');
}

void _showStatus() {
  print('\n📊 وضعیت فعلی سیستم:');
  final env = EnvManager.getCurrentStatus();
  print('   PUB_HOSTED_URL:           ${env[EnvManager.keyPubHosted] ?? "تنظیم نشده (پیش‌فرض گوگل)"}');
  print('   FLUTTER_STORAGE_BASE_URL: ${env[EnvManager.keyFlutterStorage] ?? "تنظیم نشده (پیش‌فرض گوگل)"}');
  print('   وضعیت Gradle init.gradle: ${GradleManager.isConfigured() ? "فعال (استفاده از میرور)" : "غیرفعال (مخازن رسمی)"}\n');
}
