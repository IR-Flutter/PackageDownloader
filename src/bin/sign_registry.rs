use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use ed25519_dalek::SigningKey;
use pdown::core::registry::RegistryManager;
use pdown::infrastructure::crypto::sign_payload;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let mirrors_path = Path::new("registry/mirrors.json");
    let sig_path = Path::new("registry/mirrors.json.sig");

    if !mirrors_path.exists() {
        bail!("registry/mirrors.json not found in working directory");
    }

    let json_bytes = fs::read(mirrors_path).context("Failed to read registry/mirrors.json")?;

    // 1. JSON Schema validation
    RegistryManager::validate_json_schema(&json_bytes)
        .context("registry/mirrors.json failed JSON Schema validation")?;

    // 2. Semantic validation
    let doc: pdown::core::models::RegistryDocument =
        serde_json::from_slice(&json_bytes).context("Failed to parse registry JSON")?;
    RegistryManager::validate_semantics(&doc)
        .context("registry/mirrors.json failed semantic validation")?;

    // 3. Obtain signing key from environment or file argument
    let key_b64 = match env::var("REGISTRY_SIGNING_PRIVATE_KEY") {
        Ok(k) if !k.trim().is_empty() => k.trim().to_string(),
        _ => {
            if let Some(arg) = env::args().nth(1) {
                fs::read_to_string(arg)?.trim().to_string()
            } else {
                bail!("REGISTRY_SIGNING_PRIVATE_KEY environment variable or key file path argument is required");
            }
        }
    };

    let key_bytes = BASE64
        .decode(&key_b64)
        .context("Private key must be valid base64")?;
    let key_arr: [u8; 32] = key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Private key must be exactly 32 bytes"))?;
    let signing_key = SigningKey::from_bytes(&key_arr);

    let signature = sign_payload(&json_bytes, &signing_key);
    fs::write(sig_path, format!("{}\n", signature))
        .context("Failed to write registry/mirrors.json.sig")?;

    println!(
        "✅ Successfully signed registry/mirrors.json (revision {})",
        doc.revision
    );
    println!("   Signature written to registry/mirrors.json.sig");
    Ok(())
}
