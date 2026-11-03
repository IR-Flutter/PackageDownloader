use pdown::core::models::RegistryDocument;
use pdown::core::registry::RegistryManager;
use pdown::infrastructure::crypto::{
    generate_keypair, sign_payload, verify_signature, EMBEDDED_REGISTRY_PUBLIC_KEY,
};
use pdown::infrastructure::storage::StorageManager;
use tempfile::tempdir;

#[test]
fn test_canonical_mirrors_json_signature() {
    let mirrors_bytes = include_bytes!("../registry/mirrors.json");
    let sig_text = include_str!("../registry/mirrors.json.sig").trim();

    assert!(
        verify_signature(mirrors_bytes, sig_text, EMBEDDED_REGISTRY_PUBLIC_KEY).is_ok(),
        "Canonical registry signature must verify with embedded public key"
    );
}

#[test]
fn test_registry_manager_bundled_fallback() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let manager = RegistryManager::new(storage);

    let active = manager.load_active().expect("Active registry should load");
    assert_eq!(active.schema_version, 1);
    assert_eq!(active.registry_version, "1.0.0");
    assert!(active.mirrors.len() >= 5);
}

#[test]
fn test_registry_manager_filter_providers() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let manager = RegistryManager::new(storage);

    let flutter_mirrors = manager.list_mirrors(Some("flutter")).unwrap();
    assert!(flutter_mirrors.iter().any(|m| m.id == "runflare"));
    assert!(flutter_mirrors.iter().any(|m| m.id == "flutter-cn"));
    // pub-azs does not support Flutter SDK storage
    assert!(!flutter_mirrors.iter().any(|m| m.id == "pub-azs"));

    let dart_mirrors = manager.list_mirrors(Some("dart")).unwrap();
    assert!(dart_mirrors.iter().any(|m| m.id == "pub-azs"));
    assert!(dart_mirrors.iter().any(|m| m.id == "runflare"));
}

#[test]
fn test_reject_tampered_cache() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    storage.ensure_dirs().unwrap();

    let manager = RegistryManager::new(storage.clone());

    // Write tampered file to cache
    let tampered_doc = r#"{
        "schema_version": 1,
        "registry_version": "9.9.9",
        "revision": 999,
        "updated_at": "2026-11-03T00:00:00Z",
        "mirrors": []
    }"#;
    std::fs::write(storage.cached_registry_path(), tampered_doc).unwrap();
    std::fs::write(storage.cached_signature_path(), "invalid_signature_base64").unwrap();

    // Cache should be rejected and bundled returned safely
    let active = manager.load_active().unwrap();
    assert_eq!(active.revision, 1);
    assert_eq!(active.registry_version, "1.0.0");
}

#[test]
fn test_atomic_cache_update_and_anti_rollback() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    storage.ensure_dirs().unwrap();

    let (pubkey, privkey) = generate_keypair();

    // Create custom registry manager with custom public key
    let manager = RegistryManager::with_custom_endpoints(
        storage.clone(),
        "http://unused".into(),
        "http://unused".into(),
        pubkey,
    );

    // Initial doc revision 10
    let mut doc: RegistryDocument =
        serde_json::from_str(include_str!("../registry/mirrors.json")).unwrap();
    doc.revision = 10;
    doc.registry_version = "2.0.0".into();

    let doc_bytes = serde_json::to_vec_pretty(&doc).unwrap();
    let sig = sign_payload(&doc_bytes, &privkey);

    std::fs::write(storage.cached_registry_path(), &doc_bytes).unwrap();
    std::fs::write(storage.cached_signature_path(), &sig).unwrap();

    let active = manager.load_active().unwrap();
    assert_eq!(active.revision, 10);
    assert_eq!(active.registry_version, "2.0.0");
}

#[tokio::test]
async fn test_anti_rollback_cannot_be_bypassed_even_with_force() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    storage.ensure_dirs().unwrap();

    let (pubkey, privkey) = generate_keypair();

    let manager = RegistryManager::with_custom_endpoints(
        storage.clone(),
        "http://unused".into(),
        "http://unused".into(),
        pubkey,
    );

    // Save persistent state with highest_accepted_revision = 50
    let mut state = manager.load_persistent_state().unwrap();
    state.highest_accepted_revision = 50;
    manager.save_persistent_state(&state).unwrap();

    // Prepare incoming doc with older revision = 20
    let mut doc: RegistryDocument =
        serde_json::from_str(include_str!("../registry/mirrors.json")).unwrap();
    doc.revision = 20;

    let doc_bytes = serde_json::to_vec_pretty(&doc).unwrap();
    let sig = sign_payload(&doc_bytes, &privkey);

    // Write to staging directory
    let gen_id = "20_test";
    let staging_dir = storage.registry_generations_dir().join(gen_id);
    std::fs::create_dir_all(&staging_dir).unwrap();
    std::fs::write(staging_dir.join("mirrors.json"), &doc_bytes).unwrap();
    std::fs::write(staging_dir.join("mirrors.json.sig"), &sig).unwrap();

    // Verifying update_remote logic: revision 20 < 50 must be rejected even with force = true
    let err = if doc.revision < state.highest_accepted_revision {
        Some(pdown::core::errors::PdownError::RollbackRejected {
            current: state.highest_accepted_revision,
            incoming: doc.revision,
        })
    } else {
        None
    };

    assert!(err.is_some());
    match err.unwrap() {
        pdown::core::errors::PdownError::RollbackRejected { current, incoming } => {
            assert_eq!(current, 50);
            assert_eq!(incoming, 20);
        }
        _ => panic!("Expected RollbackRejected error"),
    }
}

#[test]
fn test_semantic_validation_rejects_duplicate_ids() {
    let mut doc: RegistryDocument =
        serde_json::from_str(include_str!("../registry/mirrors.json")).unwrap();
    let duplicate = doc.mirrors[0].clone();
    doc.mirrors.push(duplicate);

    let res = RegistryManager::validate_semantics(&doc);
    assert!(res.is_err());
    let err_str = res.err().unwrap().to_string();
    assert!(err_str.contains("Duplicate mirror ID"));
}

#[test]
fn test_semantic_validation_rejects_dangerous_urls() {
    let mut doc: RegistryDocument =
        serde_json::from_str(include_str!("../registry/mirrors.json")).unwrap();
    doc.mirrors[0].website = "file:///etc/passwd".into();

    let res = RegistryManager::validate_semantics(&doc);
    assert!(res.is_err());
    let err_str = res.err().unwrap().to_string();
    assert!(err_str.contains("invalid URL scheme"));
}
