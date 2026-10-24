use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PiHoleConfig {
    pub enabled: bool,
    pub api_url: String,
    pub password: String,
}

impl Default for PiHoleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_url: "http://pi.hole/api".into(),
            password: String::new(),
        }
    }
}

pub fn validate(config: &crate::config::Config) -> crate::Result<()> {
    if config.pi_hole.enabled {
        validate_api_url(&config.pi_hole.api_url)?;
    }
    Ok(())
}

pub fn validate_api_url(value: &str) -> crate::Result<reqwest::Url> {
    let url = reqwest::Url::parse(value).map_err(|_| "Invalid Pi-hole API URL")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Use an HTTP/HTTPS API URL without credentials, query, or fragment".into());
    }
    Ok(url)
}
