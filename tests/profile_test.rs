use pdown::core::profile::ProfileManager;
use pdown::infrastructure::storage::StorageManager;
use tempfile::tempdir;

#[test]
fn test_profile_lifecycle() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let mgr = ProfileManager::new(storage);

    // Initial profiles
    let profiles = mgr.list_profiles().unwrap();
    assert!(profiles.contains(&"default".to_string()));
    assert!(profiles.contains(&"iran".to_string()));

    // Active profile starts at "default"
    assert_eq!(mgr.get_active_profile_name(), "default");

    // Create a new profile
    let dev = mgr.create_profile("staging").unwrap();
    assert_eq!(dev.name, "staging");

    // Switch active profile
    mgr.set_active_profile_name("staging").unwrap();
    assert_eq!(mgr.get_active_profile_name(), "staging");
}
