use thiserror::Error;

#[derive(Error, Debug)]
pub enum LabpeepError {
    #[error("GitLab API error: {0}")]
    GitLabApi(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Editor launch failed: {0}")]
    EditorLaunch(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Resource not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, LabpeepError>;
