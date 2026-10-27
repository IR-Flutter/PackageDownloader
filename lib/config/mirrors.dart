import 'dart:convert';
import 'dart:io';

class MirrorInfo {
  final String id;
  final String name;
  final String description;
  final String pubHostedUrl;
  final String flutterStorageUrl;
  final String? mavenUrl;
  final String website;
  final bool isDefault;
  final bool supportsFlutter;
  final bool supportsGradle;

  const MirrorInfo({
    required this.id,
    required this.name,
    required this.description,
    required this.pubHostedUrl,
    required this.flutterStorageUrl,
    this.mavenUrl,
    required this.website,
    this.isDefault = false,
    this.supportsFlutter = true,
    this.supportsGradle = false,
  });

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'description': description,
        'pubHostedUrl': pubHostedUrl,
        'flutterStorageUrl': flutterStorageUrl,
        'mavenUrl': mavenUrl,
        'website': website,
        'isDefault': isDefault,
        'supportsFlutter': supportsFlutter,
        'supportsGradle': supportsGradle,
      };

  factory MirrorInfo.fromJson(Map<String, dynamic> json) => MirrorInfo(
        id: json['id'] as String,
        name: json['name'] as String,
        description: json['description'] as String,
        pubHostedUrl: json['pubHostedUrl'] as String,
        flutterStorageUrl: json['flutterStorageUrl'] as String,
        mavenUrl: json['mavenUrl'] as String?,
        website: json['website'] as String,
        isDefault: json['isDefault'] as bool? ?? false,
        supportsFlutter: json['supportsFlutter'] as bool? ?? true,
        supportsGradle: json['supportsGradle'] as bool? ?? (json['mavenUrl'] != null),
      );
}

class MirrorRegistry {
  static const MirrorInfo official = MirrorInfo(
    id: 'official',
    name: 'Official Google / Dart Mirrors',
    description: 'مخازن رسمی و پیش‌فرض گوگل و pub.dev بدون پروکسی/میرور',
    pubHostedUrl: 'https://pub.dev',
    flutterStorageUrl: 'https://storage.googleapis.com',
    mavenUrl: null,
    website: 'https://pub.dev',
    isDefault: true,
    supportsFlutter: true,
    supportsGradle: true,
  );

  static const List<MirrorInfo> builtInMirrors = [
    MirrorInfo(
      id: 'pub-azs',
      name: 'PUB-AZS',
      description: 'میرور ایرانی کامل برای پکیج‌های دارت، باینری فلاتر و پلاگین‌های گریدل/ماون',
      pubHostedUrl: 'https://pub-azs.ir',
      flutterStorageUrl: 'https://pub-azs.ir',
      mavenUrl: 'https://pub-azs.ir/api/mavens/',
      website: 'https://pub-azs.ir',
      supportsFlutter: true,
      supportsGradle: true,
    ),
    MirrorInfo(
      id: 'runflare',
      name: 'Runflare Mirrors (رن‌فلر)',
      description: 'میرور کامل فلاتر و مخازن گریدل/ماون پلتفرم ابری Runflare (برگرفته از لیست رامین اسلامی)',
      pubHostedUrl: 'https://mirror-flutter.runflare.com',
      flutterStorageUrl: 'https://mirror-gcs.runflare.com',
      mavenUrl: 'https://mirror-maven.runflare.com/maven2/',
      website: 'https://runflare.com/mirrors',
      supportsFlutter: true,
      supportsGradle: true,
    ),
    MirrorInfo(
      id: 'jamko',
      name: 'Jamko / Chrepo (جامکو)',
      description: 'میرور اختصاصی مخازن Gradle و Maven در ایران توسط Jamko (برگرفته از لیست رامین اسلامی)',
      pubHostedUrl: 'https://pub.dev',
      flutterStorageUrl: 'https://storage.googleapis.com',
      mavenUrl: 'https://gradle.chrepo.ir',
      website: 'https://jamko.ir',
      supportsFlutter: false,
      supportsGradle: true,
    ),
    MirrorInfo(
      id: 'iranserver',
      name: 'IranServer Maven Mirror (ایران‌سرور)',
      description: 'میرور داخلی مخازن Maven و وابستگی‌های جاوا/اندروید گریدل توسط ایران‌سرور',
      pubHostedUrl: 'https://pub.dev',
      flutterStorageUrl: 'https://storage.googleapis.com',
      mavenUrl: 'https://maven.iranserver.com/repository/maven/',
      website: 'https://mirror.iranserver.com',
      supportsFlutter: false,
      supportsGradle: true,
    ),
    MirrorInfo(
      id: 'ito',
      name: 'ITO National Archive (مخزن ملی فناوری اطلاعات)',
      description: 'مخزن ملی کشور برای پکیج‌ها و پلاگین‌های Maven Central و Gradle Plugins',
      pubHostedUrl: 'https://pub.dev',
      flutterStorageUrl: 'https://storage.googleapis.com',
      mavenUrl: 'https://archive.ito.gov.ir/gradle/maven_central/',
      website: 'https://repo.ito.gov.ir',
      supportsFlutter: false,
      supportsGradle: true,
    ),
    MirrorInfo(
      id: 'flutter-cn',
      name: 'Flutter Community Mirror (CN)',
      description: 'میرور رسمی جامعه فلاتر برای دانلود پکیج‌ها و باینری‌های SDK',
      pubHostedUrl: 'https://pub.flutter-io.cn',
      flutterStorageUrl: 'https://storage.flutter-io.cn',
      mavenUrl: null,
      website: 'https://flutter.cn',
      supportsFlutter: true,
      supportsGradle: false,
    ),
  ];

  static File get _configFile {
    final home = Platform.environment['HOME'] ??
        Platform.environment['USERPROFILE'] ??
        '.';
    return File('$home/.irflutter/mirrors.json');
  }

  static Future<List<MirrorInfo>> getAllMirrors() async {
    final list = <MirrorInfo>[official, ...builtInMirrors];
    try {
      if (await _configFile.exists()) {
        final content = await _configFile.readAsString();
        final List<dynamic> data = jsonDecode(content);
        for (final item in data) {
          final m = MirrorInfo.fromJson(item as Map<String, dynamic>);
          if (!list.any((existing) => existing.id == m.id)) {
            list.add(m);
          }
        }
      }
    } catch (_) {}
    return list;
  }

  static Future<MirrorInfo?> findById(String id) async {
    final mirrors = await getAllMirrors();
    return mirrors.cast<MirrorInfo?>().firstWhere(
          (m) => m?.id.toLowerCase() == id.toLowerCase(),
          orElse: () => null,
        );
  }

  static Future<void> addCustomMirror(MirrorInfo mirror) async {
    final customMirrors = <MirrorInfo>[];
    try {
      if (await _configFile.exists()) {
        final content = await _configFile.readAsString();
        final List<dynamic> data = jsonDecode(content);
        for (final item in data) {
          customMirrors.add(MirrorInfo.fromJson(item as Map<String, dynamic>));
        }
      }
    } catch (_) {}

    customMirrors.removeWhere((m) => m.id == mirror.id);
    customMirrors.add(mirror);

    if (!await _configFile.parent.exists()) {
      await _configFile.parent.create(recursive: true);
    }
    await _configFile.writeAsString(
      const JsonEncoder.withIndent('  ').convert(
        customMirrors.map((e) => e.toJson()).toList(),
      ),
    );
  }
}
