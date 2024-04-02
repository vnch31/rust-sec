use crate::{
    modules::{HttpFinding, HttpModule, Module},
    Error,
};
use async_trait::async_trait;

pub struct EnvFileDisclosure {}

impl EnvFileDisclosure {
    pub fn new() -> Self {
        Self {}
    }
}

impl Module for EnvFileDisclosure {
    fn name(&self) -> String {
        "http_modules/env_file_disclosure".to_string()
    }

    fn description(&self) -> String {
        "Check if the target is exposing the .env file".to_string()
    }
}

#[async_trait]
impl HttpModule for EnvFileDisclosure {
    async fn scan(
        &self,
        http_client: &reqwest::Client,
        endpoint: &str,
    ) -> Result<Option<HttpFinding>, Error> {
        let url = format!("{}/.env", &endpoint);
        let res = http_client.get(&url).send().await?;

        if !res.status().is_success() {
            return Ok(None);
        }

        Ok(Some(HttpFinding::EnvFileDisclosure(url)))
    }
}
