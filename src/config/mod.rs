mod legacy;
mod modern;

use anyhow::Result;

pub use modern::BedrustConfig;

impl BedrustConfig {
    pub fn load_with_migration() -> Result<Self, anyhow::Error> {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("bedrust"))
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        // Check for existing RON config
        let ron_path = config_dir.join("bedrust_config.ron");
        if ron_path.exists() {
            // Load legacy config
            let legacy_config = legacy::load_legacy_config(&ron_path)?;
            
            // Convert to new format
            let new_config = Self::from_legacy(legacy_config);
            
            // Create new TOML config
            let toml_path = config_dir.join("config.toml");
            if !toml_path.exists() {
                println!("📝 Migrating configuration to new format...");
                new_config.save(&toml_path)?;
                println!("✅ Configuration migrated successfully!");
                println!("ℹ️  Old configuration preserved at: {}", ron_path.display());
                println!("ℹ️  New configuration created at: {}", toml_path.display());
            }
            
            Ok(new_config)
        } else {
            // No legacy config exists, proceed with normal loading/creation
            Self::load_or_create()
        }
    }
    fn from_legacy(legacy: legacy::LegacyBedrustConfig) -> Self {
        Self {
            aws: modern::AwsConfig {
                profile: legacy.aws_profile,
                region: None,
            },
            app: modern::AppConfig {
                default_model: legacy.default_model,
                show_banner: legacy.show_banner,
            },
            inference: modern::InferenceConfig {
                temperature: legacy.inference_params.temperature,
                max_tokens: legacy.inference_params.max_tokens,
                top_p: legacy.inference_params.top_p,
            },
            captioning: modern::CaptioningConfig {
                supported_images: legacy.supported_images,
                caption_prompt: legacy.caption_prompt,
            },
        }
    }
}
