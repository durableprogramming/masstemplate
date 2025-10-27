use crate::{CopierError, Result, Task};
use std::path::PathBuf;
use tokio::process::Command;

pub struct TaskRunner {
    dest_dir: PathBuf,
}

impl TaskRunner {
    pub fn new(dest_dir: PathBuf) -> Self {
        Self { dest_dir }
    }

    /// Execute all post-generation tasks
    pub async fn run_tasks(&self, tasks: &[Task]) -> Result<()> {
        for task in tasks {
            self.run_task(task).await?;
        }
        Ok(())
    }

    /// Execute single command task
    async fn run_task(&self, task: &Task) -> Result<()> {
        match task {
            Task::Command(args) => {
                if args.is_empty() {
                    return Err(CopierError::TaskFailed("Empty command".to_string()));
                }

                let (cmd, args) = args.split_first().unwrap();
                let full_command = if args.is_empty() {
                    cmd.to_string()
                } else {
                    format!("{} {}", cmd, args.join(" "))
                };

                println!("  → {}", full_command);

                let output = Command::new(cmd)
                    .args(args)
                    .current_dir(&self.dest_dir)
                    .output()
                    .await
                    .map_err(|e| CopierError::TaskFailed(format!(
                        "Failed to execute command '{}': {}\n\nThis usually means the command is not installed or not in your PATH.",
                        full_command, e
                    )))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(CopierError::TaskFailed(format!(
                        "Command '{}' failed with exit code {}:\n{}",
                        full_command,
                        output.status.code().unwrap_or(-1),
                        stderr
                    )));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.is_empty() {
                    println!("{}", stdout);
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_run_simple_task() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TaskRunner::new(temp_dir.path().to_path_buf());

        let task = Task::Command(vec!["echo".to_string(), "test".to_string()]);
        let result = runner.run_task(&task).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_failing_task() {
        let temp_dir = TempDir::new().unwrap();
        let runner = TaskRunner::new(temp_dir.path().to_path_buf());

        let task = Task::Command(vec!["false".to_string()]);
        let result = runner.run_task(&task).await;

        assert!(result.is_err());
    }
}
