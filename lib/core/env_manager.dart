import 'dart:io';

class EnvManager {
  static const String keyPubHosted = 'PUB_HOSTED_URL';
  static const String keyFlutterStorage = 'FLUTTER_STORAGE_BASE_URL';

  static Map<String, String?> getCurrentStatus() {
    return {
      keyPubHosted: Platform.environment[keyPubHosted],
      keyFlutterStorage: Platform.environment[keyFlutterStorage],
    };
  }

  static String get _userHome {
    return Platform.environment['HOME'] ??
        Platform.environment['USERPROFILE'] ??
        '';
  }

  static Future<void> setVariables({
    required String pubHostedUrl,
    required String flutterStorageUrl,
  }) async {
    if (Platform.isLinux || Platform.isMacOS) {
      await _setUnixEnv(pubHostedUrl, flutterStorageUrl);
    } else if (Platform.isWindows) {
      await _setWindowsEnv(pubHostedUrl, flutterStorageUrl);
    }
  }

  static Future<void> removeVariables() async {
    if (Platform.isLinux || Platform.isMacOS) {
      await _removeUnixEnv();
    } else if (Platform.isWindows) {
      await _removeWindowsEnv();
    }
  }

  static List<File> _getUnixShellFiles() {
    final home = _userHome;
    if (home.isEmpty) return [];

    final candidateNames = ['.zshrc', '.bashrc', '.bash_profile', '.profile'];
    final files = <File>[];

    for (final name in candidateNames) {
      final f = File('$home/$name');
      if (f.existsSync()) {
        files.add(f);
      }
    }

    if (files.isEmpty) {
      files.add(File('$home/.bashrc'));
    }
    return files;
  }

  static Future<void> _setUnixEnv(
    String pubUrl,
    String storageUrl,
  ) async {
    final files = _getUnixShellFiles();
    final markerStart = '# >>> IR-Flutter Mirror Config >>>';
    final markerEnd = '# <<< IR-Flutter Mirror Config <<<';
    final payload = '''$markerStart
export $keyPubHosted="$pubUrl"
export $keyFlutterStorage="$storageUrl"
$markerEnd''';

    for (final file in files) {
      var content = await file.readAsString();
      if (content.contains(markerStart)) {
        final regex = RegExp('$markerStart[\\s\\S]*?$markerEnd');
        content = content.replaceAll(regex, payload);
      } else {
        content = '$content\n\n$payload\n';
      }
      await file.writeAsString(content);
    }
  }

  static Future<void> _removeUnixEnv() async {
    final files = _getUnixShellFiles();
    final markerStart = '# >>> IR-Flutter Mirror Config >>>';
    final markerEnd = '# <<< IR-Flutter Mirror Config <<<';

    for (final file in files) {
      var content = await file.readAsString();
      if (content.contains(markerStart)) {
        final regex = RegExp('\n?$markerStart[\\s\\S]*?$markerEnd\n?');
        content = content.replaceAll(regex, '\n');
        await file.writeAsString(content);
      }
    }
  }

  static Future<void> _setWindowsEnv(
    String pubUrl,
    String storageUrl,
  ) async {
    final script = '''
[Environment]::SetEnvironmentVariable('$keyPubHosted', '$pubUrl', 'User')
[Environment]::SetEnvironmentVariable('$keyFlutterStorage', '$storageUrl', 'User')
''';
    await Process.run('powershell', ['-NoProfile', '-Command', script]);
  }

  static Future<void> _removeWindowsEnv() async {
    final script = '''
[Environment]::SetEnvironmentVariable('$keyPubHosted', \$null, 'User')
[Environment]::SetEnvironmentVariable('$keyFlutterStorage', \$null, 'User')
''';
    await Process.run('powershell', ['-NoProfile', '-Command', script]);
  }
}
