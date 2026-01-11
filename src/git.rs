use crate::error::{LabpeepError, Result};
use git2::Repository;
use url::Url;

#[derive(Debug, Clone)]
pub struct GitLabProject {
    pub namespace: String,
    pub name: String,
    pub host: String,
}

impl GitLabProject {
    pub fn path(&self) -> String {
        format!("{}/{}", self.namespace, self.name)
    }

    pub fn url_encoded_path(&self) -> String {
        self.path().replace('/', "%2F")
    }
}

/// Detect GitLab project from git remote URL
pub fn detect_project_from_git() -> Result<GitLabProject> {
    let repo = Repository::open(".")
        .map_err(|e| LabpeepError::Config(format!("Not a git repository: {}", e)))?;

    let remote = repo
        .find_remote("origin")
        .map_err(|e| LabpeepError::Config(format!("No 'origin' remote found: {}", e)))?;

    let url = remote
        .url()
        .ok_or_else(|| LabpeepError::Config("Remote URL is not valid UTF-8".to_string()))?;

    parse_gitlab_url(url)
}

/// Get the current git branch name
pub fn get_current_branch() -> Result<String> {
    let repo = Repository::open(".")
        .map_err(|e| LabpeepError::Config(format!("Not a git repository: {}", e)))?;

    let head = repo.head()
        .map_err(|e| LabpeepError::Config(format!("Failed to get HEAD: {}", e)))?;

    let branch_name = head
        .shorthand()
        .ok_or_else(|| LabpeepError::Config("Could not determine branch name".to_string()))?
        .to_string();

    Ok(branch_name)
}

fn parse_gitlab_url(git_url: &str) -> Result<GitLabProject> {
    // Handle SSH URLs like git@gitlab.com:namespace/project.git
    if git_url.starts_with("git@") {
        return parse_ssh_url(git_url);
    }

    // Handle HTTPS URLs like https://gitlab.com/namespace/project.git
    if git_url.starts_with("http://") || git_url.starts_with("https://") {
        return parse_https_url(git_url);
    }

    Err(LabpeepError::Config(format!(
        "Unsupported git remote URL format: {}",
        git_url
    )))
}

fn parse_ssh_url(url: &str) -> Result<GitLabProject> {
    // Format: git@gitlab.com:namespace/project.git
    let without_prefix = url
        .strip_prefix("git@")
        .ok_or_else(|| LabpeepError::Config("Invalid SSH URL format".to_string()))?;

    let parts: Vec<&str> = without_prefix.split(':').collect();
    if parts.len() != 2 {
        return Err(LabpeepError::Config("Invalid SSH URL format".to_string()));
    }

    let host = parts[0].to_string();
    let path = parts[1].trim_end_matches(".git");

    let path_parts: Vec<&str> = path.split('/').collect();
    if path_parts.len() < 2 {
        return Err(LabpeepError::Config(
            "Could not parse namespace/project from URL".to_string(),
        ));
    }

    Ok(GitLabProject {
        host,
        namespace: path_parts[0].to_string(),
        name: path_parts[1].to_string(),
    })
}

fn parse_https_url(url_str: &str) -> Result<GitLabProject> {
    let url = Url::parse(url_str)
        .map_err(|e| LabpeepError::Config(format!("Invalid HTTPS URL: {}", e)))?;

    let host = url
        .host_str()
        .ok_or_else(|| LabpeepError::Config("No host in URL".to_string()))?
        .to_string();

    let path = url.path().trim_start_matches('/').trim_end_matches(".git");

    let path_parts: Vec<&str> = path.split('/').collect();
    if path_parts.len() < 2 {
        return Err(LabpeepError::Config(
            "Could not parse namespace/project from URL".to_string(),
        ));
    }

    Ok(GitLabProject {
        host,
        namespace: path_parts[0].to_string(),
        name: path_parts[1].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let url = "git@gitlab.com:myorg/myproject.git";
        let project = parse_ssh_url(url).unwrap();

        assert_eq!(project.host, "gitlab.com");
        assert_eq!(project.namespace, "myorg");
        assert_eq!(project.name, "myproject");
        assert_eq!(project.path(), "myorg/myproject");
    }

    #[test]
    fn test_parse_ssh_url_without_git_suffix() {
        let url = "git@gitlab.example.com:namespace/repo";
        let project = parse_ssh_url(url).unwrap();

        assert_eq!(project.host, "gitlab.example.com");
        assert_eq!(project.namespace, "namespace");
        assert_eq!(project.name, "repo");
    }

    #[test]
    fn test_parse_https_url() {
        let url = "https://gitlab.com/myorg/myproject.git";
        let project = parse_https_url(url).unwrap();

        assert_eq!(project.host, "gitlab.com");
        assert_eq!(project.namespace, "myorg");
        assert_eq!(project.name, "myproject");
    }

    #[test]
    fn test_parse_http_url() {
        let url = "http://gitlab.example.com/namespace/repo.git";
        let project = parse_https_url(url).unwrap();

        assert_eq!(project.host, "gitlab.example.com");
        assert_eq!(project.namespace, "namespace");
        assert_eq!(project.name, "repo");
    }

    #[test]
    fn test_url_encoded_path() {
        let project = GitLabProject {
            host: "gitlab.com".to_string(),
            namespace: "my-org".to_string(),
            name: "my-project".to_string(),
        };

        assert_eq!(project.url_encoded_path(), "my-org%2Fmy-project");
    }

    #[test]
    fn test_parse_invalid_ssh_url() {
        let url = "git@gitlab.com/invalid";
        assert!(parse_ssh_url(url).is_err());
    }

    #[test]
    fn test_parse_invalid_https_url() {
        let url = "https://gitlab.com/singlepart";
        assert!(parse_https_url(url).is_err());
    }

    #[test]
    fn test_get_current_branch() {
        // This test only works if we're in a git repo
        // We'll make it optional
        if let Ok(branch) = get_current_branch() {
            assert!(!branch.is_empty());
            // Branch name should not contain slashes at the start
            assert!(!branch.starts_with('/'));
        }
    }
}
