use std::path::Path;
use tokio::process::Command;
use crate::CoreError;

pub struct ScriptRunner;

impl ScriptRunner {
    /// Run pre-install script if it exists
    pub async fn run_pre_install_script(template_dir: &Path, destination_dir: &Path) -> Result<(), CoreError> {
        let script_path = template_dir.join(".mtem").join("pre_install.sh");
        if script_path.exists() {
            Self::run_script(&script_path, destination_dir).await?;
        }
        Ok(())
    }
    
    /// Run post-install script if it exists
    pub async fn run_post_install_script(template_dir: &Path, destination_dir: &Path) -> Result<(), CoreError> {
        let script_path = template_dir.join(".mtem").join("post_install.sh");
        if script_path.exists() {
            Self::run_script(&script_path, destination_dir).await?;
        }
        Ok(())
    }
    
    /// Execute a shell script in the destination directory
    async fn run_script(script_path: &Path, working_dir: &Path) -> Result<(), CoreError> {
        println!("Running script: {}", script_path.display());
        
        let output = Command::new("bash")
            .arg(script_path)
            .current_dir(working_dir)
            .output()
            .await?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::ScriptExecutionFailed(stderr.to_string()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[tokio::test]
    async fn test_run_pre_install_script_exists() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("template");
        let dest_dir = temp_dir.path().join("dest");
        
        fs::create_dir_all(&template_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();
        fs::create_dir_all(template_dir.join(".mtem")).unwrap();
        
        // Create a simple script that creates a file
        fs::write(template_dir.join(".mtem").join("pre_install.sh"), "#!/bin/bash\necho 'pre install' > pre_install_done.txt").unwrap();
        
        ScriptRunner::run_pre_install_script(&template_dir, &dest_dir).await.unwrap();
        
        assert!(dest_dir.join("pre_install_done.txt").exists());
    }
    
    #[tokio::test]
    async fn test_run_pre_install_script_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("template");
        let dest_dir = temp_dir.path().join("dest");
        
        fs::create_dir_all(&template_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();
        
        // Should not error if script doesn't exist
        ScriptRunner::run_pre_install_script(&template_dir, &dest_dir).await.unwrap();
    }
}
