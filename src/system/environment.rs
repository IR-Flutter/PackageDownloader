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

    /// Locate the targeted primary user shell configuration file on Unix systems.
    /// Strictly targets ONLY the user's active/primary shell configuration file to
    /// prevent modifying multiple shell files (.zshrc, .bashrc, .profile) simultaneously.
    pub fn get_primary_unix_shell_file() -> Option<PathBuf> {
        let home = match env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
            Ok(h) => PathBuf::from(h),
            Err(_) => return None,
        };

        let shell_env = env::var("SHELL").unwrap_or_default();
        let target_name = if shell_env.ends_with("/zsh") || shell_env == "zsh" {
            ".zshrc"
        } else if shell_env.ends_with("/bash") || shell_env == "bash" {
            if cfg!(target_os = "macos") {
                if home.join(".bash_profile").exists() {
                    ".bash_profile"
                } else {
                    ".bashrc"
                }
            } else {
                ".bashrc"
            }
        } else if cfg!(target_os = "macos") {
            if home.join(".zshrc").exists() || !home.join(".bash_profile").exists() {
                ".zshrc"
            } else {
                ".bash_profile"
            }
        } else {
            ".bashrc"
        };

        Some(home.join(target_name))
    }

    /// Locate candidate user shell configuration files on Unix systems.
    /// Defaults to the single targeted primary shell configuration file.
    pub fn get_unix_shell_files() -> Vec<PathBuf> {
        Self::get_primary_unix_shell_file().into_iter().collect()
    }

    /// Safely escape a shell value using POSIX single-quote escaping.
    /// Rejects forbidden newline, carriage return, null bytes, and malicious metacharacters.
    pub fn escape_shell_value(val: &str) -> Result<String> {
        if val.contains('\n') || val.contains('\r') || val.contains('\0') {
            return Err(crate::core::errors::PdownError::InvalidConfig(
                "Environment variable value contains forbidden newline or control characters"
                    .into(),
            ));
        }

        // Validate URLs to prevent shell command injection via parameter expansion or subshells
        if (val.starts_with("http://") || val.starts_with("https://"))
            && (val.contains(';')
                || val.contains('&')
                || val.contains('|')
                || val.contains('`')
                || val.contains('$')
                || val.contains('(')
                || val.contains(')'))
        {
            return Err(crate::core::errors::PdownError::InvalidConfig(format!(
                "Suspicious shell metacharacter detected in URL value: '{val}'"
            )));
        }

        // POSIX single-quote escaping: wrap in '...' and replace every ' with '\''.
        // Inside single quotes, all characters are treated literally with zero expansion.
        let escaped = val.replace('\'', r"'\''");
        Ok(format!("'{escaped}'"))
    }

    /// Render shell export block for given environment variable mappings with robust escaping.
    pub fn render_unix_block(
        marker_start: &str,
        marker_end: &str,
        vars: &[(&str, &str)],
    ) -> Result<String> {
        let mut lines = Vec::new();
        lines.push(marker_start.to_string());
        for (k, v) in vars {
            if !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') || k.is_empty() {
                return Err(crate::core::errors::PdownError::InvalidConfig(format!(
                    "Invalid environment variable key: '{k}'"
                )));
            }
            let escaped = Self::escape_shell_value(v)?;
            lines.push(format!("export {k}={escaped}"));
        }
        lines.push(marker_end.to_string());
        Ok(lines.join("\n"))
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

        let payload = Self::render_unix_block(marker_start, marker_end, vars)?;

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

    #[cfg(target_os = "windows")]
    pub fn set_windows_user_var(key: &str, val: &str) -> std::io::Result<()> {
        std::process::Command::new("setx")
            .args([key, val])
            .output()?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn remove_windows_user_var(key: &str) -> std::io::Result<()> {
        let script = format!("[Environment]::SetEnvironmentVariable('{key}', $null, 'User')");
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_shell_value_normal() {
        let escaped = EnvironmentManager::escape_shell_value("https://pub-azs.ir").unwrap();
        assert_eq!(escaped, "'https://pub-azs.ir'");
    }

    #[test]
    fn test_escape_shell_value_with_single_quotes() {
        let escaped = EnvironmentManager::escape_shell_value("custom'value").unwrap();
        assert_eq!(escaped, r"'custom'\''value'");
    }

    #[test]
    fn test_escape_shell_value_rejects_newlines() {
        assert!(EnvironmentManager::escape_shell_value("val\nmalicious").is_err());
        assert!(EnvironmentManager::escape_shell_value("val\rmalicious").is_err());
    }

    #[test]
    fn test_escape_shell_value_rejects_command_injection() {
        assert!(EnvironmentManager::escape_shell_value("https://pub.ir;$(whoami)").is_err());
        assert!(EnvironmentManager::escape_shell_value("https://pub.ir|cat /etc/passwd").is_err());
        assert!(EnvironmentManager::escape_shell_value("https://pub.ir`id`").is_err());
    }

    #[test]
    fn test_render_unix_block_escaping() {
        let vars = [
            ("PUB_HOSTED_URL", "https://pub-azs.ir"),
            ("TEST_KEY", "simple_val"),
        ];
        let block = EnvironmentManager::render_unix_block("# START", "# END", &vars).unwrap();
        assert!(block.contains("export PUB_HOSTED_URL='https://pub-azs.ir'"));
        assert!(block.contains("export TEST_KEY='simple_val'"));
    }
}
