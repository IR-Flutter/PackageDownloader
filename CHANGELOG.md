# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0-alpha.1] - 2026-11-03

### Added
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
