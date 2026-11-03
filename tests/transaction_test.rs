use pdown::core::models::ChangePlan;
use pdown::core::provider::Provider;
use pdown::core::registry::RegistryManager;
use pdown::core::transaction::TransactionEngine;
use pdown::infrastructure::storage::StorageManager;
use pdown::providers::{FlutterProvider, GradleProvider};
use pdown::system::backup::BackupManager;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_transaction_dry_run_safety() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let reg = RegistryManager::new(storage.clone());

    let mirror = reg.find_mirror("runflare").unwrap().unwrap();
    let plan = GradleProvider.plan_apply(&mirror).unwrap();

    let backup_mgr = BackupManager::new(&storage);
    let engine = TransactionEngine::new(backup_mgr, true); // dry-run = true

    let report = engine.execute(vec![plan]).unwrap();
    assert!(report.dry_run);
    assert!(!report.diffs.is_empty());
}

#[test]
fn test_transaction_execution_and_rollback() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    storage.ensure_dirs().unwrap();

    let file_a = tmp.path().join("a.txt");
    fs::write(&file_a, "initial a content").unwrap();

    let plan_success = ChangePlan {
        provider_id: "test-success".into(),
        description: "modify a".into(),
        items: vec![pdown::core::models::ChangeItem::FileWrite {
            path: file_a.clone(),
            old_content: Some("initial a content".into()),
            new_content: "modified a content".into(),
            diff: "+modified".into(),
        }],
    };

    // A plan that will fail because its target parent is a file, not a directory
    let blocker = tmp.path().join("blocker_file");
    fs::write(&blocker, "i am a file").unwrap();
    let file_fail = blocker.join("child.txt");

    let plan_fail = ChangePlan {
        provider_id: "test-fail".into(),
        description: "modify child of file".into(),
        items: vec![pdown::core::models::ChangeItem::FileWrite {
            path: file_fail,
            old_content: None,
            new_content: "fail content".into(),
            diff: "+fail".into(),
        }],
    };

    let backup_mgr = BackupManager::new(&storage);
    let engine = TransactionEngine::new(backup_mgr, false);

    let res = engine.execute(vec![plan_success, plan_fail]);
    assert!(res.is_err(), "Transaction must fail and roll back");

    // File A must be restored to "initial a content"
    assert_eq!(fs::read_to_string(&file_a).unwrap(), "initial a content");
}

#[test]
fn test_reject_incompatible_explicit_only_provider() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let reg = RegistryManager::new(storage);

    let pub_azs = reg.find_mirror("pub-azs").unwrap().unwrap();

    // pub-azs does not support flutter
    let flutter_plan = FlutterProvider.plan_apply(&pub_azs);
    assert!(flutter_plan.is_err());
}
