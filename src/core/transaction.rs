use crate::core::errors::{PdownError, Result};
use crate::core::models::{ChangeItem, ChangePlan};
use crate::system::backup::{BackupManager, BackupRecord};
use crate::system::filesystem::atomic_write;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TransactionReport {
    pub dry_run: bool,
    pub successful_providers: Vec<String>,
    pub modified_files: Vec<PathBuf>,
    pub backups: Vec<BackupRecord>,
    pub diffs: Vec<String>,
}

enum RollbackAction {
    RestoreBackup(BackupRecord),
    DeleteCreatedFile(PathBuf),
}

pub struct TransactionEngine {
    backup_manager: BackupManager,
    dry_run: bool,
}

impl TransactionEngine {
    pub fn new(backup_manager: BackupManager, dry_run: bool) -> Self {
        Self {
            backup_manager,
            dry_run,
        }
    }

    /// Execute a series of change plans with transactional rollback guarantees.
    pub fn execute(&self, plans: Vec<ChangePlan>) -> Result<TransactionReport> {
        let mut report = TransactionReport {
            dry_run: self.dry_run,
            successful_providers: Vec::new(),
            modified_files: Vec::new(),
            backups: Vec::new(),
            diffs: Vec::new(),
        };

        // Collect all diffs for pre-flight display
        for plan in &plans {
            for item in &plan.items {
                match item {
                    ChangeItem::FileWrite { diff, .. } => {
                        if !diff.is_empty() {
                            report.diffs.push(diff.clone());
                        }
                    }
                    ChangeItem::FileDelete { path, .. } => {
                        report.diffs.push(format!(
                            "--- a/{}\n+++ /dev/null\n[File will be deleted]",
                            path.display()
                        ));
                    }
                    _ => {}
                }
            }
        }

        // If in dry-run mode, return report without executing disk changes
        if self.dry_run {
            info!("Dry-run mode active. No changes will be written to disk.");
            for plan in plans {
                report.successful_providers.push(plan.provider_id);
            }
            return Ok(report);
        }

        let mut rollback_stack: Vec<RollbackAction> = Vec::new();

        for plan in plans {
            debug!("Applying plan for provider '{}'", plan.provider_id);
            if let Err(e) = self.apply_plan_items(&plan.items, &mut rollback_stack, &mut report) {
                error!(
                    "Provider '{}' failed during configuration: {e}. Initiating rollback.",
                    plan.provider_id
                );
                self.unwind_rollback(rollback_stack);
                return Err(PdownError::TransactionFailed(format!(
                    "Transaction aborted due to failure in provider '{}': {e}",
                    plan.provider_id
                )));
            }
            report.successful_providers.push(plan.provider_id);
        }

        Ok(report)
    }

    fn apply_plan_items(
        &self,
        items: &[ChangeItem],
        rollback_stack: &mut Vec<RollbackAction>,
        report: &mut TransactionReport,
    ) -> Result<()> {
        for item in items {
            match item {
                ChangeItem::FileWrite {
                    path, new_content, ..
                } => {
                    let existed_before = path.exists();
                    if existed_before {
                        if let Some(backup) = self.backup_manager.backup_file(path)? {
                            report.backups.push(backup.clone());
                            rollback_stack.push(RollbackAction::RestoreBackup(backup));
                        }
                    } else {
                        rollback_stack.push(RollbackAction::DeleteCreatedFile(path.clone()));
                    }

                    atomic_write(path, new_content)?;
                    report.modified_files.push(path.clone());
                }
                ChangeItem::FileDelete { path, .. } => {
                    if path.exists() {
                        if let Some(backup) = self.backup_manager.backup_file(path)? {
                            report.backups.push(backup.clone());
                            rollback_stack.push(RollbackAction::RestoreBackup(backup));
                        }
                        fs::remove_file(path)?;
                        report.modified_files.push(path.clone());
                    }
                }
                ChangeItem::EnvironmentSet { .. } | ChangeItem::EnvironmentUnset { .. } => {
                    // Handled via shell profile FileWrites
                }
            }
        }
        Ok(())
    }

    fn unwind_rollback(&self, mut stack: Vec<RollbackAction>) {
        warn!("Unwinding {} rollback actions...", stack.len());
        while let Some(action) = stack.pop() {
            match action {
                RollbackAction::RestoreBackup(record) => {
                    if let Err(e) = self.backup_manager.restore(&record) {
                        error!(
                            "Failed to restore backup for '{}': {e}",
                            record.original_path.display()
                        );
                    } else {
                        info!("Restored snapshot for '{}'", record.original_path.display());
                    }
                }
                RollbackAction::DeleteCreatedFile(path) => {
                    if path.exists() {
                        if let Err(e) = fs::remove_file(&path) {
                            error!(
                                "Failed to remove created file during rollback '{}': {e}",
                                path.display()
                            );
                        } else {
                            info!(
                                "Removed newly created file '{}' during rollback",
                                path.display()
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_dry_run_does_not_modify_disk() {
        let tmp = tempdir().unwrap();
        let target = tmp.path().join("file.txt");
        let backup_mgr = BackupManager::with_custom_dir(tmp.path().join("backups"));

        let plan = ChangePlan {
            provider_id: "test".into(),
            description: "test plan".into(),
            items: vec![ChangeItem::FileWrite {
                path: target.clone(),
                old_content: None,
                new_content: "dry run content".into(),
                diff: "+dry run content".into(),
            }],
        };

        let engine = TransactionEngine::new(backup_mgr, true);
        let report = engine.execute(vec![plan]).unwrap();

        assert!(report.dry_run);
        assert!(
            !target.exists(),
            "Target file must not be created in dry-run mode"
        );
    }

    #[test]
    fn test_transaction_rollback_on_failure() {
        let tmp = tempdir().unwrap();
        let target_a = tmp.path().join("a.txt");
        let target_invalid_dir = tmp.path().join("non_existent_dir_as_file");
        // Create a file where a directory would be needed to provoke an I/O error
        fs::write(&target_invalid_dir, "blocker").unwrap();
        let target_b = target_invalid_dir.join("b.txt");

        fs::write(&target_a, "original a").unwrap();

        let backup_mgr = BackupManager::with_custom_dir(tmp.path().join("backups"));

        let plan_a = ChangePlan {
            provider_id: "a".into(),
            description: "change a".into(),
            items: vec![ChangeItem::FileWrite {
                path: target_a.clone(),
                old_content: Some("original a".into()),
                new_content: "mutated a".into(),
                diff: "diff a".into(),
            }],
        };

        let plan_b = ChangePlan {
            provider_id: "b".into(),
            description: "change b".into(),
            items: vec![ChangeItem::FileWrite {
                path: target_b,
                old_content: None,
                new_content: "new b".into(),
                diff: "diff b".into(),
            }],
        };

        let engine = TransactionEngine::new(backup_mgr, false);
        let result = engine.execute(vec![plan_a, plan_b]);

        assert!(result.is_err(), "Transaction must fail on invalid write");
        // Verify target_a was rolled back to original content
        assert_eq!(fs::read_to_string(&target_a).unwrap(), "original a");
    }
}
