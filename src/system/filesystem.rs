use similar::TextDiff;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

/// Generate unified diff between old and new text.
pub fn generate_diff(old: &str, new: &str, label: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    diff.unified_diff()
        .header(&format!("a/{label}"), &format!("b/{label}"))
        .to_string()
}

/// Atomically write bytes to a file using a temporary file and atomic rename.
pub fn atomic_write_bytes(
    path: &Path,
    bytes: &[u8],
    permissions: Option<fs::Permissions>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp = NamedTempFile::new_in(parent_dir)?;
    temp.write_all(bytes)?;
    temp.flush()?;

    if let Some(perms) = permissions {
        let _ = fs::set_permissions(temp.path(), perms);
    }

    temp.persist(path)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}

/// Atomically write content string to a file using temporary file and atomic rename.
pub fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    atomic_write_bytes(path, content.as_bytes(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_diff() {
        let old = "line 1\nline 2\n";
        let new = "line 1\nline 2 modified\nline 3\n";
        let diff = generate_diff(old, new, "test.txt");
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+line 2 modified"));
    }

    #[test]
    fn test_atomic_write() {
        let tmp = tempdir().unwrap();
        let target = tmp.path().join("sub/file.txt");
        atomic_write(&target, "hello world").unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "hello world");
    }
}
