# PackageDownloader Repository Audit (v1.1.0 to v2.0)

**Date of Audit:** November 3, 2026  
**Auditor:** PackageDownloader Engineering Team  
**Scope:** Complete repository (`IR-Flutter/PackageDownloader`), source code, installers, CI/CD workflows, documentation, website, and external mirror endpoints.

---

## 1. Executive Summary

PackageDownloader (previously known as `irflutter`) was developed in Dart to help Iranian developers configure mirrors for Flutter, Dart, and Gradle. While the tool successfully addressed a critical need for local developers facing geo-blocking and sanctions, an exhaustive audit reveals severe architectural, security, and technical deficiencies in the v1.x codebase:

1. **Unverified & Broken Endpoint Mappings:** The v1.1.0 registry erroneously assumed `pub-azs` served Flutter SDK storage artifacts (`FLUTTER_STORAGE_BASE_URL="https://pub-azs.ir"`), causing Flutter SDK upgrades and engine downloads to fail with Next.js 404 responses.
2. **Harmful Repository Collapsing:** Gradle configuration in v1.x forcibly redirected *all* Gradle dependency and plugin repositories (`maven.google.com`, `plugins.gradle.org`, `mavenCentral`, etc.) to a single Maven URL. For mirrors that only host Maven Central (such as Runflare's `/maven2/`), this completely broke Google Android dependencies (`androidx.*`, `com.google.android.*`).
3. **Shell and File Tampering Without Transactions:** Environment variables were injected by regex search-and-replace into user shell profiles (`.zshrc`, `.bashrc`, `.profile`) and `init.gradle` without atomic write guarantees, validation diffs, rollback journals, or dry-run safety.
4. **Hardcoded Mirror Registry:** Mirrors were compiled into Dart code. Any mirror outage or new endpoint required building and publishing a new application binary release.
5. **Insecure Installers and Supply Chain:** `install.sh` and `install.ps1` downloaded unsigned binaries directly from GitHub releases without SHA-256 verification or artifact integrity checks. CI workflows relied on unpinned actions.
6. **Superficial Benchmarks:** The benchmark command only executed a single HTTP request to the mirror domain root, which yielded false positives for non-functional mirrors.

PackageDownloader 2.0 addresses these issues with a high-performance native Rust implementation (`pdown`), an Ed25519-signed dynamic registry, provider-isolated transactions, and rigorously validated endpoint routing.

---

## 2. Source Code & Architectural Review

### 2.1 File-by-File Analysis

| File | Purpose in v1.x | Identified Defects & Risks |
| :--- | :--- | :--- |
| `bin/irflutter.dart` | CLI entry point | Procedural command parsing with hardcoded switches; exits abruptly with `exit(1)`; default mirror (`pub-azs`) silently assumed if parameter omitted in `use`; no structured JSON output mode for IDEs or scripts. |
| `lib/config/mirrors.dart` | Mirror definitions | Mirror records merged distinct ecosystems into single URLs; local custom mirrors stored in unversioned `~/.irflutter/mirrors.json` without schema or validation; no remote registry synchronization. |
| `lib/core/env_manager.dart` | Shell environment setup | Modifies all detected shell profiles (`.zshrc`, `.bashrc`, `.bash_profile`, `.profile`) simultaneously; regex replacement risks truncating user dotfiles if markers are malformed; no lock file or atomic write. |
| `lib/core/gradle_manager.dart` | Gradle `init.gradle` writer | Directly overwrites `~/.gradle/init.gradle`; backup mechanism is fragile (`init.gradle.irflutter.bak` can be overwritten if tool is run twice); all repositories rewritten to a single mirror URL. |
| `lib/core/network.dart` | Connectivity checker | Sends generic HTTP GET without checking ecosystem endpoints; does not measure latency variance, packet drop, or download throughput; accepts 3xx status codes as reachability without following redirects. |
| `install.sh` / `install.ps1` | Web-based installation | Pipes curl/Invoke-WebRequest output without checksum validation or signature verification; installs to `/usr/local/bin` without checking permissions or fallback safety. |
| `.github/workflows/release.yml`| Release workflow | Third-party GitHub Actions are unpinned (floating tags `@v4`, `@v1`); no automated testing, clippy, or audit step before release; manual dispatch can release untagged builds. |

---

## 3. Deep Dive: Mirror Endpoints & Verification (PUB-AZS & Others)

A primary directive of the audit was direct technical verification of all endpoints rather than relying on unverified claims.

### 3.1 PUB-AZS Technical Audit (`https://pub-azs.ir/`)

| Endpoint Tested | Expected Ecosystem | Actual Response | Verification Status |
| :--- | :--- | :--- | :--- |
| `https://pub-azs.ir/api/packages/http` | Dart Pub API | `HTTP 200 OK` (Valid JSON metadata for `http 1.6.0`) | **Confirmed Working** |
| `https://pub-azs.ir/download.flutter.io/io/flutter/flutter_embedding_debug/...` | Flutter Android Embeddings | `HTTP 200 OK` (Valid `.pom` XML) | **Confirmed Working** |
| `https://pub-azs.ir/api/mavens/androidx/appcompat/appcompat/1.6.1/appcompat-1.6.1.pom` | Google Maven (`dl.google.com`) | `HTTP 200 OK` (Valid `.pom` XML) | **Confirmed Working** |
| `https://pub-azs.ir/api/mavens/org/jetbrains/kotlin/kotlin-stdlib/1.9.0/kotlin-stdlib-1.9.0.pom` | Maven Central (`repo1.maven.org`) | `HTTP 200 OK` (Valid `.pom` XML) | **Confirmed Working** |
| `https://pub-azs.ir/flutter_infra_release/releases/releases_linux.json` | Flutter Storage (`FLUTTER_STORAGE_BASE_URL`) | `HTTP 404 Not Found` (Next.js HTML page) | **Confirmed Broken / Unsupported** |
| `https://pub-azs.ir/distributions/gradle-8.5-bin.zip` | Gradle Distribution Wrapper | `HTTP 404 Not Found` (Next.js HTML page) | **Confirmed Unsupported** |
| `https://pub-azs.ir/cocoapods/` | iOS CocoaPods Specs | `HTTP 404 Not Found` | **Confirmed Unsupported** |

> [!CAUTION]
> **Critical Finding:** PUB-AZS is a valid mirror for **Dart Pub** (`PUB_HOSTED_URL="https://pub-azs.ir"`) and **Maven / Google Maven dependencies** (`https://pub-azs.ir/api/mavens/`), but it **does not mirror Google Cloud Storage for Flutter engine/SDK artifacts**.
> Setting `FLUTTER_STORAGE_BASE_URL="https://pub-azs.ir"` breaks `flutter upgrade`, `flutter precache`, and new Flutter engine installations.

### 3.2 Runflare Technical Audit (`https://runflare.com/mirrors`)

| Endpoint Tested | Expected Ecosystem | Actual Response | Verification Status |
| :--- | :--- | :--- | :--- |
| `https://mirror-flutter.runflare.com/api/packages/http` | Dart Pub API | `HTTP 200 OK` | **Confirmed Working** |
| `https://mirror-gcs.runflare.com/flutter_infra_release/releases/releases_linux.json` | Flutter SDK Storage | `HTTP 200 OK` | **Confirmed Working** |
| `https://mirror-maven.runflare.com/maven2/org/jetbrains/kotlin/...` | Maven Central | `HTTP 200 OK` | **Confirmed Working** |
| `https://mirror-maven.runflare.com/maven2/androidx/appcompat/...` | Google Maven | `HTTP 404 Not Found` | **Not Supported** (Maven Central only) |
| `https://mirror-maven.runflare.com/distributions/gradle-8.5-bin.zip` | Gradle Distribution | `HTTP 200 OK` | **Confirmed Working** |

> [!IMPORTANT]
> Runflare successfully mirrors **Flutter Storage** and **Gradle Distributions**, and mirrors **Maven Central**, but **does not mirror Google Maven**. In v1.x, redirecting Google Maven to Runflare broke Android builds.

### 3.3 Other Listed Mirrors

- **Jamko (`https://gradle.chrepo.ir`):** Confirmed working for Maven Central and Google Maven artifacts; does not mirror Gradle distribution binaries or Flutter storage.
- **IranServer (`https://maven.iranserver.com/repository/maven/`):** Confirmed working for general Maven artifacts; does not mirror Flutter SDK storage.
- **ITO (`https://archive.ito.gov.ir/gradle/`):** Returned 404 for standard Kotlin stdlib POM; endpoint is unstable or restricted.
- **Flutter-CN (`https://pub.flutter-io.cn` / `https://storage.flutter-io.cn`):** Confirmed working for Pub packages and Flutter Storage; does not mirror Gradle.
- **iOS (CocoaPods / Swift Package Manager):** No active Iranian mirror endpoint exists for CocoaPods CDN or SPM registry. Attempting to spoof an iOS mirror without an authentic upstream backend would produce broken builds.

---

## 4. Preservation & Migration Strategy

### 4.1 Functionality to Preserve
- The intuitive CLI syntax (`list`, `use`, `reset`, `status`, `test`).
- Single-command activation capability for users who want quick setup.
- Standalone zero-dependency executable delivery.
- Compatibility with existing user workflows (e.g., alias `irflutter` mapping to `pdown`).

### 4.2 Obsolete / Unsafe Behavior to Discard
- **Discard blind Gradle rewriting:** Gradle configuration must explicitly distinguish Maven Central, Google Maven, and Gradle Plugin Portal.
- **Discard direct modification of `~/.gradle/init.gradle`:** Instead, place managed init scripts into `~/.gradle/init.d/pdown-mirror.gradle` with clear ownership headers.
- **Discard unvalidated dotfile mutations:** Replace manual regex writing with atomic file operations, backup snapshots, diff generation, and dry-run preview.
- **Discard hardcoded mirror tables:** All mirrors must be loaded from the verifiable remote/cached/bundled registry.
- **Discard insecure shell download piping:** Installers must verify cryptographic checksums against signed manifests.

---

## 5. Audit Conclusion

The transition from v1.1.0 (Dart script) to v2.0 (Production Rust) is essential to ensure reliability, security, and developer trust. The architecture outlined in `SOFTWARE_ARCHITECTURE.md` and `TECHNICAL_DESIGN.md` rectifies every vulnerability and endpoint mismatch discovered in this audit.
