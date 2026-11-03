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

/// Atomically write content to a file using temporary file and atomic rename.
pub fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp = NamedTempFile::new_in(parent_dir)?;
    temp.write_all(content.as_bytes())?;
    temp.flush()?;
    temp.persist(path)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
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
