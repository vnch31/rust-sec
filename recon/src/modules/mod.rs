use crate::Error;
use async_trait::async_trait;
use reqwest::Client;

mod http;
mod subdomains;

pub trait Module {
    fn name(&self) -> String;
    fn description(&self) -> String;
}

pub fn subdomain_modules() -> Vec<Box<dyn SubdomainModule>> {
    vec![Box::new(subdomains::Crtsh::new())]
}

pub fn http_modules() -> Vec<Box<dyn HttpModule>> {
    vec![
        Box::new(http::GitlabOpenRegistrations::new()),
        Box::new(http::GitHeadDisclosure::new()),
        Box::new(http::EnvFileDisclosure::new()),
        Box::new(http::DsStoreFileDisclosure::new()),
        Box::new(http::DirectoryListingDisclosure::new()),
    ]
}

/// Http

#[async_trait]
pub trait HttpModule: Module {
    async fn scan(
        &self,
        http_client: &Client,
        endpoint: &str,
    ) -> Result<Option<HttpFinding>, Error>;
}

#[derive(Debug, Clone)]
pub enum HttpFinding {
    GitlabOpenRegistration(String),
    GitHeadDisclosure(String),
    EnvFileDisclosure(String),
    DsStoreFileDisclosure(String),
    DirectoryListingDisclosure(String),
}

/// Subdomain

#[async_trait]
pub trait SubdomainModule: Module {
    async fn enumerate(&self, domain: &str) -> Result<Vec<String>, Error>;
}

#[derive(Debug, Clone)]
pub struct Subdomain {
    pub domain: String,
    pub open_ports: Vec<Port>,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub port: u16,
    pub is_open: bool,
    pub findings: Vec<HttpFinding>,
}
