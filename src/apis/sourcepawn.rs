//! Client for the self-hosted sourcepawn-api service (see sourcepawn-api/).
//!
//! The service is enabled by setting SOURCEPAWN_API_URL; a health check at
//! startup fetches the toolchain version used for compiler display names.

use serde::{Deserialize, Serialize};

pub struct SourcePawnService {
    http: reqwest::Client,
    endpoint: String,
    pub version: String,
}

#[derive(Serialize)]
struct CompileRequest<'a> {
    code: &'a str,
    execute: bool,
    asm: bool,
}

#[derive(Deserialize, Default)]
pub struct StageResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub truncated: bool,
}

#[derive(Deserialize)]
pub struct SourcePawnResponse {
    pub compile: StageResult,
    pub run: Option<StageResult>,
    pub asm: Option<StageResult>,
}

#[derive(Deserialize)]
struct HealthResponse {
    version: String,
}

impl SourcePawnService {
    pub async fn new(http: reqwest::Client, endpoint: &str) -> Result<Self, reqwest::Error> {
        let endpoint = endpoint.trim_end_matches('/').to_string();
        let health: HealthResponse = http
            .get(&endpoint)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        // Health reports e.g. "SourcePawn version: 1.13"
        let version = health
            .version
            .trim_start_matches("SourcePawn version:")
            .trim()
            .to_string();

        Ok(SourcePawnService {
            http,
            endpoint,
            version,
        })
    }

    /// Compiler display name, e.g. "spcomp 1.13"
    pub fn compiler_name(&self) -> String {
        format!("spcomp {}", self.version)
    }

    pub async fn compile(
        &self,
        code: &str,
        execute: bool,
        asm: bool,
    ) -> Result<SourcePawnResponse, reqwest::Error> {
        self.http
            .post(format!("{}/compile", self.endpoint))
            .json(&CompileRequest { code, execute, asm })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
