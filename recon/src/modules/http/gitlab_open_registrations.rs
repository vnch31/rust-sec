use async_trait::async_trait;
use reqwest::Client;

use crate::{
    modules::{HttpFinding, HttpModule, Module},
    Error,
};

pub struct GitlabOpenRegistrations {}

impl GitlabOpenRegistrations {
    pub fn new() -> Self {
        Self {}
    }
}

impl Module for GitlabOpenRegistrations {
    fn name(&self) -> String {
        String::from("http/gitlab_open_registration")
    }

    fn description(&self) -> String {
        String::from("Check if the GitLab instance is open to registrations")
    }
}

#[async_trait]
impl HttpModule for GitlabOpenRegistrations {
    async fn scan(
        &self,
        http_client: &Client,
        endpoint: &str,
    ) -> Result<Option<HttpFinding>, Error> {
        let url = (&endpoint).to_string();
        let res = http_client.get(&url).send().await?;

        if !res.status().is_success() {
            return Ok(None);
        }

        let body = res.text().await?;
        if body.contains("This is a self-managed instance of GitLab") {
            return Ok(Some(HttpFinding::GitlabOpenRegistration()));
        }

        Ok(None)
    }
}
