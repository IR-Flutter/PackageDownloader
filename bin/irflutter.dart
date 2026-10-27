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
      print('irflutter v1.0.0 (IR-Flutter Community)');
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
  irflutter list             نمایش لیست تمام میرورهای موجود
  irflutter test             تست سرعت و در دسترس بودن میرورها (Ping)
  irflutter use <نام_میرور>   فعال‌سازی میرور دلخواه (پیش‌فرض: pub-azs)
  irflutter reset            غیرفعال‌سازی میرور و بازگشت به تنظیمات اصلی گوگل
  irflutter status           بررسی وضعیت متغیرهای محیطی و فایل‌های تنظیمات
  irflutter version          نمایش نسخه برنامه

نمونه‌ها:
  irflutter use pub-azs
  irflutter use flutter-cn
  irflutter reset
''');
}

Future<void> _listMirrors() async {
  final mirrors = await MirrorRegistry.getAllMirrors();
  print('\n📋 لیست میرورهای در دسترس فلاتر و گریدل:\n');

  for (final m in mirrors) {
    print('🔹 [${m.id}] ${m.name}');
    print('   توضیحات: ${m.description}');
    print('   وب‌سایت:  ${m.website}');
    print('   Flutter:  ${m.flutterStorageUrl}');
    print('   Pub:      ${m.pubHostedUrl}');
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
    stdout.write('🔍 تست [${m.id}] (${m.website}) ... ');
    final result = await NetworkBenchmark.testMirror(m.id, m.pubHostedUrl);
    if (result.isReachable) {
      print('✅ آنلاین | تاخیر: ${result.latencyMs}ms');
    } else {
      print('❌ ناموفق / فیلتر');
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
  await EnvManager.setVariables(
    pubHostedUrl: mirror.pubHostedUrl,
    flutterStorageUrl: mirror.flutterStorageUrl,
  );
  print('✅ متغیرهای محیطی PUB_HOSTED_URL و FLUTTER_STORAGE_BASE_URL تنظیم شدند.');

  // 2. تنظیم Gradle
  if (mirror.mavenUrl != null) {
    await GradleManager.configureGradle(mavenUrl: mirror.mavenUrl!);
    print('✅ اسکریپت بازنویسی مخازن Gradle (init.gradle) و تایم‌اوت‌ها اعمال شدند.');
  } else {
    await GradleManager.restoreGradle();
    print('ℹ️ این میرور مخزن اختصاصی Gradle ندارد، تنظیمات گریدل به حالت عادی بازگردانده شد.');
  }

  print('\n🎉 میرور "${mirror.name}" با موفقیت فعال شد!');
  print('📌 نکته: لطفاً ترمینال یا IDE خود (VS Code / Android Studio) را یک‌بار ببندید و دوباره باز کنید.');
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
