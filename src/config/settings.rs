use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub gitlab: GitLabConfig,
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub editor: EditorConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitLabConfig {
    pub token: String,
    pub default_project_id: Option<u64>,
    #[serde(default = "default_instance_url")]
    pub instance_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,
    #[serde(default = "default_max_tracked_mrs")]
    pub max_tracked_mrs: usize,
    #[serde(default = "default_focus_current_branch")]
    pub focus_current_branch: bool,
    #[serde(default = "default_auto_refresh_interval_minutes")]
    pub auto_refresh_interval_minutes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    #[serde(default = "default_relative_timestamps")]
    pub relative_timestamps: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EditorConfig {
    pub custom_editor: Option<String>,
}

// Default functions
fn default_instance_url() -> String {
    "https://gitlab.com".to_string()
}

fn default_refresh_interval() -> u64 {
    30
}

fn default_max_tracked_mrs() -> usize {
    5
}

fn default_focus_current_branch() -> bool {
    true
}

fn default_auto_refresh_interval_minutes() -> u64 {
    1
}

fn default_relative_timestamps() -> bool {
    true
}

fn default_theme() -> String {
    "dark".to_string()
}

// Defaults for the configs
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval: default_refresh_interval(),
            max_tracked_mrs: default_max_tracked_mrs(),
            focus_current_branch: default_focus_current_branch(),
            auto_refresh_interval_minutes: default_auto_refresh_interval_minutes(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            relative_timestamps: default_relative_timestamps(),
            theme: default_theme(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            custom_editor: None,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.gitlab.token.is_empty() {
            anyhow::bail!("GitLab token cannot be empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_valid_config() {
        let toml = r#"
            [gitlab]
            token = "test-token"
        "#;

        let settings: Settings = toml::from_str(toml).unwrap();
        assert_eq!(settings.gitlab.token, "test-token");
        assert_eq!(settings.gitlab.instance_url, "https://gitlab.com");
        assert_eq!(settings.app.refresh_interval, 30);
        assert_eq!(settings.app.max_tracked_mrs, 5);
        assert_eq!(settings.app.auto_refresh_interval_minutes, 1);
        assert!(settings.ui.relative_timestamps);
        assert_eq!(settings.ui.theme, "dark");
        assert!(settings.editor.custom_editor.is_none());
    }

    #[test]
    fn test_full_config() {
        let toml = r#"
            [gitlab]
            token = "glpat-test123"
            default_project_id = 42
            instance_url = "https://gitlab.example.com"

            [app]
            refresh_interval = 60
            max_tracked_mrs = 10
            auto_refresh_interval_minutes = 5

            [ui]
            relative_timestamps = false
            theme = "light"

            [editor]
            custom_editor = "nvim"
        "#;

        let settings: Settings = toml::from_str(toml).unwrap();
        assert_eq!(settings.gitlab.token, "glpat-test123");
        assert_eq!(settings.gitlab.default_project_id, Some(42));
        assert_eq!(settings.gitlab.instance_url, "https://gitlab.example.com");
        assert_eq!(settings.app.refresh_interval, 60);
        assert_eq!(settings.app.max_tracked_mrs, 10);
        assert_eq!(settings.app.auto_refresh_interval_minutes, 5);
        assert!(!settings.ui.relative_timestamps);
        assert_eq!(settings.ui.theme, "light");
        assert_eq!(settings.editor.custom_editor, Some("nvim".to_string()));
    }

    #[test]
    fn test_validation_empty_token() {
        let settings = Settings {
            gitlab: GitLabConfig {
                token: String::new(),
                default_project_id: Some(1),
                instance_url: "https://gitlab.com".to_string(),
            },
            app: AppConfig::default(),
            ui: UiConfig::default(),
            editor: EditorConfig::default(),
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validation_valid_token() {
        let settings = Settings {
            gitlab: GitLabConfig {
                token: "valid-token".to_string(),
                default_project_id: Some(1),
                instance_url: "https://gitlab.com".to_string(),
            },
            app: AppConfig::default(),
            ui: UiConfig::default(),
            editor: EditorConfig::default(),
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.refresh_interval, 30);
        assert_eq!(config.max_tracked_mrs, 5);
        assert_eq!(config.auto_refresh_interval_minutes, 1);
    }

    #[test]
    fn test_ui_config_defaults() {
        let config = UiConfig::default();
        assert!(config.relative_timestamps);
        assert_eq!(config.theme, "dark");
    }

    #[test]
    fn test_editor_config_defaults() {
        let config = EditorConfig::default();
        assert!(config.custom_editor.is_none());
    }
}
