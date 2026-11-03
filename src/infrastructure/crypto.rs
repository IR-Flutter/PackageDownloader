use crate::core::errors::{PdownError, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// Embedded trusted Ed25519 public key for verifying registry authenticity.
/// Base64 encoded 32-byte Ed25519 public key.
pub const EMBEDDED_REGISTRY_PUBLIC_KEY: &str = "NuU4h8pLwVEL8yA9rUBhgr/ru15QnaXmAxl8N9u/QjY="; // Production Ed25519 registry trust anchor

/// Verify an Ed25519 signature over a byte payload.
pub fn verify_signature(payload: &[u8], signature_b64: &str, public_key_b64: &str) -> Result<()> {
    let signature_bytes = BASE64.decode(signature_b64.trim()).map_err(|e| {
        PdownError::SignatureVerification(format!("Malformed base64 signature: {e}"))
    })?;

    if signature_bytes.len() != 64 {
        return Err(PdownError::SignatureVerification(format!(
            "Invalid signature length: expected 64 bytes, got {}",
            signature_bytes.len()
        )));
    }

    let pubkey_bytes = BASE64.decode(public_key_b64.trim()).map_err(|e| {
        PdownError::SignatureVerification(format!("Malformed base64 public key: {e}"))
    })?;

    let pubkey_array: [u8; 32] = pubkey_bytes.try_into().map_err(|_| {
        PdownError::SignatureVerification("Public key must be exactly 32 bytes".into())
    })?;

    let verifying_key = VerifyingKey::from_bytes(&pubkey_array).map_err(|e| {
        PdownError::SignatureVerification(format!("Invalid Ed25519 public key: {e}"))
    })?;

    let signature = Signature::from_slice(&signature_bytes).map_err(|e| {
        PdownError::SignatureVerification(format!("Invalid Ed25519 signature format: {e}"))
    })?;

    verifying_key.verify(payload, &signature).map_err(|e| {
        PdownError::SignatureVerification(format!("Signature verification failed: {e}"))
    })?;

    Ok(())
}

/// Sign a payload using an Ed25519 signing key (used in tests and registry publishing).
pub fn sign_payload(payload: &[u8], signing_key: &SigningKey) -> String {
    let signature = signing_key.sign(payload);
    BASE64.encode(signature.to_bytes())
}

/// Generate a new random Ed25519 keypair (returns (base64_pubkey, signing_key)).
pub fn generate_keypair() -> (String, SigningKey) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let pubkey = signing_key.verifying_key();
    let pubkey_b64 = BASE64.encode(pubkey.to_bytes());
    (pubkey_b64, signing_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_roundtrip() {
        let (pubkey, privkey) = generate_keypair();
        let payload = b"hello from package downloader registry";

        let sig = sign_payload(payload, &privkey);
        assert!(verify_signature(payload, &sig, &pubkey).is_ok());

        // Tamper test
        let tampered = b"tampered registry content";
        assert!(verify_signature(tampered, &sig, &pubkey).is_err());
    }

    #[test]
    fn test_verify_canonical_mirrors_json() {
        let mirrors_bytes = include_bytes!("../../registry/mirrors.json");
        let sig = include_str!("../../registry/mirrors.json.sig").trim();
        assert!(verify_signature(mirrors_bytes, sig, EMBEDDED_REGISTRY_PUBLIC_KEY).is_ok());

        // Verification must fail if public key or payload is altered
        assert!(verify_signature(b"tampered", sig, EMBEDDED_REGISTRY_PUBLIC_KEY).is_err());
    }
}
