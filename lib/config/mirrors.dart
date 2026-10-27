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

  const MirrorInfo({
    required this.id,
    required this.name,
    required this.description,
    required this.pubHostedUrl,
    required this.flutterStorageUrl,
    this.mavenUrl,
    required this.website,
    this.isDefault = false,
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
    ),
    MirrorInfo(
      id: 'flutter-cn',
      name: 'Flutter Community Mirror (CN)',
      description: 'میرور رسمی جامعه فلاتر برای دانلود پکیج‌ها و باینری‌های SDK',
      pubHostedUrl: 'https://pub.flutter-io.cn',
      flutterStorageUrl: 'https://storage.flutter-io.cn',
      mavenUrl: null,
      website: 'https://flutter.cn',
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
