use reqwest::StatusCode;
use serde::*;

pub const POST_URL: &str = "https://cppinsights.io/api/v1/transform";

#[derive(Serialize)]
pub struct InsightsRequest {
    pub code: String,
    #[serde(rename = "insightsOptions")]
    pub insights_options: Vec<String>,
}

#[derive(Deserialize)]
pub struct InsightsResponse {
    #[serde(rename = "returncode")]
    pub return_code: i32,
    pub stderr: String,
    pub stdout: String,
}
pub struct InsightsAPI {
    client: reqwest::Client,
}

impl InsightsAPI {
    pub fn new() -> Self {
        InsightsAPI {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_insights(&self, req: InsightsRequest) -> Option<InsightsResponse> {
        let req_result = self
            .client
            .post(POST_URL)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await;
        if let Err(e) = req_result {
            warn!("Insights request failure: {}", e);
            return None;
        }
        let req = req_result.unwrap();
        if req.status() != StatusCode::OK {
            warn!("Received non-ok status code.");
            return None;
        }

        let resp = req.json::<InsightsResponse>().await;
        if let Err(e) = resp {
            warn!("Unable to get source insights: {}", e);
            None
        } else {
            Some(resp.unwrap())
        }
    }
}
