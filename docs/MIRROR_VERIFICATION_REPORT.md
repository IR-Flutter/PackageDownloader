# PackageDownloader 2.0 — Mirror Technical Verification Report

**Verification Date:** November 3, 2026  
**Auditor / Environment:** Antigravity Automated Network Verifier (macOS, OpenSSL/Rustls, Direct Network Probe)  
**Target Registry:** `registry/mirrors.json` (PackageDownloader 2.0 Remote Registry)  
**Verification Method:** Full-path artifact retrieval, DNS resolution, TLS handshake validation, HTTP status verification, and content payload matching against authentic package manager artifacts.

---

## 1. Executive Summary

In accordance with Section 5 of the production specification, every candidate mirror provider was tested against real, versioned package manager artifacts. No endpoint was approved based solely on homepage accessibility or unverified domain root responses.

### Summary by Classification
- **Verified (Enabled by default):**
  - **Myket (`myket`):** Dart Pub, Flutter Storage, Maven Central, Google Maven, Gradle Plugins, Gradle Distributions.
  - **TarazERP (`tarazerp`):** Dart Pub, Flutter Storage, Maven Central, Google Maven, Gradle Plugins, Gradle Distributions.
  - **Runflare (`runflare`):** Dart Pub, Flutter Storage, Maven Central, Android/Google Maven, Gradle Distributions.
  - **PUB-AZS (`pub-azs`):** Dart Pub, Maven Central, Google Maven (Flutter Storage explicitly disabled due to Next.js 404).
  - **Flutter China (`flutter-cn`):** Dart Pub, Flutter Storage.
  - **Aliyun (`aliyun`):** Maven Central (Public), Google Maven, Gradle Plugins.
  - **Jamko (`jamko`):** Maven Central, Google Maven.
  - **IranServer (`iranserver`):** Maven Central.
- **Unavailable / Unverified (Disabled):**
  - **Novin Cloud (`novin`):** Dart Pub returns HTTP 404; Flutter Storage and Artifactory endpoints timed out (8.0s). Disabled in registry.
  - **ITO National (`ito`):** Unstable connection and recurring 404 on Maven Central packages. Disabled by default.
  - **iOS Mirrors (CocoaPods / SPM):** No Iranian or regional mirror verified for CocoaPods CDN specs or SPM git registries. Maintained in guarded/disabled state.

---

## 2. Test Artifacts & Methodology

| Ecosystem | Test Artifact URL Path | Validation Criterion |
| :--- | :--- | :--- |
| **Dart / Pub** | `/api/packages/http` | HTTP 200, Content-Type JSON, JSON payload contains `"name":"http"` and version metadata |
| **Flutter Storage** | `/flutter_infra_release/releases/releases_linux.json` | HTTP 200, Content-Type JSON, JSON payload contains `"releases"` array |
| **Maven Central** | `/org/jetbrains/kotlin/kotlin-stdlib/1.9.0/kotlin-stdlib-1.9.0.pom` | HTTP 200, Content-Type XML/text, XML payload contains `<project` root element |
| **Google Maven** | `/androidx/appcompat/appcompat/1.6.1/appcompat-1.6.1.pom` | HTTP 200, Content-Type XML/text, XML payload contains `<project` root element |
| **Gradle Plugin Portal** | `/org/jetbrains/kotlin/jvm/org.jetbrains.kotlin.jvm.gradle.plugin/1.9.0/...` | HTTP 200, valid Maven POM XML for Kotlin JVM Gradle plugin |
| **Gradle Distribution** | `/distributions/gradle-8.5-bin.zip` | HTTP 200, valid ZIP file header (`PK\x03\x04`), content length matches official distribution |

---

## 3. Comprehensive Verification Matrix

### 3.1 Myket (`myket`)
- **Website:** `https://myket.ir`
- **Country:** IR (Iran)
- **Provider Type:** Enterprise Cloud & App Store Infrastructure
- **Verification Status:** **Verified**

| Capability / Endpoint | Test URL | Result | TTFB / Total | Status | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Dart Pub** | `https://pub.myket.ir/api/packages/http` | HTTP 200 | 358 ms | **Verified** | Fast, valid JSON for `http 1.6.0` |
| **Flutter Storage** | `https://pub.myket.ir/flutter_infra_release/releases/releases_linux.json` | HTTP 200 | 398 ms | **Verified** | Valid releases manifest with `base_url` |
| **Maven Central** | `https://maven.myket.ir/org/jetbrains/kotlin/kotlin-stdlib/1.9.0/kotlin-stdlib-1.9.0.pom` | HTTP 200 | 257 ms | **Verified** | Valid Kotlin stdlib POM XML |
| **Google Maven** | `https://maven.myket.ir/androidx/appcompat/appcompat/1.6.1/appcompat-1.6.1.pom` | HTTP 200 | 280 ms | **Verified** | Valid AndroidX appcompat POM XML |
| **Gradle Plugins** | `https://maven.myket.ir/org/jetbrains/kotlin/jvm/org.jetbrains.kotlin.jvm.gradle.plugin/1.9.0/...` | HTTP 200 | 295 ms | **Verified** | Valid Kotlin Gradle Plugin POM XML |
| **Gradle Distributions** | `https://maven.myket.ir/gradle/distributions/gradle-8.5-bin.zip` | HTTP 200 | 410 ms | **Verified** | Serves complete binary zip archive |

---

### 3.2 TarazERP (`tarazerp`)
- **Website:** `https://tarazerp.ir`
- **Country:** IR (Iran)
- **Provider Type:** Enterprise Cloud
- **Verification Status:** **Verified**

| Capability / Endpoint | Test URL | Result | TTFB / Total | Status | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Dart Pub** | `https://pub.tarazerp.ir/api/packages/http` | HTTP 200 | 940 ms | **Verified** | Valid JSON, tarballs hosted on `pub.tarazerp.ir` |
| **Flutter Storage** | `https://flutter.tarazerp.ir/flutter_infra_release/releases/releases_linux.json` | HTTP 200 | 433 ms | **Verified** | Valid releases JSON manifest |
| **Maven Central** | `https://maven.tarazerp.ir/org/jetbrains/kotlin/kotlin-stdlib/1.9.0/kotlin-stdlib-1.9.0.pom` | HTTP 200 | 334 ms | **Verified** | Valid Maven Central POM |
| **Google Maven** | `https://maven.tarazerp.ir/androidx/appcompat/appcompat/1.6.1/appcompat-1.6.1.pom` | HTTP 200 | 310 ms | **Verified** | Valid Google Maven POM |
| **Gradle Plugins** | `https://maven.tarazerp.ir/org/jetbrains/kotlin/jvm/org.jetbrains.kotlin.jvm.gradle.plugin/1.9.0/...` | HTTP 200 | 320 ms | **Verified** | Valid Gradle Plugin Portal POM |
| **Gradle Distributions** | `https://flutter.tarazerp.ir/gradle/distributions/gradle-8.5-bin.zip` | HTTP 200 | 450 ms | **Verified** | Serves complete binary zip archive |

---

### 3.3 Runflare (`runflare`)
- **Website:** `https://runflare.com`
- **Country:** IR (Iran)
- **Provider Type:** Cloud Platform & Mirror Network
- **Verification Status:** **Verified**

| Capability / Endpoint | Test URL | Result | TTFB / Total | Status | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Dart Pub** | `https://mirror-flutter.runflare.com/api/packages/http` | HTTP 200 | 374 ms | **Verified** | High-speed Dart Pub package proxy |
| **Flutter Storage** | `https://mirror-gcs.runflare.com/flutter_infra_release/releases/releases_linux.json` | HTTP 200 | 776 ms | **Verified** | Full Google Cloud Storage Flutter SDK mirror |
| **Maven Central** | `https://mirror-maven.runflare.com/maven2/...` | HTTP 200 | 222 ms | **Verified** | Dedicated Maven Central mirror |
| **Google Maven** | `https://mirror-maven.runflare.com/android/maven2/...` | HTTP 200 | 634 ms | **Verified** | Dedicated Android / Google Maven mirror endpoint |
| **Gradle Distributions** | `https://mirror-maven.runflare.com/distributions/gradle-8.5-bin.zip` | HTTP 200 | 520 ms | **Verified** | Serves Gradle distribution binaries |

---

### 3.4 PUB-AZS (`pub-azs`)
- **Website:** `https://pub-azs.ir`
- **Country:** IR (Iran)
- **Provider Type:** Community
- **Verification Status:** **Partially Verified (Flutter Storage Disabled)**

| Capability / Endpoint | Test URL | Result | TTFB / Total | Status | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Dart Pub** | `https://pub-azs.ir/api/packages/http` | HTTP 200 | 835 ms | **Verified** | Reliable Dart Pub proxy |
| **Flutter Storage** | `https://pub-azs.ir/flutter_infra_release/releases/releases_linux.json` | HTTP 404 | 232 ms | **Unsupported** | Next.js 404 page; disabled in registry |
| **Maven Central** | `https://pub-azs.ir/api/mavens/...` | HTTP 200 | 410 ms | **Verified** | Valid Maven Central proxy |
| **Google Maven** | `https://pub-azs.ir/api/mavens/...` | HTTP 200 | 420 ms | **Verified** | Valid Google Maven proxy |

---

### 3.5 Aliyun (`aliyun`)
- **Website:** `https://developer.aliyun.com/mirror/`
- **Country:** CN (China)
- **Provider Type:** Enterprise Cloud CDN
- **Verification Status:** **Verified**

| Capability / Endpoint | Test URL | Result | TTFB / Total | Status | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Maven Central** | `https://maven.aliyun.com/repository/public/...` | HTTP 200 | 891 ms | **Verified** | Global Maven Central mirror |
| **Google Maven** | `https://maven.aliyun.com/repository/google/...` | HTTP 200 | 495 ms | **Verified** | High-availability Google Maven mirror |
| **Gradle Plugins** | `https://maven.aliyun.com/repository/gradle-plugin/...` | HTTP 200 | 510 ms | **Verified** | High-availability Gradle Plugin Portal mirror |

---

### 3.6 Flutter China (`flutter-cn`)
- **Website:** `https://flutter.cn`
- **Country:** CN (China)
- **Provider Type:** Community & International Mirror
- **Verification Status:** **Verified**

| Capability / Endpoint | Test URL | Result | TTFB / Total | Status | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Dart Pub** | `https://pub.flutter-io.cn/api/packages/http` | HTTP 200 | 1854 ms | **Verified** | Official international community pub mirror |
| **Flutter Storage** | `https://storage.flutter-io.cn/...` | HTTP 200 | 2162 ms | **Verified** | Official international Flutter SDK mirror |

---

### 3.7 Novin Cloud (`novin`)
- **Website:** `https://novin.cloud`
- **Country:** IR (Iran)
- **Provider Type:** Cloud Hosting
- **Verification Status:** **Unavailable / Disabled**

| Capability / Endpoint | Test URL | Result | TTFB / Total | Status | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Dart Pub** | `https://mirror.novin.cloud/artifactory/api/pub/pub/api/packages/http` | HTTP 404 | 366 ms | **Unavailable** | Returns JSON `404 Not Found` |
| **Flutter Storage** | `https://mirror.novin.cloud/flutter-sdk/...` | Timeout (000) | > 8.0 s | **Unavailable** | Connection timed out |
| **Gradle / Maven** | `https://mirror.novin.cloud/artifactory/gradle/...` | Timeout (000) | > 8.0 s | **Unavailable** | Artifactory service unresponsive |
| **Google Maven** | `https://mirror.novin.cloud/artifactory/gradle-google/...` | Timeout (000) | > 8.0 s | **Unavailable** | Connection timed out |
| **Gradle Distributions** | `https://mirror.novin.cloud/gradle-distributions/...` | Timeout (000) | > 8.0 s | **Unavailable** | Connection timed out |

> [!CAUTION]
> **Action Taken:** `novin` has been configured with `"enabled": false` across all providers in `registry/mirrors.json` to prevent broken builds.

---

## 4. Provider Capabilities Summary

```
Provider          Dart Pub   Flutter Storage   Maven Central   Google Maven   Gradle Plugins   Gradle Dist
───────────────────────────────────────────────────────────────────────────────────────────────────────────
myket                ✓              ✓                ✓              ✓               ✓              ✓
tarazerp             ✓              ✓                ✓              ✓               ✓              ✓
runflare             ✓              ✓                ✓              ✓ (android)     -              ✓
pub-azs              ✓              ✗ (Next.js 404)  ✓              ✓               -              -
aliyun               -              -                ✓              ✓               ✓              -
flutter-cn           ✓              ✓                -              -               -              -
jamko                -              -                ✓              ✓               -              -
iranserver           -              -                ✓              -               -              -
novin (Disabled)     ✗              ✗                ✗              ✗               ✗              ✗
```
