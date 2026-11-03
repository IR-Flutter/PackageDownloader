# Security Policy

## 1. Supported Versions

Security updates and patches are actively maintained for the following versions:

| Version | Supported          |
| :---    | :---               |
| 2.0.x   | :white_check_mark: |
| 1.x     | :x: (Deprecated)   |

---

## 2. Security Design Principles

PackageDownloader 2.0 incorporates defense-in-depth architectural safeguards:

1. **Ed25519 Cryptographic Verification:** All remote registry updates (`mirrors.json`) must be cryptographically signed by the official project release key. Unsigned, modified, or tampered registries are discarded immediately.
2. **Replay & Rollback Protection:** The registry loader enforces monotonically increasing revisions to prevent downgrade attacks.
3. **Transactional Safety:** File modifications are atomic (`tempfile` + atomic rename) and accompanied by automatic SHA-256 verified pre-write backups in `~/.local/share/pdown/backups`.
4. **No Privilege Escalation:** The tool executes strictly in user space without requiring root (`sudo`) or Administrator permissions.
5. **No Supply-Chain Blind Trust:** Installer scripts verify SHA-256 checksums from signed manifests before writing binaries to executable paths.

---

## 3. Reporting a Vulnerability

If you discover a potential security vulnerability in PackageDownloader:

1. **Do not create a public GitHub issue.**
2. Send an email to the security response team at: `security@irflutter.dev` (or open a Private Vulnerability Advisory on GitHub).
3. Include details of the vulnerability, reproduction steps, and potential impact.
4. We will acknowledge receipt within 48 hours and work with you on a coordinated disclosure timeline.
