/// Context information passed to hooks during execution
#[derive(Debug, Clone)]
pub struct HookContext {
    pub template_name: String,
    pub template_path: std::path::PathBuf,
    pub destination_path: std::path::PathBuf,
    pub collision_strategy: masstemplate_fileops::CollisionStrategy,
}

impl HookContext {
    pub fn new(
        template_name: String,
        template_path: std::path::PathBuf,
        destination_path: std::path::PathBuf,
        collision_strategy: masstemplate_fileops::CollisionStrategy,
    ) -> Self {
        Self {
            template_name,
            template_path,
            destination_path,
            collision_strategy,
        }
    }

    /// Get the template name
    pub fn template_name(&self) -> &str {
        &self.template_name
    }

    /// Get the template path
    pub fn template_path(&self) -> &std::path::Path {
        &self.template_path
    }

    /// Get the destination path
    pub fn destination_path(&self) -> &std::path::Path {
        &self.destination_path
    }

    /// Get the collision strategy
    pub fn collision_strategy(&self) -> masstemplate_fileops::CollisionStrategy {
        self.collision_strategy
    }

    /// Resolve a working directory path relative to the destination path
    /// If working_dir is None, returns the destination path
    /// If working_dir is relative, joins it with destination_path
    /// If working_dir is absolute, returns it as-is
    pub fn resolve_working_directory(&self, working_dir: Option<&str>) -> std::path::PathBuf {
        match working_dir {
            Some(wd) => {
                let path = std::path::Path::new(wd);
                if path.is_relative() {
                    self.destination_path.join(path)
                } else {
                    path.to_path_buf()
                }
            }
            None => self.destination_path.clone(),
        }
    }

    /// Get the standard environment variables that should be set for hooks
    pub fn get_environment_variables(&self) -> std::collections::HashMap<String, String> {
        let mut env = std::collections::HashMap::new();
        env.insert("MTEM_TEMPLATE_NAME".to_string(), self.template_name.clone());
        env.insert("MTEM_TEMPLATE_PATH".to_string(), self.template_path.display().to_string());
        env.insert("MTEM_DESTINATION_PATH".to_string(), self.destination_path.display().to_string());
        env
    }
}