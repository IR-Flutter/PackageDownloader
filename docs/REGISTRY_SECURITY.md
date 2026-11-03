# PackageDownloader Registry Security Architecture

**Version:** 2.0.1 (Production Hardened)  
**Algorithm:** Ed25519 (PureEd25519, RFC 8032)  
**Embedded Trust Anchor:** `EMBEDDED_REGISTRY_PUBLIC_KEY` in `src/infrastructure/crypto.rs`  
**Current Trust Anchor (Base64):** `NuU4h8pLwVEL8yA9rUBhgr/ru15QnaXmAxl8N9u/QjY=`

---

## 1. Threat Model & Production Mitigations

| Threat | Attack Vector | Security Countermeasure in PackageDownloader 2.0 |
| :--- | :--- | :--- |
| **Malicious Registry Injection** | Attacker tampers with CDN or network transit and alters JSON | Detached Ed25519 signature (`mirrors.json.sig`) is verified strictly against exact raw downloaded bytes before any staging. |
| **Signature Invalidation via Re-Serialization** | Reserializing altered indentation, whitespace, or object keys | **Exact Bytes Cache**: The application stores raw signed bytes bit-for-bit without round-trip re-serialization. |
| **Rollback / Downgrade Attack** | Attacker serves an older, signed registry revision | **Persistent Monotonic Anti-Rollback**: Monotonic counter `highest_accepted_revision` is saved in `$DATA_DIR/registry_state.json`. Revisions `< highest_accepted_revision` are rejected unconditionally—even if `--force` is passed. |
| **Cache Corruption via Power Failure** | Power loss or crash during file write | **Generation-Based Atomic Activation**: Generations are assembled in a staging folder (`.staging`), verified cryptographically in place, and atomically renamed to the generation directory. |
| **Active Generation Corruption** | Disk bit-rot or file corruption | **Last-Known-Good Recovery**: If the active generation fails integrity verification, the engine automatically falls back to `last_known_good_generation`, then to the immutable bundled binary registry. |
| **Denial of Service via Network Failure** | GitHub raw endpoints blocked or unreachable | Application transparently serves the active verified local generation or bundled fallback. Zero crashes or unhandled panics. |
| **Semantic & Schema Exploits** | Dangerous URL schemes (`javascript:`, `file:`, relative paths) | Deep semantic validation (`RegistryManager::validate_semantics`) enforces strict URL parsing, `https://`/`http://` scheme checks, priority bounds (1..=1000), duplicate ID rejections, and endpoint completeness. |

---

## 2. Key Management & Release Signing Process

1. **Private Signing Key Safety:**
   - The Ed25519 private signing key is **never** committed to version control or included in binary builds.
   - It is managed as a cryptographically protected GitHub Actions repository secret (`REGISTRY_SIGNING_PRIVATE_KEY`).
2. **Automated CI/CD Signing:**
   - Verified pull requests and pushes to `main` touching `registry/**` trigger `.github/workflows/registry-signing.yml`.
   - The workflow compiles `src/bin/sign_registry.rs`, validates the schema, verifies semantic rules, signs `registry/mirrors.json`, and outputs `registry/mirrors.json.sig`.
3. **Canonical Verification:**
   - CI Quality Gates (`.github/workflows/ci.yml`) run `cargo test --test registry_test` across Linux, macOS, and Windows to verify that the committed `registry/mirrors.json` matches `registry/mirrors.json.sig` against the embedded trust anchor.

---

## 3. Key Rotation and Disaster Recovery Procedure

### 3.1 Scheduled Key Rotation
To rotate the registry signing key without breaking client verification:
1. Generate a new Ed25519 keypair using `pdown::infrastructure::crypto::generate_keypair()`.
2. Securely store the new private key in GitHub repository secret `REGISTRY_SIGNING_PRIVATE_KEY`.
3. In the next application release (`v2.x`), update `EMBEDDED_REGISTRY_PUBLIC_KEY` in `src/infrastructure/crypto.rs` to the new public key.
4. Sign `registry/mirrors.json` with the new private key and commit `registry/mirrors.json.sig`.

### 3.2 Compromise Recovery
If the private signing key is suspected of compromise:
1. **Immediate Revocation**: Immediately revoke and rotate the secret `REGISTRY_SIGNING_PRIVATE_KEY` in GitHub Repository Settings.
2. **Emergency Patch Release**: Bump application patch release with the new embedded public key and incremented registry revision.
3. **Anti-Rollback Forward Leap**: Increment registry `revision` to exceed any revision signed by the compromised key. The application's forward-only anti-rollback engine will permanently reject any replays from the compromised generation.

---

## 4. Branch Protection and CI/CD Security Policy

All production releases and canonical registry changes adhere to the following mandatory branch protections on `main`:
1. **Require Pull Request Reviews**: Direct pushes by non-maintainers are blocked.
2. **Require Status Checks to Pass Before Merging**:
   - `lint-and-format`: `cargo fmt --check` and `cargo clippy -- -D warnings` must pass.
   - `registry-verification`: Schema and cryptographic signature verification must pass.
   - `cross-platform-test-matrix`: Full test suite on Linux, macOS, and Windows must pass.
3. **Immutable Action Pinning**: All third-party GitHub Actions are pinned to full 40-character commit SHAs.
4. **Isolated Release Gate**: Release workflows require the `pre-release-quality-gate` cross-platform matrix to pass completely before binaries are built and published.

---

## 5. Emergency Administrative Reset

If an administrator or maintainer needs to reset local anti-rollback tracking to the bundled baseline (e.g., following testing or manual intervention):

```bash
pdown registry reset-anti-rollback
```

This resets `$DATA_DIR/registry_state.json` to the embedded binary revision baseline while preserving cryptographic verification integrity.
