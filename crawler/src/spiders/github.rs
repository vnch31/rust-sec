use crate::{error::Error, spiders::Spider};
use async_trait::async_trait;
use regex::Regex;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubItem {
    login: String,
    id: u64,
    node_id: String,
    html_url: String,
    avatar_url: String,
}

pub struct GithubSpider {
    http_client: Client,
    page_regex: Regex,
    expected_number_of_results: usize,
}

impl GithubSpider {
    pub fn new() -> Self {
        let http_timeout = Duration::from_secs(10);
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        let http_client = Client::builder()
            .timeout(http_timeout)
            .default_headers(headers)
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Safari/537.36")
            .build()
            .expect("spiders/github: Building HTTP client");

        let page_regex =
            Regex::new(".*page=([0-9]*).*").expect("spiders/github: Compiling page regex");

        GithubSpider {
            http_client,
            page_regex,
            expected_number_of_results: 100,
        }
    }
}

#[async_trait]
impl Spider for GithubSpider {
    type Item = GithubItem;

    fn name(&self) -> String {
        "spider/github: Github Spider".to_string()
    }

    fn start_urls(&self) -> Vec<String> {
        vec!["https://api.github.com/orgs/google/public_members?per_page=100&page=1".to_string()]
    }

    async fn scrape(&self, url: String) -> Result<(Vec<GithubItem>, Vec<String>), Error> {
        let items: Vec<GithubItem> = self.http_client.get(&url).send().await?.json().await?;

        let next_pages_links = if items.len() == self.expected_number_of_results {
            let captures = self.page_regex.captures(&url).unwrap();
            let old_page_number = captures.get(1).unwrap().as_str().to_string();
            let mut new_page_number = old_page_number
                .parse::<usize>()
                .map_err(|_| Error::Internal("spiders/github: Parsing page number".to_string()))?;
            new_page_number += 1;
            let next_url = url.replace(
                format!("page={}", old_page_number).as_str(),
                format!("page={}", new_page_number).as_str(),
            );
            vec![next_url]
        } else {
            Vec::new()
        };
        Ok((items, next_pages_links))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        println!("{}, {}, {}", item.login, item.html_url, item.avatar_url);

        Ok(())
    }
}
