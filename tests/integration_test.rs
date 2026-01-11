// Integration tests for labpeep

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_example_config_is_valid() {
        // Verify the example config can be parsed
        let example_config = fs::read_to_string("config.toml.example").unwrap();

        // Replace placeholders with valid values
        let test_config = example_config
            .replace("glpat-xxxxxxxxxxxxxxxxxxxx", "test-token-12345")
            .replace("12345", "99999");

        // This would need the Settings type to be public, so we'll just check it's valid TOML
        let _: toml::Value = toml::from_str(&test_config).unwrap();
    }

    #[test]
    fn test_readme_exists() {
        assert!(std::path::Path::new("README.md").exists());
    }

    #[test]
    fn test_cargo_toml_has_correct_edition() {
        let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();
        assert!(cargo_toml.contains("edition = \"2021\""));
    }

    #[test]
    fn test_required_dependencies() {
        let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();

        // Check for required dependencies
        assert!(cargo_toml.contains("ratatui"));
        assert!(cargo_toml.contains("crossterm"));
        assert!(cargo_toml.contains("tokio"));
        assert!(cargo_toml.contains("reqwest"));
        assert!(cargo_toml.contains("serde"));
        assert!(cargo_toml.contains("toml"));
        assert!(cargo_toml.contains("anyhow"));
        assert!(cargo_toml.contains("thiserror"));
        assert!(cargo_toml.contains("chrono"));
        assert!(cargo_toml.contains("dirs"));
    }

    #[test]
    fn test_dev_dependencies() {
        let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();

        // Check for test dependencies
        assert!(cargo_toml.contains("mockito"));
        assert!(cargo_toml.contains("tempfile"));
        assert!(cargo_toml.contains("tokio-test"));
    }
}
