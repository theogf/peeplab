use crate::error::{LabpeepError, Result};
use super::models::{Job, MergeRequest, Note, Pipeline, Project};
use reqwest::{Client, StatusCode, header};

#[derive(Clone)]
pub struct GitLabClient {
    client: Client,
    base_url: String,
}

impl GitLabClient {
    pub fn new(instance_url: &str, token: &str) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "PRIVATE-TOKEN",
            header::HeaderValue::from_str(token)
                .map_err(|e| LabpeepError::Config(format!("Invalid token format: {}", e)))?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            base_url: format!("{}/api/v4", instance_url.trim_end_matches('/')),
        })
    }

    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        match response.status() {
            StatusCode::UNAUTHORIZED => {
                Err(LabpeepError::Authentication(
                    "Invalid GitLab token or insufficient permissions".to_string()
                ))
            }
            StatusCode::NOT_FOUND => {
                Err(LabpeepError::NotFound(
                    "Resource not found".to_string()
                ))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                Err(LabpeepError::Network(
                    "API rate limit exceeded. Please try again later.".to_string()
                ))
            }
            _ => {
                let response = response.error_for_status()?;
                Ok(response.json().await?)
            }
        }
    }

    pub async fn get_project_by_path(&self, project_path: &str) -> Result<Project> {
        // URL encode the project path (namespace/project becomes namespace%2Fproject)
        let encoded_path = project_path.replace('/', "%2F");
        let url = format!("{}/projects/{}", self.base_url, encoded_path);

        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    pub async fn get_merge_requests(&self, project_id: u64) -> Result<Vec<MergeRequest>> {
        let url = format!(
            "{}/projects/{}/merge_requests?state=opened&per_page=20",
            self.base_url, project_id
        );

        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    pub async fn get_merge_requests_by_branch(
        &self,
        project_id: u64,
        source_branch: &str,
    ) -> Result<Vec<MergeRequest>> {
        let url = format!(
            "{}/projects/{}/merge_requests?state=opened&source_branch={}&per_page=20",
            self.base_url, project_id, source_branch
        );

        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    pub async fn get_mr_pipelines(&self, project_id: u64, mr_iid: u64) -> Result<Vec<Pipeline>> {
        let url = format!(
            "{}/projects/{}/merge_requests/{}/pipelines?per_page=10",
            self.base_url, project_id, mr_iid
        );

        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    pub async fn get_pipeline_jobs(&self, project_id: u64, pipeline_id: u64) -> Result<Vec<Job>> {
        let url = format!(
            "{}/projects/{}/pipelines/{}/jobs?per_page=100",
            self.base_url, project_id, pipeline_id
        );

        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    pub async fn get_job_trace(&self, project_id: u64, job_id: u64) -> Result<String> {
        let url = format!(
            "{}/projects/{}/jobs/{}/trace",
            self.base_url, project_id, job_id
        );

        let response = self.client.get(&url).send().await?;

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                Err(LabpeepError::Authentication(
                    "Invalid GitLab token or insufficient permissions".to_string()
                ))
            }
            StatusCode::NOT_FOUND => {
                Err(LabpeepError::NotFound(
                    "Job trace not found".to_string()
                ))
            }
            _ => {
                let response = response.error_for_status()?;
                Ok(response.text().await?)
            }
        }
    }

    pub async fn get_mr_notes(&self, project_id: u64, mr_iid: u64) -> Result<Vec<Note>> {
        let url = format!(
            "{}/projects/{}/merge_requests/{}/notes?per_page=100&sort=desc&order_by=created_at",
            self.base_url, project_id, mr_iid
        );

        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Server, ServerGuard};

    async fn setup_mock_server() -> ServerGuard {
        Server::new_async().await
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = GitLabClient::new("https://gitlab.com", "test-token");
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_get_merge_requests_success() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/merge_requests?state=opened&per_page=20")
            .match_header("PRIVATE-TOKEN", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {
                    "id": 1,
                    "iid": 10,
                    "title": "Test MR",
                    "author": {"id": 1, "username": "user1", "name": "User One"},
                    "state": "opened",
                    "web_url": "https://gitlab.com/test/-/merge_requests/10",
                    "created_at": "2024-01-01T10:00:00Z",
                    "updated_at": "2024-01-01T11:00:00Z"
                }
            ]"#)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_merge_requests(123).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let mrs = result.unwrap();
        assert_eq!(mrs.len(), 1);
        assert_eq!(mrs[0].title, "Test MR");
    }

    #[tokio::test]
    async fn test_get_merge_requests_unauthorized() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/merge_requests?state=opened&per_page=20")
            .with_status(401)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "invalid-token").unwrap();
        let result = client.get_merge_requests(123).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            LabpeepError::Authentication(_) => {}
            _ => panic!("Expected Authentication error"),
        }
    }

    #[tokio::test]
    async fn test_get_mr_pipelines_success() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/merge_requests/10/pipelines?per_page=10")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {
                    "id": 456,
                    "iid": 78,
                    "status": "success",
                    "ref": "main",
                    "created_at": "2024-01-01T10:00:00Z",
                    "updated_at": "2024-01-01T11:00:00Z",
                    "web_url": "https://gitlab.com/test/-/pipelines/456"
                }
            ]"#)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_mr_pipelines(123, 10).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let pipelines = result.unwrap();
        assert_eq!(pipelines.len(), 1);
        assert_eq!(pipelines[0].id, 456);
    }

    #[tokio::test]
    async fn test_get_pipeline_jobs_success() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/pipelines/456/jobs?per_page=100")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {
                    "id": 789,
                    "name": "test-job",
                    "status": "failed",
                    "stage": "test",
                    "created_at": "2024-01-01T10:00:00Z",
                    "started_at": "2024-01-01T10:05:00Z",
                    "finished_at": "2024-01-01T10:10:00Z",
                    "duration": 300.5,
                    "web_url": "https://gitlab.com/test/-/jobs/789"
                }
            ]"#)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_pipeline_jobs(123, 456).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let jobs = result.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "test-job");
    }

    #[tokio::test]
    async fn test_get_job_trace_success() {
        let mut server = setup_mock_server().await;

        let trace_content = "Running tests...\nAll tests passed!";
        let mock = server
            .mock("GET", "/api/v4/projects/123/jobs/789/trace")
            .with_status(200)
            .with_body(trace_content)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_job_trace(123, 789).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), trace_content);
    }

    #[tokio::test]
    async fn test_get_job_trace_not_found() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/jobs/999/trace")
            .with_status(404)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_job_trace(123, 999).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            LabpeepError::NotFound(_) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_rate_limit_error() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/merge_requests?state=opened&per_page=20")
            .with_status(429)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_merge_requests(123).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            LabpeepError::Network(msg) => {
                assert!(msg.contains("rate limit"));
            }
            _ => panic!("Expected Network error for rate limit"),
        }
    }

    #[tokio::test]
    async fn test_get_mr_notes_success() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/merge_requests/10/notes?per_page=100&sort=desc&order_by=created_at")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {
                    "id": 301,
                    "body": "Great work!",
                    "author": {"id": 1, "username": "reviewer", "name": "Reviewer"},
                    "created_at": "2024-01-01T10:00:00Z",
                    "updated_at": "2024-01-01T10:00:00Z",
                    "system": false,
                    "noteable_id": 123,
                    "noteable_type": "MergeRequest",
                    "project_id": 456,
                    "noteable_iid": 10,
                    "resolvable": false,
                    "confidential": false,
                    "internal": false
                }
            ]"#)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_mr_notes(123, 10).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let notes = result.unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "Great work!");
        assert_eq!(notes[0].author.username, "reviewer");
        assert!(!notes[0].system);
    }

    #[tokio::test]
    async fn test_get_mr_notes_empty() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/merge_requests/10/notes?per_page=100&sort=desc&order_by=created_at")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_mr_notes(123, 10).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let notes = result.unwrap();
        assert_eq!(notes.len(), 0);
    }

    #[tokio::test]
    async fn test_get_mr_notes_not_found() {
        let mut server = setup_mock_server().await;

        let mock = server
            .mock("GET", "/api/v4/projects/123/merge_requests/999/notes?per_page=100&sort=desc&order_by=created_at")
            .with_status(404)
            .create_async()
            .await;

        let client = GitLabClient::new(&server.url(), "test-token").unwrap();
        let result = client.get_mr_notes(123, 999).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            LabpeepError::NotFound(_) => {}
            _ => panic!("Expected NotFound error"),
        }
    }
}
