# PackageDownloader 2.0 — CLI Usage Guide

**Binary:** `pdown` (alias `irflutter`)  
**License:** MIT

---

## 1. Quick Start

### 1.1 List Available Mirrors
Inspect all certified development mirror endpoints:
```bash
pdown list
```
Filter by ecosystem:
```bash
pdown list --provider flutter
pdown list --provider gradle
```

### 1.2 Inspect Current System Status
View active environment variables, managed init scripts, and current routing:
```bash
pdown status
```

### 1.3 Test & Benchmark Mirrors
Execute real-time multi-metric reachability and latency benchmarks:
```bash
pdown test
```
Benchmark a specific mirror with a custom timeout:
```bash
pdown test --mirror pub-azs --timeout 8
```

---

## 2. Activating Mirrors

### 2.1 Full Activation (Flutter + Dart + Gradle + Android)
Activate all supported ecosystems for a certified mirror:
```bash
pdown use runflare
```

### 2.2 Ecosystem-Specific Activation (`--only`)
Configure only Dart/Pub:
```bash
pdown use pub-azs --only dart
```
Configure only Gradle and Android dependencies:
```bash
pdown use jamko --only gradle
```

### 2.3 Safe Preview (`--dry-run`)
View an exact unified diff of all proposed modifications before touching any file:
```bash
pdown use runflare --dry-run
```

---

## 3. Reverting Configuration

Reset all settings to official Google, pub.dev, and Maven Central defaults:
```bash
pdown reset
```
Preview reset changes safely:
```bash
pdown reset --dry-run
```

---

## 4. Diagnostics & Troubleshooting

Run system environment diagnostic checks:
```bash
pdown doctor
```
Inspect current storage directories, active profile, and public key anchor:
```bash
pdown config show
```

---

## 5. Remote Registry Management

Inspect local registry revision and cryptographic signature:
```bash
pdown registry status
```
Check for online registry updates on GitHub:
```bash
pdown registry check
```
Download, verify Ed25519 signature, and atomically update cache:
```bash
pdown registry update
```

---

## 6. Profiles Management

List available profiles:
```bash
pdown profile list
```
Create a new profile:
```bash
pdown profile create project-a
```
Switch active profile:
```bash
pdown profile use project-a
```

---

## 7. JSON Mode for IDEs & Automation

All commands support `--json` for automation and script integration:
```bash
pdown list --json
pdown status --json
pdown test --json
pdown doctor --json
pdown registry status --json
```
