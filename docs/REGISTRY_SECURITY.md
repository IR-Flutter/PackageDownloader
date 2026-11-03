# PackageDownloader Registry Security Architecture

**Version:** 1.0.0  
**Algorithm:** Ed25519 (PureEd25519, RFC 8032)  
**Embedded Trust Anchor:** `EMBEDDED_REGISTRY_PUBLIC_KEY` in `src/infrastructure/crypto.rs`

---

## 1. Threat Model & Mitigations

| Threat | Attack Vector | Security Countermeasure in PackageDownloader 2.0 |
| :--- | :--- | :--- |
| **Malicious Registry Injection** | Attacker compromises GitHub repository or branch and alters JSON | Registry client strictly verifies detached Ed25519 signature (`mirrors.json.sig`) against embedded public key. Unsigned/tampered JSON is discarded. |
| **Replay / Rollback Attack** | Attacker serves an older, vulnerable registry revision | Monotonically increasing `revision` check enforces forward-only updates. `revision <= active.revision` is rejected. |
| **Denial of Service via Network Failure** | GitHub raw endpoints blocked or unreachable | Application automatically falls back to valid local cached registry, and finally to offline bundled snapshot. Network failures never crash the tool. |
| **Cache Corruption via Crash** | System power-loss during registry write | Cache updates write to an isolated temporary file in `$DATA_DIR/cache/` followed by an atomic OS-level filesystem rename. |
| **Malformed Payload Crash** | Schema corruption or memory exhaustion | Schema verification validates types and bounds prior to deserializing provider logic. |

---

## 2. Key Management & Release Signing Process

1. **Private Signing Key Safety:**
   - The Ed25519 private signing key is **never** committed to version control.
   - It is stored as an encrypted GitHub Actions secret (`REGISTRY_SIGNING_KEY`).
2. **Automated CI/CD Signing:**
   - When a release tag is pushed or a PR modifying `registry/mirrors.json` is merged into `main`, GitHub Actions validates the schema, signs `mirrors.json`, commits `mirrors.json.sig`, and verifies the signature with the public key.
3. **Key Rotation Procedure:**
   - To rotate keys, a new application release introduces a transitional key list where both the legacy key and the new key are authorized. Once deployed, the primary key transitions to the new key.
