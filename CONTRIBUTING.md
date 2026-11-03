# Contributing to PackageDownloader

Thank you for your interest in contributing to PackageDownloader (`pdown`)! We welcome contributions, bug fixes, mirror endpoint updates, and documentation enhancements.

---

## 1. Development Guidelines

1. **Language & Toolchain:**
   - Rust 2021 Edition (MSRV 1.75+).
   - Strict static analysis: zero warnings permitted (`cargo clippy --all-targets --all-features -- -D warnings`).
   - Formatting: strict adherence to `cargo fmt --check`.

2. **Pre-commit Checklist:**
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```

3. **Branching & Commits:**
   - Use conventional commit messages (`feat: ...`, `fix: ...`, `docs: ...`, `test: ...`).
   - Create pull requests against the `main` branch.

---

## 2. Adding or Modifying Mirrors in the Registry

The mirror catalog is maintained in `registry/mirrors.json`. To submit a new mirror or update an existing endpoint:

1. **Technical Verification is Mandatory:**
   - You must verify that each proposed endpoint is reachable and serves expected artifact formats.
   - Do **NOT** assume that a mirror supporting Dart Pub also mirrors Flutter SDK storage or Gradle Google Maven.
   - Provide `curl` verification evidence in your pull request description.
2. **JSON Schema Compatibility:**
   - Ensure `registry/mirrors.json` strictly conforms to `registry/schema.json`.
3. **Monotonic Revision:**
   - Increment `revision` by at least `1`.
   - Update `updated_at` with current UTC timestamp.
4. **Signature:**
   - CI will automatically verify and sign updated registries upon maintainer review.

---

## 3. Code of Conduct

Please be respectful and constructive in all discussions, issues, and code reviews.
