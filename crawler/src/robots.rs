use anyhow::Result;
use reqwest::{get, StatusCode};
use std::collections::HashMap;
use tracing::{error, info};
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct RobotsTxt {
    allow_rules: HashMap<String, Vec<String>>,
    disallow_rules: HashMap<String, Vec<String>>,
}

impl RobotsTxt {
    pub async fn from_url(url: Url) -> Result<Self> {
        info!("gettig robots.txt for {}", url);
        let url = url.join("robots.txt")?;
        let response = get(url).await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(RobotsTxt::default());
        }

        let robots = response.text().await?;

        let mut allow_rules = HashMap::new();
        let mut disallow_rules = HashMap::new();
        let mut current_user_agent = String::from("*");

        for line in robots.lines() {
            if line.starts_with("User-agent:") {
                let user_agent = line.split(':').nth(1).unwrap_or("").trim().to_string();
                current_user_agent.clone_from(&user_agent);

                allow_rules
                    .entry(user_agent.clone())
                    .or_insert_with(Vec::new);
                disallow_rules.entry(user_agent).or_insert_with(Vec::new);
            } else if line.starts_with("Allow:") {
                if let Some(path) = line.split(':').nth(1) {
                    allow_rules
                        .entry(current_user_agent.clone())
                        .or_insert_with(Vec::new)
                        .push(path.trim().to_string());
                }
            } else if line.starts_with("Disallow:") {
                if let Some(path) = line.split(':').nth(1) {
                    disallow_rules
                        .entry(current_user_agent.clone())
                        .or_insert_with(Vec::new)
                        .push(path.trim().to_string());
                }
            }
        }

        Ok(RobotsTxt {
            allow_rules,
            disallow_rules,
        })
    }

    pub fn is_allowed(&self, user_agent: &str, path: &str) -> bool {
        if let Some(allow_paths) = self.allow_rules.get(user_agent) {
            if allow_paths.iter().any(|p| path.starts_with(p)) {
                return true;
            }
        }
        if let Some(disallow_paths) = self.disallow_rules.get(user_agent) {
            if disallow_paths.iter().any(|p| path.starts_with(p)) {
                return false;
            }
        }
        true
    }
}
