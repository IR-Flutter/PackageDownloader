# PackageDownloader 2.0 — Troubleshooting & Diagnostics

---

## 1. Common Issues & Solutions

### 1.1 "Changes not taking effect in terminal or IDE"
- **Cause:** Shell configuration files (`~/.zshrc`, `~/.bashrc`) are evaluated when a shell session starts. Existing terminal windows or running IDEs (VS Code, Android Studio) retain their previous environment.
- **Resolution:**
  - Close and reopen terminal windows or your IDE.
  - Or manually re-source your profile:
    ```bash
    source ~/.zshrc   # or source ~/.bashrc
    ```

### 1.2 "Flutter upgrade or engine download fails with 404"
- **Cause:** Your active mirror does not proxy Google Cloud Storage (`flutter_infra_release`), but `FLUTTER_STORAGE_BASE_URL` was configured.
- **Resolution:**
  - Switch to a certified Flutter storage mirror:
    ```bash
    pdown use runflare
    # or
    pdown use flutter-cn
    ```
  - PUB-AZS does **not** support Flutter Storage. Running `pdown use pub-azs` automatically preserves official GCS or configures only Dart/Pub and Gradle.

### 1.3 "Gradle Android dependencies fail on Runflare"
- **Cause:** Legacy v1.x redirected Google Maven to Runflare's `/maven2/` which only mirrors Maven Central.
- **Resolution:**
  - PackageDownloader 2.0 strictly separates Google Maven and Maven Central.
  - Run `pdown use runflare` (which now correctly skips Google Maven redirection).
  - Or use a mirror that actively mirrors Google Maven:
    ```bash
    pdown use pub-azs
    # or
    pdown use jamko --only gradle
    ```

### 1.4 "Legacy init.gradle conflict detected"
- **Cause:** Running `pdown doctor` reports a legacy v1.x `init.gradle` file in `~/.gradle/init.gradle`.
- **Resolution:**
  - PackageDownloader 2.0 uses `~/.gradle/init.d/pdown-mirror.gradle` and avoids touching `init.gradle`.
  - Remove the legacy file:
    ```bash
    rm ~/.gradle/init.gradle
    ```

### 1.5 "Registry update signature verification failed"
- **Cause:** The downloaded `mirrors.json` did not match `mirrors.json.sig` or was signed by an untrusted key.
- **Resolution:**
  - PackageDownloader rejects untrusted updates to protect your system from compromised sources.
  - The application automatically retains your last known good local registry.
  - Verify your network connection or check if GitHub has outages.

---

## 2. Recovery & Diagnostics Commands

- Run full diagnostic check:
  ```bash
  pdown doctor
  ```
- Reset everything to factory defaults:
  ```bash
  pdown reset
  ```
- Check backups directory:
  ```bash
  pdown config show
  ```
  Snapshots of all modified files are kept in the backup directory before any write operation.
