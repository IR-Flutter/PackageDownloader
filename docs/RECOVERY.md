# PackageDownloader 2.0 — Disaster Recovery & Safe Rollback Guide

PackageDownloader 2.0 (`pdown`) is engineered with **fail-closed, transactional safety guarantees**. Every modification to disk or user environment configuration is backed by automated backups, cryptographic checks, and structured rollback handlers.

---

## 1. Automated Transaction Safety & Rollback

### How Transactions Work
Whenever `pdown use <mirror>` or `pdown reset` executes:
1. **Pre-Execution Backup:** An isolated snapshot of every target configuration file is created in `$DATA_DIR/backups/<tx_id>/` with exact file bytes, Unix permissions (`0o644`/`0o755`), and a SHA-256 metadata manifest.
2. **Atomic In-Memory Planning:** Changes are computed as structured `ChangeItem` plans and verified before writing to disk.
3. **Execution Phase:** Changes are applied sequentially using atomic temporary file writes (`atomic_write_bytes`) followed by atomic filesystem rename.
4. **Automated Rollback on Failure:**
   - If an error occurs on step $N$, the transaction engine automatically rolls back changes from steps $N-1$ down to 1 in reverse order.
   - If rollback succeeds: An informative error is displayed showing the primary failure and confirming clean rollback.
   - If rollback itself encounters an unexpected OS-level error: A structured `TransactionAndRollbackFailed` error is thrown containing:
     - The root cause of the primary failure
     - The exact rollback errors encountered
     - The absolute path to the backup snapshot directory for manual recovery

---

## 2. Manual Recovery Procedures

### 2.1 Restoring from Backup Snapshots
Backup files and manifests are stored at:
- **macOS / Linux:** `~/.local/share/pdown/backups/<timestamp>_<uuid>/`
- **Windows:** `%LOCALAPPDATA%\pdown\data\backups\<timestamp>_<uuid>\`

Each backup folder contains:
- `manifest.json`: JSON metadata documenting the target path, SHA-256 checksum, and file permissions.
- Original unmodified copies of each changed file.

To manually restore:
```bash
# Locate the latest backup directory
ls -lt ~/.local/share/pdown/backups/

# Inspect manifest
cat ~/.local/share/pdown/backups/<backup_id>/manifest.json

# Copy back the original files if needed
cp ~/.local/share/pdown/backups/<backup_id>/<file> <target_destination>
```

### 2.2 Shell Environment Variables Recovery (Flutter & Dart)
PackageDownloader manages environment variables inside user shell configuration files (`~/.zshrc` on macOS/zsh, `~/.bashrc` on Linux) between clear marker comments:

```bash
# >>> Managed by pdown [Flutter & Dart Mirrors] >>>
export FLUTTER_STORAGE_BASE_URL='...'
export PUB_HOSTED_URL='...'
# <<< Managed by pdown [Flutter & Dart Mirrors] <<<
```

To automatically revert:
```bash
pdown reset --only flutter,dart
```

To manually revert:
Open your shell configuration (`~/.zshrc` or `~/.bashrc`) in any text editor, delete the block between the `# >>> Managed by pdown` and `# <<< Managed by pdown` markers, and run `source ~/.zshrc` (or open a new terminal window).

### 2.3 Gradle & Maven Mirror Recovery
PackageDownloader configures Gradle through:
1. `~/.gradle/init.d/pdown-mirror.gradle`
2. `~/.gradle/gradle.properties` (timeout properties between markers)
3. `gradle/wrapper/gradle-wrapper.properties` (in project scope)

To automatically revert:
```bash
pdown reset --only gradle
```

To manually revert:
1. Delete `~/.gradle/init.d/pdown-mirror.gradle` (or `%USERPROFILE%\.gradle\init.d\pdown-mirror.gradle` on Windows).
2. Remove the timeout block from `~/.gradle/gradle.properties`.
3. In your project, check `gradle/wrapper/gradle-wrapper.properties` and ensure `distributionUrl` points to `https\://services.gradle.org/distributions/<gradle-version>-bin.zip`.

### 2.4 Corrupted Remote Registry Cache Recovery
If the local registry cache generation is corrupted or fails signature verification:
1. The engine automatically attempts fallback to the `last_known_good_generation`.
2. If `last_known_good_generation` is missing or corrupted, the engine falls back to the immutable binary bundled registry.

To force a fresh update from GitHub:
```bash
pdown registry update --force
```

To administratively reset the anti-rollback state to the bundled baseline:
```bash
pdown registry reset-anti-rollback
```

Or delete the cache directory manually:
```bash
# macOS / Linux
rm -rf ~/.local/share/pdown/registry_state.json ~/.local/share/pdown/generations/

# Windows PowerShell
Remove-Item -Recurse -Force $env:LOCALAPPDATA\pdown\data\registry_state.json, $env:LOCALAPPDATA\pdown\data\generations
```
The application will immediately initialize cleanly from the embedded bundled registry without errors.
