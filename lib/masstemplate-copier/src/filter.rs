use std::path::{Path, PathBuf};

pub struct FileFilter {
    skip_patterns: Vec<String>,
    template_suffix: Option<String>,
}

impl FileFilter {
    pub fn new(skip_patterns: Vec<String>, template_suffix: Option<String>) -> Self {
        Self {
            skip_patterns,
            template_suffix,
        }
    }

    /// Check if file should be skipped
    pub fn should_skip(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.skip_patterns {
            // Simple glob-like matching
            if pattern.contains('*') {
                // Convert glob pattern to simple matching
                if pattern.starts_with("*.") {
                    // Extension pattern like *.log
                    let extension = &pattern[1..]; // .log
                    if path_str.ends_with(extension) {
                        return true;
                    }
                } else if pattern.starts_with('*') {
                    // Suffix pattern like *foo
                    let suffix = &pattern[1..];
                    if path_str.ends_with(suffix) {
                        return true;
                    }
                } else if pattern.ends_with('*') {
                    // Prefix pattern like foo*
                    let prefix = &pattern[..pattern.len() - 1];
                    if path_str.contains(prefix) {
                        return true;
                    }
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Remove template suffix from filename
    pub fn strip_suffix(&self, path: &Path) -> PathBuf {
        if let Some(ref suffix) = self.template_suffix {
            if !suffix.is_empty() {
                if let Some(file_name) = path.file_name() {
                    let name_str = file_name.to_string_lossy();
                    if let Some(stripped) = name_str.strip_suffix(suffix) {
                        return path.with_file_name(stripped);
                    }
                }
            }
        }
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip() {
        let filter = FileFilter::new(
            vec!["*.log".to_string(), ".cache/".to_string()],
            None,
        );

        assert!(filter.should_skip(Path::new("test.log")));
        assert!(filter.should_skip(Path::new(".cache/data")));
        assert!(!filter.should_skip(Path::new("test.txt")));
    }

    #[test]
    fn test_strip_suffix() {
        let filter = FileFilter::new(vec![], Some(".jinja".to_string()));

        let result = filter.strip_suffix(Path::new("template.txt.jinja"));
        assert_eq!(result, PathBuf::from("template.txt"));

        let result = filter.strip_suffix(Path::new("normal.txt"));
        assert_eq!(result, PathBuf::from("normal.txt"));
    }
}
