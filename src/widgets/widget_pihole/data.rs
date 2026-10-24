use crate::{
    config::PiHoleConfig,
    http::{Http, HttpError},
};
use reqwest::{
    Method, Url,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub queries: u64,
    pub blocked: u64,
    pub unique_domains: u64,
}

#[derive(Clone)]
pub struct PiHole {
    config: PiHoleConfig,
    sid: Arc<Mutex<Option<String>>>,
}

impl PiHole {
    pub async fn reconfigure(&mut self, config: &PiHoleConfig, http: &Http) -> bool {
        let changed = self.config.api_url.trim_end_matches('/')
            != config.api_url.trim_end_matches('/')
            || self.config.password != config.password
            || (self.config.enabled && !config.enabled);
        if changed {
            self.logout(http).await;
            *self = Self::new(config.clone());
        } else {
            self.config = config.clone();
        }
        changed
    }

    pub fn new(config: PiHoleConfig) -> Self {
        Self {
            config,
            sid: Arc::new(Mutex::new(None)),
        }
    }

    fn url(&self, path: &str) -> Url {
        Url::parse(&format!(
            "{}/{path}",
            self.config.api_url.trim_end_matches('/')
        ))
        .expect("validated API URL")
    }

    async fn authenticate(&self, http: &Http) -> std::result::Result<Option<String>, HttpError> {
        #[derive(Deserialize)]
        struct Auth {
            session: Session,
        }

        #[derive(Deserialize)]
        struct Session {
            valid: bool,
            sid: Option<String>,
        }
        let bytes = http
            .request(
                "pi-hole authentication",
                Method::POST,
                self.url("auth"),
                HeaderMap::new(),
                Some(serde_json::json!({"password": self.config.password})),
            )
            .await?;
        let auth: Auth = serde_json::from_slice(&bytes)
            .map_err(|_| HttpError::from("Invalid authentication response".to_owned()))?;
        if !auth.session.valid {
            return Err(
                "Pi-hole authentication failed; use an application password if 2FA is enabled"
                    .to_owned()
                    .into(),
            );
        }
        Ok(auth.session.sid)
    }

    pub async fn fetch(&self, http: &Http) -> std::result::Result<Stats, HttpError> {
        // Serialize renewal so concurrent consumers cannot create competing sessions.
        let mut session = self.sid.lock().await;
        for attempt in 0..2 {
            let mut headers = HeaderMap::new();
            if let Some(sid) = &*session {
                headers.insert(
                    "X-FTL-SID",
                    HeaderValue::from_str(sid)
                        .map_err(|_| HttpError::from("Invalid session token".to_owned()))?,
                );
            }
            match http
                .request(
                    "pi-hole statistics",
                    Method::GET,
                    self.url("stats/summary"),
                    headers,
                    None,
                )
                .await
            {
                Ok(bytes) => {
                    #[derive(Deserialize)]
                    struct Summary {
                        queries: Queries,
                    }

                    #[derive(Deserialize)]
                    struct Queries {
                        total: u64,
                        blocked: u64,
                        unique_domains: u64,
                    }
                    let summary: Summary = serde_json::from_slice(&bytes)
                        .map_err(|_| HttpError::from("Invalid Pi-hole statistics".to_owned()))?;
                    return Ok(Stats {
                        queries: summary.queries.total,
                        blocked: summary.queries.blocked,
                        unique_domains: summary.queries.unique_domains,
                    });
                }
                Err(error) if error.status == Some(401) && attempt == 0 => {
                    *session = self.authenticate(http).await?;
                }
                Err(error) => return Err(error),
            }
        }
        unreachable!()
    }

    pub async fn logout(&self, http: &Http) {
        let mut session = self.sid.lock().await;
        if let Some(sid) = session.take() {
            let mut headers = HeaderMap::new();
            if let Ok(value) = HeaderValue::from_str(&sid) {
                headers.insert("X-FTL-SID", value);
                let _ = http
                    .request(
                        "pi-hole logout",
                        Method::DELETE,
                        self.url("auth"),
                        headers,
                        None,
                    )
                    .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn reconfiguration_tracks_enable_disable_and_connection_changes() {
        let http = Http::new(false).unwrap();
        let mut config = PiHoleConfig::default();
        let mut client = PiHole::new(config.clone());
        config.enabled = true;
        assert!(!client.reconfigure(&config, &http).await);
        config.api_url.push('/');
        assert!(!client.reconfigure(&config, &http).await);
        config.enabled = false;
        assert!(client.reconfigure(&config, &http).await);
        config.password = "changed".into();
        assert!(client.reconfigure(&config, &http).await);
        assert!(!client.reconfigure(&config, &http).await);
    }
}
