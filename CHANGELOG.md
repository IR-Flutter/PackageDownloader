# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.1] - 2026-11-03

### Fixed
- **Anti-Rollback State Integrity (P0-1):**
  - Resolved silent fallback to bundled baseline on persistent state corruption (`registry_state.json`). Implemented safe recovery (`recover_persistent_state`) reconstructing monotonic state from verified on-disk generations without weakening revision boundaries.
  - Enforced `highest_accepted_revision` across all cache generations and legacy fallback.
- **Registry Signing Pipeline (P0-2):**
  - Resolved circular testing dependency in `.github/workflows/registry-signing.yml` by adding `--validate-only` schema and semantics check mode to `sign_registry`.
  - Fixed step-level secret environment variable evaluation in signing condition; workflow now fails closed if `REGISTRY_SIGNING_PRIVATE_KEY` is not configured.
  - Replaced silent `continue-on-error: true` with structured informational endpoint probing and logging.
- **Windows CRLF Checkout Line Ending Normalization (P0-3):**
  - Configured repository `.gitattributes` enforcing `text eol=lf` across all registry JSON, signatures, Rust source code, and shell scripts, preventing Windows git checkout CRLF byte alterations.
- **Windows Environment Variable Error Handling (P0-4):**
  - Replaced unverified command execution with strict exit code checks, stderr capture, and variable name/value validation (`validate_env_var_name`, `validate_env_var_value`) in `EnvironmentManager`.
  - Wired `EnvironmentSet` and `EnvironmentUnset` change items into the transactional execution engine with reversible rollback actions (`RollbackAction::RevertEnvironment`).
- **Global Mirror Enabled Enforcement (P1-1):**
  - Strictly enforced `mirror.enabled` filtering across `list_mirrors`, `find_mirror`, `handle_use`, and benchmark recommendations.
- **Registry Verification Consistency (P1-2):**
  - Extended `registry/schema.json` with `upstream_reference` verification type and updated `official` baseline.
  - Documented international CDN latency considerations for `flutter-cn`.
- **Benchmark Multi-Sample Probes & Recommendations (P1-3):**
  - Implemented multi-sample probes in `BenchmarkEngine` to smooth out network jitter and separate transfer throughput from handshake latency.
  - Added automated per-ecosystem recommendations (`best_for_flutter`, `best_for_dart`, `best_for_gradle`).
  - Guaranteed that mirrors with disabled ecosystem providers are never recommended for that ecosystem.
- **Gradle Routing & Custom Repository Preservation (P1-4):**
  - Verified preservation of JitPack (`https://jitpack.io`) and custom internal repositories in Gradle init.d scripts.
- **CI/CD Supply-Chain Hardening (P2-1):**
  - Pinned all third-party GitHub Actions to immutable 40-character commit SHAs.
  - Enforced mandatory cross-platform quality gate matrix (Linux, macOS, Windows) prior to release compilation.

---

## [2.0.0] - 2026-11-03

### Added
- **Production Hardening & Remote Mirror Expansion:**
  - Expanded verified mirror registry to revision 2 (`registry/mirrors.json`) with live-tested endpoints for Myket, TarazERP, Runflare, PUB-AZS, Aliyun, Flutter-CN, Jamko, and IranServer.
  - Comprehensive provider capabilities and verified artifact routes for Flutter Storage, Dart Pub, Maven Central, Google Maven, Gradle Plugins, and Gradle Distributions.
  - Fail-closed installer checksum enforcement in `install.sh` and `install.ps1`: installations abort immediately if `checksums.txt` is missing or mismatched.
  - Automated `gradle/wrapper/gradle-wrapper.properties` distribution URL configuration and rollback for project and global Gradle distribution routing.
  - Multi-factor weighted quality benchmark engine (40% availability, 25% reliability, 20% latency, 15% throughput) with penalties preventing partially broken mirrors from being recommended as 100% healthy.
  - Cross-platform tool detection in `doctor` inspecting PATH, PATHEXT (.com, .exe, .bat, .cmd), and `where.exe` on Windows.
  - Automated CI Quality Gates workflow (`.github/workflows/ci.yml`) enforcing code style, strict linter checks, schema/signature validation, and cross-platform matrix testing on Linux, macOS, and Windows.
  - Dedicated registry verification and maintainer signing workflow (`.github/workflows/registry-signing.yml`) with zero-dependency `sign_registry` binary tool.
  - Generation-based registry caching with atomic directory rename and automatic recovery to `last_known_good_generation`.
  - Exact raw byte preservation for cached registry JSON, completely eliminating cryptographic signature invalidation caused by JSON re-serialization.
  - Monotonic anti-rollback state persisted in `$DATA_DIR/registry_state.json`, forbidding older revision acceptance even with `--force`.
  - Single-quote POSIX shell variable escaping and shell metacharacter injection protection in environment variable managers.
  - Structured rollback failure reporting returning both primary error, rollback errors, and backup directory recovery guidance.

### Changed
- Disabled unverified and failing Novin Cloud endpoints (`enabled: false`) with explicit verification status metadata.
- Upgraded `registry/schema.json` with provider capability flags, provider types, and documentation metadata.
- Replaced Gradle init script silent `catch (ignored) {}` with diagnostic logging.

---

## [2.0.0-alpha.1] - 2026-11-03

### Added
- Phase 8: Release Engineering & Supply-Chain Hardening.
  - Implemented secure `install.sh` and `install.ps1` with automated SHA-256 checksum verification.
  - Pinned GitHub Actions CI/CD workflows (`.github/workflows/ci.yml` and `.github/workflows/release.yml`).
  - Added matrix compilation targets: Linux x64, macOS Apple Silicon (arm64), macOS Intel (x64), and Windows x64.
  - Release manifest generation with SHA-256 checksums and automated tag-based release packaging.
  - Created community documentation: `CONTRIBUTING.md`, `SECURITY.md`, and complete `README.md`.
- Phase 7: Cross-Platform Validation & Binary Parity.
  - Validated cross-platform CLI execution with `assert_cmd`.
  - Verified binary alias parity between `pdown` and `irflutter`.
  - Added Windows user environment persistence support via `setx` and PowerShell hooks.
  - Multi-platform integration test suite (`tests/cross_platform_test.rs`).
- Phase 6: Profiles, System Diagnostics & Documentation.
  - Multi-tenant profile management (`default`, `iran`, `custom`).
  - Implemented `pdown profile list|use|create`.
  - Implemented `pdown doctor` comprehensive diagnostic checks (tools, permissions, conflict detection).
  - Implemented `pdown config show` displaying storage directories and active configurations.
  - Integration tests in `tests/profile_test.rs`.
  - Complete user guides: `docs/USAGE.md`, `docs/INSTALLATION.md`, and `docs/TROUBLESHOOTING.md`.
- Phase 5: Mirror Health and Multi-Metric Selection Engine.
  - Implemented multi-stage health probes: ecosystem-specific payload requests, HTTP status verification, and latency measurement.
  - Specific probes: `/api/packages/http` (Pub), `releases_linux.json` (Flutter Storage), and real POM XML downloads (Google Maven & Maven Central).
  - Reproducible ranking and automatic best-mirror recommendation.
  - Implemented `pdown test [--mirror <id>] [--timeout <secs>] [--json]`.
  - Integration test in `tests/benchmark_test.rs`.
- Phase 4: Safe Configuration Engine & Transactions.
  - Implemented transactional apply with automatic LIFO rollback upon partial failure.
  - Snapshot backup engine with SHA-256 integrity verification (`BackupManager`).
  - Safe dry-run preview (`--dry-run`) displaying color-unified diffs without modifying files.
  - Atomic file writing preventing partial corruption during system crash or power-loss.
  - Strict ownership tags (`# >>> Managed by pdown >>>`).
  - Coalesced multi-ecosystem environment plans for Flutter, Dart, Gradle, and Android.
  - Implemented `pdown use <id> [--only <providers>] [--dry-run]` and `pdown reset [--only <providers>] [--dry-run]`.
  - Transaction integration tests (`tests/transaction_test.rs`).
- Phase 3: Provider Implementations & Ecosystem Boundaries.
  - Implemented modular `Provider` trait and engine.
  - `FlutterProvider`: Handles `FLUTTER_STORAGE_BASE_URL` with verification that the target mirror actually hosts `flutter_infra_release` SDK artifacts.
  - `DartProvider`: Handles `PUB_HOSTED_URL` for pub.dev package proxying.
  - `GradleProvider`: Non-destructively manages `~/.gradle/init.d/pdown-mirror.gradle` and timeout properties in `gradle.properties`. Solved Maven Central vs Google Maven collisions.
  - `AndroidProvider`: Handles Google Maven and Android SDK artifact proxying.
  - `IosProvider`: Guarded unsupported provider with informative diagnostic explanation.
  - Implemented `pdown status` across all providers with human-readable and `--json` modes.
  - Provider integration tests in `tests/providers_test.rs`.
  - Comprehensive provider documentation in `docs/PROVIDERS.md`.
- Phase 2: Remote Mirror Registry & Cryptographic Verification.
  - Implemented 3-tier registry resolution: Bundled snapshot, persistent Cache, and Remote GitHub raw endpoint.
  - Ed25519 digital signature generation and verification (`mirrors.json.sig`).
  - Monotonic revision comparison to prevent rollback and replay attacks.
  - Verified mirror catalog: `pub-azs`, `runflare`, `flutter-cn`, `jamko`, `iranserver`, and upstream `official`.
  - JSON schema draft 2020-12 validation (`registry/schema.json`).
  - Implemented commands: `pdown list`, `pdown mirror info <id>`, `pdown registry status|check|update`.
  - Comprehensive unit and integration test suite (`tests/registry_test.rs`).
  - Documented registry format in `docs/REGISTRY_SPECIFICATION.md` and security in `docs/REGISTRY_SECURITY.md`.
- Phase 1: Rust architecture and foundation (`package-downloader` v2.0.0).
  - Configured native binaries `pdown` and alias `irflutter`.
  - Comprehensive CLI definition using `clap` (derive).
  - Error hierarchy using `thiserror` with standardized POSIX exit codes.
  - Core domain models for registries, provider contracts, profiles, diffs, and benchmarks.
  - Infrastructure layer: multi-platform storage paths, Ed25519 signature engine, HTTP client, and `tracing` logging.
  - Enforced zero clippy warnings and strict code formatting.
- Phase 0: Complete repository audit and verification of mirror endpoints (`docs/audit/REPOSITORY_AUDIT.md`).
- Software architecture specification (`docs/architecture/SOFTWARE_ARCHITECTURE.md`).
- Technical design blueprint (`docs/architecture/TECHNICAL_DESIGN.md`).
- Engineering roadmap (`docs/ROADMAP.md`).
- Identified and documented critical defects in legacy v1.x:
  - PUB-AZS does not support Flutter Storage artifacts (`FLUTTER_STORAGE_BASE_URL`).
  - Blind Gradle rewrite in v1.x broke Google Maven repositories for Maven Central-only mirrors like Runflare.
  - Insecure shell and `init.gradle` file mutation without backups or transactions.

## [1.1.0] - 2026-10-28
- Legacy Dart CLI release with support for mirrors from ramineslami.ir (Runflare, Jamko, IranServer, ITO).

## [1.0.0] - 2026-10-28
- Initial Dart CLI implementation.
