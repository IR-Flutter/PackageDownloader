import 'dart:io';

class GradleManager {
  static String get _gradleHome {
    final env = Platform.environment['GRADLE_USER_HOME'];
    if (env != null && env.isNotEmpty) return env;
    final home = Platform.environment['HOME'] ??
        Platform.environment['USERPROFILE'] ??
        '';
    return '$home/.gradle';
  }

  static Directory get gradleDirectory => Directory(_gradleHome);

  static File get initGradleFile => File('${gradleDirectory.path}/init.gradle');
  static File get backupInitGradleFile =>
      File('${gradleDirectory.path}/init.gradle.irflutter.bak');
  static File get gradlePropertiesFile =>
      File('${gradleDirectory.path}/gradle.properties');

  static bool isConfigured() {
    if (!initGradleFile.existsSync()) return false;
    final content = initGradleFile.readAsStringSync();
    return content.contains('IR-Flutter Gradle Mirror Script');
  }

  static Future<void> configureGradle({required String mavenUrl}) async {
    if (!await gradleDirectory.exists()) {
      await gradleDirectory.create(recursive: true);
    }

    // Backup existing init.gradle if not already backed up
    if (await initGradleFile.exists() && !isConfigured()) {
      if (!await backupInitGradleFile.exists()) {
        await initGradleFile.copy(backupInitGradleFile.path);
      }
    }

    final script = '''// IR-Flutter Gradle Mirror Script
// Automatically rewrites standard Maven/Google/JitPack repositories to Iranian Mirror

def MAVEN_MIRROR_URL = "$mavenUrl"

def DEFAULT_REWRITE_LIST = [
    "https://dl.google.com/dl/android/maven2/",
    "https://maven.google.com/",
    "https://plugins.gradle.org/m2/",
    "https://repo1.maven.org/maven2/",
    "https://repo.maven.apache.org/maven2/",
    "https://www.jitpack.io/",
    "https://developer.huawei.com/repo/"
]

def normalizeUrl = { String u ->
    (u ?: "").trim().replaceAll('/+\$', '') + '/'
}

def shouldRewrite = { String url ->
    if (!url) return false
    def norm = normalizeUrl(url)
    if (norm.startsWith("file:") || !norm.contains("://")) return false
    return DEFAULT_REWRITE_LIST.any { normalizeUrl(it) == norm }
}

def rewriteRepo = { repo ->
    if (repo instanceof org.gradle.api.artifacts.repositories.MavenArtifactRepository) {
        def current = repo.url?.toString()
        if (shouldRewrite(current)) {
            repo.setUrl(uri(MAVEN_MIRROR_URL))
        }
    }
}

gradle.allprojects {
    buildscript {
        repositories { all { rewriteRepo(it) } }
    }
    repositories {
        all { rewriteRepo(it) }
    }
}

gradle.settingsEvaluated { settings ->
    try { settings.pluginManagement.repositories.all { rewriteRepo(it) } } catch (ignored) {}
    try { settings.dependencyResolutionManagement.repositories.all { rewriteRepo(it) } } catch (ignored) {}
}
''';

    await initGradleFile.writeAsString(script);

    // Timeout adjustments in gradle.properties
    const timeoutProps = '''
# >>> IR-Flutter Timeout Configuration >>>
systemProp.org.gradle.internal.http.connectionTimeout=120000
systemProp.org.gradle.internal.http.socketTimeout=120000
# <<< IR-Flutter Timeout Configuration <<<
''';

    if (await gradlePropertiesFile.exists()) {
      var content = await gradlePropertiesFile.readAsString();
      if (!content.contains('IR-Flutter Timeout Configuration')) {
        content = '$content\n$timeoutProps';
        await gradlePropertiesFile.writeAsString(content);
      }
    } else {
      await gradlePropertiesFile.writeAsString(timeoutProps);
    }
  }

  static Future<void> restoreGradle() async {
    // If init.gradle was configured by us, delete or restore
    if (await initGradleFile.exists() && isConfigured()) {
      if (await backupInitGradleFile.exists()) {
        await backupInitGradleFile.copy(initGradleFile.path);
        await backupInitGradleFile.delete();
      } else {
        await initGradleFile.delete();
      }
    }

    // Clean gradle.properties
    if (await gradlePropertiesFile.exists()) {
      var content = await gradlePropertiesFile.readAsString();
      final regex = RegExp(
          r'\n?# >>> IR-Flutter Timeout Configuration >>>[\s\S]*?# <<< IR-Flutter Timeout Configuration <<<\n?');
      if (content.contains(regex)) {
        content = content.replaceAll(regex, '\n');
        await gradlePropertiesFile.writeAsString(content);
      }
    }
  }
}
