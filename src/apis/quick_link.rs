use reqwest::StatusCode;

pub struct LinkAPI {
    client: reqwest::Client,
    request_base: String,
    redirect_base: String,
}

impl LinkAPI {
    pub fn new(request_base: &str, redirect_base: &str) -> Self {
        LinkAPI {
            client: reqwest::Client::new(),
            request_base: request_base.to_string(),
            redirect_base: redirect_base.to_string(),
        }
    }

    pub async fn get_link(&self, url: String) -> Option<String> {
        let trimmed = url.trim_end().to_string();
        let req_result = self
            .client
            .post(&self.request_base)
            .header("Content-Type", "text/plain")
            .body(trimmed)
            .send()
            .await;
        if let Err(e) = req_result {
            warn!("Quick link request failure: {}", e);
            return None;
        }
        let req = req_result.unwrap();
        if req.status() != StatusCode::OK {
            warn!("Received non-ok status code.");
            return None;
        }

        let body = req.text().await;
        if let Err(e) = body {
            warn!("Unable to get quick link: {}", e);
            None
        } else {
            let url = format!("{}{}", self.redirect_base, body.unwrap());
            info!("Generated url: {}", &url);
            Some(url)
        }
    }
}
