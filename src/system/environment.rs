use crate::core::errors::Result;
use crate::core::models::ChangeItem;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const MARKER_START_FLUTTER_DART: &str = "# >>> Managed by pdown [Flutter & Dart] >>>";
pub const MARKER_END_FLUTTER_DART: &str = "# <<< Managed by pdown [Flutter & Dart] <<<";

pub struct EnvironmentManager;

impl EnvironmentManager {
    /// Read an environment variable from current process.
    pub fn get_var(key: &str) -> Option<String> {
        env::var(key).ok()
    }

    /// Locate candidate user shell configuration files on Unix systems.
    pub fn get_unix_shell_files() -> Vec<PathBuf> {
        let home = match env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
            Ok(h) => PathBuf::from(h),
            Err(_) => return Vec::new(),
        };

        let candidate_names = if cfg!(target_os = "macos") {
            vec![".zshrc", ".bash_profile", ".bashrc", ".profile"]
        } else {
            vec![".bashrc", ".profile", ".zshrc"]
        };

        let mut existing = Vec::new();
        for name in candidate_names {
            let path = home.join(name);
            if path.exists() {
                existing.push(path);
            }
        }

        if existing.is_empty() {
            if cfg!(target_os = "macos") {
                existing.push(home.join(".zshrc"));
            } else {
                existing.push(home.join(".bashrc"));
            }
        }

        existing
    }

    /// Render shell export block for given environment variable mappings.
    pub fn render_unix_block(
        marker_start: &str,
        marker_end: &str,
        vars: &[(&str, &str)],
    ) -> String {
        let mut lines = Vec::new();
        lines.push(marker_start.to_string());
        for (k, v) in vars {
            lines.push(format!("export {k}=\"{v}\""));
        }
        lines.push(marker_end.to_string());
        lines.join("\n")
    }

    /// Plan updating a shell file with new environment variables.
    pub fn plan_unix_file_update(
        path: &Path,
        marker_start: &str,
        marker_end: &str,
        vars: &[(&str, &str)],
    ) -> Result<ChangeItem> {
        let old_content = if path.exists() {
            Some(fs::read_to_string(path)?)
        } else {
            None
        };

        let payload = Self::render_unix_block(marker_start, marker_end, vars);

        let new_content = match &old_content {
            Some(existing) => {
                if let (Some(s), Some(e)) = (existing.find(marker_start), existing.find(marker_end))
                {
                    let end_pos = e + marker_end.len();
                    format!("{}{}{}", &existing[..s], payload, &existing[end_pos..])
                } else {
                    format!("{existing}\n\n{payload}\n")
                }
            }
            None => format!("{payload}\n"),
        };

        let diff = crate::system::filesystem::generate_diff(
            old_content.as_deref().unwrap_or(""),
            &new_content,
            &path.to_string_lossy(),
        );

        Ok(ChangeItem::FileWrite {
            path: path.to_path_buf(),
            old_content,
            new_content,
            diff,
        })
    }

    /// Plan removing managed environment variables from a shell file.
    pub fn plan_unix_file_revert(
        path: &Path,
        marker_start: &str,
        marker_end: &str,
    ) -> Result<Option<ChangeItem>> {
        if !path.exists() {
            return Ok(None);
        }

        let old_content = fs::read_to_string(path)?;
        let new_content = if let (Some(s), Some(e)) =
            (old_content.find(marker_start), old_content.find(marker_end))
        {
            let end_pos = e + marker_end.len();
            let before = old_content[..s].trim_end();
            let after = old_content[end_pos..].trim_start();
            if before.is_empty() && after.is_empty() {
                String::new()
            } else if before.is_empty() {
                format!("{after}\n")
            } else if after.is_empty() {
                format!("{before}\n")
            } else {
                format!("{before}\n\n{after}\n")
            }
        } else {
            return Ok(None);
        };

        let diff = crate::system::filesystem::generate_diff(
            &old_content,
            &new_content,
            &path.to_string_lossy(),
        );

        Ok(Some(ChangeItem::FileWrite {
            path: path.to_path_buf(),
            old_content: Some(old_content),
            new_content,
            diff,
        }))
    }
}
