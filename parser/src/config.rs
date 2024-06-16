use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct Site {
    url: String,
    depth: u32,
    rps: u32,
}

#[derive(Debug)]
pub struct SiteConfig {
    map: HashMap<String, Site>,
}

impl SiteConfig {
    pub fn load_config() -> Result<Self> {
        let sites = read_to_string("sites.json")?;
        let val = serde_json::from_str::<serde_json::Value>(&sites)?;
        let val = val
            .get("sites")
            .ok_or(anyhow!("no field \"sites\" found in config"))?;
        let val = serde_json::from_value::<Vec<Site>>(val.to_owned())?;

        let mut map = HashMap::new();

        for site in val {
            let url = Url::parse(&site.url)?;
            let host = url.host().unwrap().clone().to_string();
            map.insert(host, site);
        }

        Ok(SiteConfig { map })
    }

    pub fn is_allowed(&self, host: String, _current_depth: u32) -> bool {
        // if let Some(site) = self.map.get(&host) {
        //     if current_depth <= site.depth {
        //         return true;
        //     }
        // }
        // false
        self.map.contains_key(&host)
    }
}
