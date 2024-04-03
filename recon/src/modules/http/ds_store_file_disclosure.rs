use crate::{
    modules::{HttpFinding, HttpModule, Module},
    Error,
};
use async_trait::async_trait;

pub struct DsStoreFileDisclosure {}

impl DsStoreFileDisclosure {
    pub fn new() -> Self {
        Self {}
    }

    fn is_ds_store_file(&self, content: &[u8]) -> bool {
        if content.len() < 8 {
            return false;
        }
        let signature = [0x0, 0x0, 0x0, 0x1, 0x42, 0x75, 0x64, 0x31];
        return content[0..8] == signature;
    }
}

impl Module for DsStoreFileDisclosure {
    fn name(&self) -> String {
        "http_modules/ds_store_file_disclosure".to_string()
    }

    fn description(&self) -> String {
        "Check if the target is exposing the .DS_Store file".to_string()
    }
}

#[async_trait]
impl HttpModule for DsStoreFileDisclosure {
    async fn scan(
        &self,
        http_client: &reqwest::Client,
        endpoint: &str,
    ) -> Result<Option<HttpFinding>, Error> {
        let url = format!("{}/.DS_Store", &endpoint);
        let res = http_client.get(&url).send().await?;

        if !res.status().is_success() {
            return Ok(None);
        }

        let body = res.bytes().await?;
        if self.is_ds_store_file(&body.as_ref()) {
            return Ok(Some(HttpFinding::DsStoreFileDisclosure(url)));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn is_ds_store() {
        let module = super::DsStoreFileDisclosure::new();
        let body = "testetstes";
        let body1 = [
            0x00, 0x00, 0x00, 0x01, 0x42, 0x75, 0x64, 0x31, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00,
            0x08, 0x0,
        ];
        assert_eq!(false, module.is_ds_store_file(body.as_bytes()));
        assert_eq!(true, module.is_ds_store_file(&body1));
    }
}
