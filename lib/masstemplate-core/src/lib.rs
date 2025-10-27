pub mod error;
pub mod template_finder;
pub mod file_copier;
pub mod script_runner;
pub mod applicator;

pub use error::*;
pub use template_finder::*;
pub use file_copier::*;
pub use script_runner::*;
pub use applicator::*;

use masstemplate_config::{load_global_config, GlobalConfig};
use masstemplate_fileops::CollisionStrategy;

/// Convenience function to apply a template with a specific collision strategy
pub async fn apply_template_with_strategy(template_name: &str, strategy: CollisionStrategy) -> Result<(), CoreError> {
    let config = load_global_config().await.unwrap_or_else(|_| GlobalConfig::default());
    let applicator = TemplateApplicator::new(config);

    let dest_path = std::env::current_dir()?;
    applicator.apply_template_with_strategy(template_name, &dest_path, strategy).await
}
