use crate::robots::RobotsTxt;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::time::{Duration, SystemTime};
use url::Url;

const DEFAULT_RPS: f64 = 0.5;
pub const FOXEYE_USER_AGENT: &str = "Foxeye Search";

#[derive(Debug, Clone)]
pub struct Sites {
    pub url: Url,
    pub depth: Option<u32>,
    pub rps: Option<u32>,
    pub timer: Timer,
    pub robots: RobotsTxt,
}

impl Sites {
    pub fn is_allowed(&self, path: &str) -> bool {
        self.robots.is_allowed(FOXEYE_USER_AGENT, path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitesConfig {
    url: String,
    depth: Option<u32>,
    rps: Option<u32>, // request per second
}

impl SitesConfig {
    pub async fn load_config() -> Result<Vec<Sites>> {
        let sites = read_to_string("sites.json")?;
        let val = serde_json::from_str::<serde_json::Value>(&sites)?;
        let val = val
            .get("sites")
            .ok_or(anyhow!("no field \"sites\" found in config"))?;
        let val = serde_json::from_value::<Vec<SitesConfig>>(val.to_owned())?;

        let mut sites = vec![];

        for v in val {
            let url = Url::parse(&v.url)?;

            let t = if v.rps.is_some() {
                1f64 / v.rps.unwrap() as f64
            } else {
                DEFAULT_RPS
            };

            let host = url.host();

            let mut robots = RobotsTxt::default();

            if let Some(h) = host {
                if url.scheme().starts_with("http") {
                    let h = format!("{}://{}", url.scheme(), h);

                    robots = RobotsTxt::from_url(Url::parse(&h)?).await?
                }
            }

            sites.push(Sites {
                url,
                depth: v.depth,
                rps: v.rps,
                timer: Timer::new(Duration::from_secs_f64(t)),
                robots,
            })
        }

        Ok(sites)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Timer {
    start_time: SystemTime,
    time_between: Duration,
}

impl Timer {
    pub fn new(time_between: Duration) -> Self {
        Self {
            start_time: SystemTime::now(),
            time_between,
        }
    }

    pub fn can_send(&mut self) -> bool {
        if SystemTime::now() > (self.start_time + self.time_between) {
            self.start_time = SystemTime::now();
            return true;
        }
        false
    }
}
