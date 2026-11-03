# PackageDownloader Registry Security Architecture

**Version:** 2.0.0 (Production Hardened)  
**Algorithm:** Ed25519 (PureEd25519, RFC 8032)  
**Embedded Trust Anchor:** `EMBEDDED_REGISTRY_PUBLIC_KEY` in `src/infrastructure/crypto.rs`  
**Current Trust Anchor (Base64):** `qa953ivnWXnctJ2hawKiWTEPQhE3t/970TKNsi6fEao=`

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

## 3. Emergency Administrative Reset

If an administrator or maintainer needs to reset local anti-rollback tracking to the bundled baseline (e.g., following testing or manual intervention):

```bash
pdown registry reset-anti-rollback
```

This resets `$DATA_DIR/registry_state.json` to the embedded binary revision baseline while preserving cryptographic verification integrity.
