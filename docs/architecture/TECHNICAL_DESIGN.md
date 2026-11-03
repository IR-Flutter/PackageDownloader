# PackageDownloader 2.0 — Technical Design Specification

**Status:** Implementation Blueprint  
**Target:** PackageDownloader 2.0 (Rust)

---

## 1. Domain Models & Data Structures

### 1.1 Registry Schema (`registry/schema.json`)

The canonical registry JSON structure is versioned and strictly validated.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistryDocument {
    pub schema_version: u32,
    pub registry_version: String,
    pub revision: u64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub mirrors: Vec<MirrorDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MirrorDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub website: String,
    pub enabled: bool,
    pub priority: Option<u32>,
    pub providers: MirrorProviders,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MirrorProviders {
    pub flutter: Option<FlutterProviderConfig>,
    pub dart: Option<DartProviderConfig>,
    pub gradle: Option<GradleProviderConfig>,
    pub android: Option<AndroidProviderConfig>,
    pub ios: Option<IosProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlutterProviderConfig {
    pub enabled: bool,
    pub storage_url: String,
    pub test_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DartProviderConfig {
    pub enabled: bool,
    pub hosted_url: String,
    pub test_package: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradleProviderConfig {
    pub enabled: bool,
    pub maven_central_url: Option<String>,
    pub google_maven_url: Option<String>,
    pub gradle_plugins_url: Option<String>,
    pub distribution_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AndroidProviderConfig {
    pub enabled: bool,
    pub google_maven_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IosProviderConfig {
    pub enabled: bool,
    pub notes: Option<String>,
}
```

---

## 2. Remote Registry & Cryptographic Verification

### 2.1 Canonical Serialization for Ed25519 Signing
To guarantee deterministic signature verification regardless of JSON key ordering or whitespace:
1. Strip the `signature` field (if stored in a wrapper object) or use a detached signature file (`registry/mirrors.json.sig`).
2. Canonicalize the JSON payload using RFC 8785 (JSON Canonicalization Scheme) or a deterministic key-sorted serializer.
3. Compute the Ed25519 signature over `SHA512(canonical_payload)` or direct Ed25519 message signing.
4. Base64-encode the resulting 64-byte Ed25519 signature.

### 2.2 Dynamic Registry Fetch Algorithm
```text
1. Read Bundled Registry from binary assets (include_str!("registry/mirrors.json")).
2. Check local cache: $DATA_DIR/cache/mirrors.json and mirrors.json.sig.
3. If cache exists:
     a. Verify signature with embedded Ed25519 public key.
     b. Verify schema_version compatibility.
     c. If valid and revision >= bundled.revision, active_registry = cached.
4. If online check requested (or pdown registry update):
     a. Download https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/registry/mirrors.json
     b. Download https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/registry/mirrors.json.sig
     c. Verify signature.
     d. Verify schema_version matches application capability.
     e. Verify revision > active_registry.revision (reject rollback attacks).
     f. Atomically write to cache file.
     g. Switch active_registry to new remote version.
5. In all cases, if remote or cache validation fails, log a warning and fall back to the last known valid local registry without crashing.
```

---

## 3. Safe Configuration Management Engine

### 3.1 Ownership Markers & Atomic Writes
Any file modified by `pdown` (e.g. `~/.zshrc`, `~/.bashrc`, `~/.gradle/init.d/pdown-mirror.gradle`, `~/.gradle/gradle.properties`) must follow strict ownership conventions:

```bash
# >>> Managed by pdown [Ecosystem: Flutter/Dart] [Profile: iran] >>>
export PUB_HOSTED_URL="https://pub-azs.ir"
export FLUTTER_STORAGE_BASE_URL="https://mirror-gcs.runflare.com"
# <<< Managed by pdown [Ecosystem: Flutter/Dart] <<<
```

### 3.2 Transaction & Rollback Stack
```rust
pub struct ConfigurationTransaction {
    pub id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub backup_records: Vec<BackupRecord>,
    pub rollback_actions: Vec<Box<dyn FnOnce() -> Result<()> + Send>>,
}
```
1. **Pre-flight Planning:** Determine required changes across all selected providers. Generate diffs.
2. **Snapshot Creation:** Before writing, copy target files to `$DATA_DIR/backups/<timestamp>-<filename>`. Compute SHA-256 before write.
3. **Write Execution:** Write new content to a temporary file (`tempfile::NamedTempFile`) in the same directory, then perform an atomic replace.
4. **Error Interception:** If any step fails, unwind `rollback_actions` in LIFO order and restore backed-up files. Report partial failure details explicitly.

### 3.3 Gradle Integration Strategy
Unlike v1.x which clobbered `~/.gradle/init.gradle`, `pdown 2.0` utilizes Gradle's built-in `init.d` auto-discovery directory:
`~/.gradle/init.d/pdown-mirror.gradle`
This guarantees:
- User's primary `~/.gradle/init.gradle` remains completely untouched.
- Clean removal upon `pdown reset` (simple file removal).
- Fine-grained URL rewriting separating:
  - Maven Central -> `maven_central_url`
  - Google Maven -> `google_maven_url`
  - Gradle Plugin Portal -> `gradle_plugins_url`

---

## 4. Benchmark & Health Check Engine

### 4.1 Multi-Stage Evaluation Pipeline
1. **DNS Resolution:** Measure latency to resolve domain via system resolver.
2. **TCP Connect:** Measure time to establish TCP 3-way handshake to port 443.
3. **TLS Handshake:** Measure TLS 1.3/1.2 handshake latency and validate server certificate.
4. **Target Endpoint Probe:**
   - **Pub:** `GET <hosted_url>/api/packages/http` (Expect 200 and valid JSON with `"name":"http"`).
   - **Flutter Storage:** `GET <storage_url>/flutter_infra_release/releases/releases_linux.json` (Expect 200 and valid JSON with `"current_release"`).
   - **Google Maven:** `GET <google_maven_url>/androidx/appcompat/appcompat/1.6.1/appcompat-1.6.1.pom` (Expect 200 and XML `<project>`).
   - **Maven Central:** `GET <maven_central_url>/org/jetbrains/kotlin/kotlin-stdlib/1.9.0/kotlin-stdlib-1.9.0.pom` (Expect 200).
5. **Score Metric:** Combined weighted latency formula:
   $$\text{Score} = \text{DNS} + \text{TCP} + \text{TLS} + \text{TTFB} + (\text{failure\_rate} \times 10000)$$

---

## 5. CLI Specification & Exit Codes

### 5.1 Command Tree
```text
pdown
├── list [--json] [--verbose]
├── status [--json]
├── test [--json] [--timeout <ms>]
├── use <mirror-id> [--only <providers>] [--dry-run] [--scope <global|project>]
├── reset [--only <providers>] [--dry-run]
├── doctor [--json]
├── mirror info <mirror-id> [--json]
├── registry
│   ├── status
│   ├── check
│   └── update [--force]
└── profile
    ├── list
    ├── use <profile-name>
    └── create <profile-name>
```

### 5.2 Process Exit Codes
- `0`: Success
- `1`: General Error (unhandled failure)
- `2`: Invalid CLI Argument or Usage
- `3`: Mirror Not Found
- `4`: Provider Unsupported or Incompatible
- `5`: Network / Registry Fetch Failure
- `6`: Cryptographic Verification Failure
- `7`: Transaction / Rollback Failure
