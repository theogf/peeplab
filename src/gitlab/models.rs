use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub path_with_namespace: String,
    pub web_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MergeRequest {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub author: User,
    pub state: String,
    pub web_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pipeline {
    pub id: u64,
    pub iid: u64,
    pub status: PipelineStatus,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub web_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatus {
    Created,
    WaitingForResource,
    Preparing,
    Pending,
    Running,
    Success,
    Failed,
    Canceled,
    Skipped,
    Manual,
}

impl PipelineStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            PipelineStatus::Success => "✓",
            PipelineStatus::Failed => "✗",
            PipelineStatus::Running => "⟳",
            PipelineStatus::Pending | PipelineStatus::Created | PipelineStatus::Preparing => "○",
            PipelineStatus::Canceled => "⊘",
            PipelineStatus::Skipped => "⊝",
            _ => "•",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Job {
    pub id: u64,
    pub name: String,
    pub status: JobStatus,
    pub stage: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration: Option<f64>,
    pub web_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Created,
    Pending,
    Running,
    Success,
    Failed,
    Canceled,
    Skipped,
    Manual,
}

impl JobStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            JobStatus::Success => "✓",
            JobStatus::Failed => "✗",
            JobStatus::Running => "⟳",
            JobStatus::Pending | JobStatus::Created => "○",
            JobStatus::Canceled => "⊘",
            JobStatus::Skipped => "⊝",
            JobStatus::Manual => "⊙",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_status_deserialization() {
        let json = r#""success""#;
        let status: PipelineStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, PipelineStatus::Success);

        let json = r#""failed""#;
        let status: PipelineStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, PipelineStatus::Failed);

        let json = r#""running""#;
        let status: PipelineStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, PipelineStatus::Running);
    }

    #[test]
    fn test_pipeline_status_symbols() {
        assert_eq!(PipelineStatus::Success.symbol(), "✓");
        assert_eq!(PipelineStatus::Failed.symbol(), "✗");
        assert_eq!(PipelineStatus::Running.symbol(), "⟳");
        assert_eq!(PipelineStatus::Pending.symbol(), "○");
        assert_eq!(PipelineStatus::Canceled.symbol(), "⊘");
        assert_eq!(PipelineStatus::Skipped.symbol(), "⊝");
    }

    #[test]
    fn test_job_status_deserialization() {
        let json = r#""success""#;
        let status: JobStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, JobStatus::Success);

        let json = r#""failed""#;
        let status: JobStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, JobStatus::Failed);

        let json = r#""running""#;
        let status: JobStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, JobStatus::Running);
    }

    #[test]
    fn test_job_status_symbols() {
        assert_eq!(JobStatus::Success.symbol(), "✓");
        assert_eq!(JobStatus::Failed.symbol(), "✗");
        assert_eq!(JobStatus::Running.symbol(), "⟳");
        assert_eq!(JobStatus::Pending.symbol(), "○");
        assert_eq!(JobStatus::Canceled.symbol(), "⊘");
        assert_eq!(JobStatus::Skipped.symbol(), "⊝");
        assert_eq!(JobStatus::Manual.symbol(), "⊙");
    }

    #[test]
    fn test_merge_request_deserialization() {
        let json = r#"{
            "id": 123,
            "iid": 45,
            "title": "Test MR",
            "author": {
                "id": 1,
                "username": "testuser",
                "name": "Test User"
            },
            "state": "opened",
            "web_url": "https://gitlab.com/test/repo/-/merge_requests/45",
            "created_at": "2024-01-01T10:00:00Z",
            "updated_at": "2024-01-01T11:00:00Z"
        }"#;

        let mr: MergeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(mr.id, 123);
        assert_eq!(mr.iid, 45);
        assert_eq!(mr.title, "Test MR");
        assert_eq!(mr.author.username, "testuser");
        assert_eq!(mr.state, "opened");
    }

    #[test]
    fn test_pipeline_deserialization() {
        let json = r#"{
            "id": 456,
            "iid": 78,
            "status": "success",
            "ref": "main",
            "created_at": "2024-01-01T10:00:00Z",
            "updated_at": "2024-01-01T11:00:00Z",
            "web_url": "https://gitlab.com/test/repo/-/pipelines/456"
        }"#;

        let pipeline: Pipeline = serde_json::from_str(json).unwrap();
        assert_eq!(pipeline.id, 456);
        assert_eq!(pipeline.iid, 78);
        assert_eq!(pipeline.status, PipelineStatus::Success);
        assert_eq!(pipeline.ref_name, "main");
    }

    #[test]
    fn test_job_deserialization() {
        let json = r#"{
            "id": 789,
            "name": "test-job",
            "status": "failed",
            "stage": "test",
            "created_at": "2024-01-01T10:00:00Z",
            "started_at": "2024-01-01T10:05:00Z",
            "finished_at": "2024-01-01T10:10:00Z",
            "duration": 300.5,
            "web_url": "https://gitlab.com/test/repo/-/jobs/789"
        }"#;

        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.id, 789);
        assert_eq!(job.name, "test-job");
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.stage, "test");
        assert_eq!(job.duration, Some(300.5));
    }

    #[test]
    fn test_job_with_null_fields() {
        let json = r#"{
            "id": 789,
            "name": "pending-job",
            "status": "pending",
            "stage": "build",
            "created_at": "2024-01-01T10:00:00Z",
            "started_at": null,
            "finished_at": null,
            "duration": null,
            "web_url": "https://gitlab.com/test/repo/-/jobs/789"
        }"#;

        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.started_at.is_none());
        assert!(job.finished_at.is_none());
        assert!(job.duration.is_none());
    }
}
