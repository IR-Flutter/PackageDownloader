use pdown::core::provider::Provider;
use pdown::core::registry::RegistryManager;
use pdown::infrastructure::storage::StorageManager;
use pdown::providers::{
    all_providers, get_provider, DartProvider, FlutterProvider, GradleProvider, IosProvider,
};
use tempfile::tempdir;

#[test]
fn test_all_providers_registration() {
    let providers = all_providers();
    assert_eq!(providers.len(), 5);

    assert!(get_provider("flutter").is_some());
    assert!(get_provider("dart").is_some());
    assert!(get_provider("gradle").is_some());
    assert!(get_provider("android").is_some());
    assert!(get_provider("ios").is_some());
    assert!(get_provider("unknown").is_none());
}

#[test]
fn test_flutter_provider_rejects_incompatible_mirror() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let reg = RegistryManager::new(storage);

    let pub_azs = reg.find_mirror("pub-azs").unwrap().unwrap();
    let provider = FlutterProvider;

    // pub-azs does not support Flutter storage artifacts; must be rejected!
    let plan = provider.plan_apply(&pub_azs);
    assert!(
        plan.is_err(),
        "Flutter provider must reject pub-azs storage configuration"
    );
}

#[test]
fn test_flutter_provider_accepts_valid_mirror() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let reg = RegistryManager::new(storage);

    let runflare = reg.find_mirror("runflare").unwrap().unwrap();
    let provider = FlutterProvider;

    let plan = provider
        .plan_apply(&runflare)
        .expect("Runflare supports Flutter");
    assert_eq!(plan.provider_id, "flutter");
    assert!(!plan.items.is_empty());
}

#[test]
fn test_dart_provider_plan_apply_and_revert() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let reg = RegistryManager::new(storage);

    let pub_azs = reg.find_mirror("pub-azs").unwrap().unwrap();
    let provider = DartProvider;

    let apply_plan = provider.plan_apply(&pub_azs).unwrap();
    assert_eq!(apply_plan.provider_id, "dart");
    assert!(!apply_plan.items.is_empty());

    let revert_plan = provider.plan_revert().unwrap();
    assert_eq!(revert_plan.provider_id, "dart");
}

#[test]
fn test_gradle_provider_selective_routing() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let reg = RegistryManager::new(storage);

    let runflare = reg.find_mirror("runflare").unwrap().unwrap();
    let provider = GradleProvider;

    let plan = provider.plan_apply(&runflare).unwrap();
    assert_eq!(plan.provider_id, "gradle");
    assert_eq!(plan.items.len(), 2); // init.d script + properties

    // Verify init.d script content does not route google maven to runflare
    let script_item = &plan.items[0];
    if let pdown::core::models::ChangeItem::FileWrite { new_content, .. } = script_item {
        assert!(new_content.contains("https://mirror-maven.runflare.com/maven2/"));
        assert!(new_content.contains("def GOOGLE_MAVEN_MIRROR = null"));
    } else {
        panic!("Expected FileWrite change item");
    }
}

#[test]
fn test_ios_provider_guarded() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let reg = RegistryManager::new(storage);

    let official = reg.find_mirror("official").unwrap().unwrap();
    let provider = IosProvider;

    assert!(!provider.is_supported());
    let plan = provider.plan_apply(&official);
    assert!(
        plan.is_err(),
        "iOS provider must return an error explaining lack of verified mirrors"
    );
}
