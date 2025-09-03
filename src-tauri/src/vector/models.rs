use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "proxemic", "Proxemic")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;

        let models_dir = proj_dirs.data_dir().join("models");
        std::fs::create_dir_all(&models_dir)?;

        Ok(Self { models_dir })
    }

    pub fn get_model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(model_name)
    }

    pub async fn download_model(&self, _model_name: &str) -> Result<PathBuf> {
        // Placeholder for model downloading
        // Will implement in next phase with actual model URLs
        Err(anyhow::anyhow!("Model downloading not yet implemented"))
    }

    pub fn list_available_models(&self) -> Result<Vec<String>> {
        let mut models = Vec::new();

        if self.models_dir.exists() {
            for entry in std::fs::read_dir(&self.models_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".onnx") {
                            models.push(name.to_string());
                        }
                    }
                }
            }
        }

        Ok(models)
    }
}
