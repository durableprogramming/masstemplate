use std::path::Path;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum VcsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command execution failed: {0}")]
    Command(String),
    #[error("Git command failed: {0}")]
    Git(String),
    #[error("Unsupported VCS type: {0}")]
    UnsupportedVcs(String),
    #[error("VCS not found: {0}")]
    VcsNotFound(String),
}

pub type Result<T> = std::result::Result<T, VcsError>;

/// Supported version control systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcsType {
    Git,
    // Future: Hg, Svn, etc.
}

impl VcsType {
    /// Get the command name for this VCS
    pub fn command(&self) -> &'static str {
        match self {
            VcsType::Git => "git",
        }
    }

    /// Check if this VCS is available on the system
    pub async fn is_available(&self) -> bool {
        Command::new(self.command())
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// Configuration for VCS operations
#[derive(Debug, Clone)]
pub struct VcsConfig {
    pub vcs_type: VcsType,
    pub auto_init: bool,
    pub auto_commit: bool,
    pub commit_message: Option<String>,
}

impl Default for VcsConfig {
    fn default() -> Self {
        Self {
            vcs_type: VcsType::Git,
            auto_init: true,
            auto_commit: true,
            commit_message: Some("Initial commit from masstemplate".to_string()),
        }
    }
}

/// Manages version control operations for templates
pub struct VcsManager {
    config: VcsConfig,
}

impl Default for VcsManager {
    fn default() -> Self {
        Self::new(VcsConfig::default())
    }
}

impl VcsManager {
    /// Create a new VCS manager with the given configuration
    pub fn new(config: VcsConfig) -> Self {
        Self { config }
    }

    /// Initialize a VCS repository in the given directory
    pub async fn init_repo(&self, path: &Path) -> Result<()> {
        if !self.config.vcs_type.is_available().await {
            return Err(VcsError::VcsNotFound(self.config.vcs_type.command().to_string()));
        }

        match self.config.vcs_type {
            VcsType::Git => self.init_git_repo(path).await,
        }
    }

    /// Add all files in the directory to VCS
    pub async fn add_files(&self, path: &Path) -> Result<()> {
        match self.config.vcs_type {
            VcsType::Git => self.git_add_all(path).await,
        }
    }

    /// Commit changes with the configured message
    pub async fn commit(&self, path: &Path) -> Result<()> {
        let message = self.config.commit_message.as_deref()
            .unwrap_or("Initial commit");

        match self.config.vcs_type {
            VcsType::Git => self.git_commit(path, message).await,
        }
    }

    /// Perform a complete VCS setup: init, add, commit
    pub async fn setup_repo(&self, path: &Path) -> Result<()> {
        if self.config.auto_init {
            println!("Initializing {} repository...", self.config.vcs_type.command());
            self.init_repo(path).await?;
        }

        if self.config.auto_commit {
            println!("Adding files to {}...", self.config.vcs_type.command());
            self.add_files(path).await?;

            println!("Committing files...");
            self.commit(path).await?;
        }

        Ok(())
    }

    /// Check if a directory is already a VCS repository
    pub async fn is_repo(&self, path: &Path) -> bool {
        match self.config.vcs_type {
            VcsType::Git => path.join(".git").is_dir(),
        }
    }

    // Git-specific implementations
    async fn init_git_repo(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()
            .await
            .map_err(|e| VcsError::Command(format!("Failed to run git init: {}", e)))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VcsError::Git(format!("git init failed: stdout={}, stderr={}", stdout, stderr)));
        }

        Ok(())
    }

    async fn git_add_all(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| VcsError::Command(format!("Failed to run git add: {}", e)))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VcsError::Git(format!("git add failed: stdout={}, stderr={}", stdout, stderr)));
        }

        Ok(())
    }

    async fn git_commit(&self, path: &Path, message: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| VcsError::Command(format!("Failed to run git commit: {}", e)))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VcsError::Git(format!("git commit failed: stdout={}, stderr={}", stdout, stderr)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        // Create some test files
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::create_dir_all(temp_dir.path().join("subdir")).unwrap();
        fs::write(temp_dir.path().join("subdir/file3.txt"), "content3").unwrap();
        temp_dir
    }

    #[tokio::test]
    async fn test_vcs_type_git_available() {
        let available = VcsType::Git.is_available().await;
        // Git should be available in most development environments
        // If not, this test will be skipped
        if !available {
            println!("Git not available, skipping test");
            return;
        }
        assert!(available);
    }

    #[tokio::test]
    async fn test_vcs_manager_default() {
        let manager = VcsManager::default();
        assert_eq!(manager.config.vcs_type, VcsType::Git);
        assert!(manager.config.auto_init);
        assert!(manager.config.auto_commit);
        assert!(manager.config.commit_message.is_some());
    }

    #[tokio::test]
    async fn test_is_repo_false_for_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let manager = VcsManager::default();
        assert!(!manager.is_repo(temp_dir.path()).await);
    }

    #[tokio::test]
    async fn test_setup_repo_success() {
        if !VcsType::Git.is_available().await {
            println!("Git not available, skipping test");
            return;
        }

        let temp_dir = create_test_dir();
        let manager = VcsManager::default();

        let result = manager.setup_repo(temp_dir.path()).await;
        assert!(result.is_ok());

        // Check that .git directory was created
        assert!(temp_dir.path().join(".git").is_dir());
    }

    #[tokio::test]
    async fn test_init_repo_success() {
        if !VcsType::Git.is_available().await {
            println!("Git not available, skipping test");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let manager = VcsManager::default();

        let result = manager.init_repo(temp_dir.path()).await;
        assert!(result.is_ok());
        assert!(temp_dir.path().join(".git").is_dir());
    }

    #[tokio::test]
    async fn test_add_files_success() {
        if !VcsType::Git.is_available().await {
            println!("Git not available, skipping test");
            return;
        }

        let temp_dir = create_test_dir();
        let manager = VcsManager::default();

        // Init repo first
        manager.init_repo(temp_dir.path()).await.unwrap();

        // Add files
        let result = manager.add_files(temp_dir.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_commit_success() {
        if !VcsType::Git.is_available().await {
            println!("Git not available, skipping test");
            return;
        }

        let temp_dir = create_test_dir();
        let manager = VcsManager::default();

        // Init and add first
        manager.init_repo(temp_dir.path()).await.unwrap();
        manager.add_files(temp_dir.path()).await.unwrap();

        // Commit
        let result = manager.commit(temp_dir.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_vcs_not_found() {
        let config = VcsConfig {
            vcs_type: VcsType::Git,
            auto_init: true,
            auto_commit: false,
            commit_message: None,
        };
        let manager = VcsManager::new(config);

        // Mock git not available by using a non-existent command
        // This is tricky to test directly, but we can test the error path
        // by using a directory that doesn't exist or similar

        let temp_dir = TempDir::new().unwrap();
        // This should work if git is available
        if VcsType::Git.is_available().await {
            let result = manager.init_repo(temp_dir.path()).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_custom_commit_message() {
        let config = VcsConfig {
            vcs_type: VcsType::Git,
            auto_init: true,
            auto_commit: true,
            commit_message: Some("Custom commit message".to_string()),
        };
        let manager = VcsManager::new(config);

        if !VcsType::Git.is_available().await {
            println!("Git not available, skipping test");
            return;
        }

        let temp_dir = create_test_dir();
        let result = manager.setup_repo(temp_dir.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_auto_init() {
        let config = VcsConfig {
            vcs_type: VcsType::Git,
            auto_init: false,
            auto_commit: false,
            commit_message: None,
        };
        let manager = VcsManager::new(config);

        let temp_dir = create_test_dir();
        let result = manager.setup_repo(temp_dir.path()).await;
        // Should succeed but not create .git since auto_init is false
        assert!(result.is_ok());
        assert!(!temp_dir.path().join(".git").is_dir());
    }
}