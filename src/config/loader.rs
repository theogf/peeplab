use crate::error::{PeeplabError, Result};
use super::settings::Settings;
use dirs::config_dir;
use std::path::PathBuf;

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = config_dir()
        .ok_or_else(|| PeeplabError::Config("Could not determine config directory".to_string()))?;

    let app_config_dir = config_dir.join("peeplab");
    std::fs::create_dir_all(&app_config_dir)?;

    Ok(app_config_dir.join("config.toml"))
}

pub fn load_config() -> Result<Settings> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err(PeeplabError::Config(format!(
            "Config file not found at {:?}. Please create one with your GitLab token.",
            config_path
        )));
    }

    let content = std::fs::read_to_string(&config_path)?;
    let settings: Settings = toml::from_str(&content)?;

    settings.validate().map_err(|e| PeeplabError::Config(e.to_string()))?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
            [gitlab]
            token = "test-token-123"
            default_project_id = 42
        "#;

        fs::write(&config_path, config_content).unwrap();

        // Mock the config path by reading directly
        let content = fs::read_to_string(&config_path).unwrap();
        let settings: Settings = toml::from_str(&content).unwrap();

        assert!(settings.validate().is_ok());
        assert_eq!(settings.gitlab.token, "test-token-123");
        assert_eq!(settings.gitlab.default_project_id, Some(42));
    }

    #[test]
    fn test_load_config_with_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let invalid_config = "this is not valid toml { ] }";
        fs::write(&config_path, invalid_config).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        let result = toml::from_str::<Settings>(&content);

        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_missing_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let incomplete_config = r#"
            [app]
            refresh_interval = 30
        "#;

        fs::write(&config_path, incomplete_config).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        let result = toml::from_str::<Settings>(&content);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_config_path() {
        let result = get_config_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("peeplab"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }
}
