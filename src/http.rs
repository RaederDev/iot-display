use crate::Result;
use reqwest::{
    Client, Method, Url,
    header::{HeaderMap, LOCATION, RETRY_AFTER},
};
use std::{
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[derive(Clone)]
pub struct Http {
    client: Client,
    pub debug: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct HttpError {
    pub message: String,
    pub status: Option<u16>,
    pub retry_after: Option<u64>,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<String> for HttpError {
    fn from(message: String) -> Self {
        Self {
            message,
            status: None,
            retry_after: None,
        }
    }
}

impl Http {
    pub fn new(debug: bool) -> Result<Self> {
        Self::with_timeout(debug, Duration::from_secs(10))
    }

    pub fn with_timeout(debug: bool, timeout: Duration) -> Result<Self> {
        // Follow redirects ourselves so every HTTP attempt passes through request logging.
        let client = Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout.min(Duration::from_secs(5)))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("iot-display/0.2")
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            client,
            debug: Arc::new(AtomicBool::new(debug)),
        })
    }

    pub async fn request(
        &self,
        target: &str,
        mut method: Method,
        mut url: Url,
        mut headers: HeaderMap,
        mut body: Option<serde_json::Value>,
    ) -> std::result::Result<Vec<u8>, HttpError> {
        for redirects in 0..=10 {
            if self.debug.load(Ordering::Relaxed) {
                let mut out = std::io::stdout().lock();
                let _ = writeln!(out, "{}", request_log(target, &method, &url));
                let _ = out.flush();
            }
            let mut request = self
                .client
                .request(method.clone(), url.clone())
                .headers(headers.clone());
            if let Some(value) = &body {
                request = request.json(value);
            }
            let response = request.send().await.map_err(|error| {
                HttpError::from(if error.is_timeout() {
                    "Request timed out".to_owned()
                } else if error.is_connect() {
                    "Connection failed (check host, network, and TLS certificate)".to_owned()
                } else {
                    "Network request failed".to_owned()
                })
            })?;
            let status = response.status();
            if matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308) {
                if redirects == 10 {
                    return Err("Too many redirects".to_owned().into());
                }
                let location = response
                    .headers()
                    .get(LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| {
                        HttpError::from("Redirect has no valid destination".to_owned())
                    })?;
                let next = url
                    .join(location)
                    .map_err(|_| HttpError::from("Invalid redirect destination".to_owned()))?;
                if !matches!(next.scheme(), "http" | "https")
                    || (url.scheme() == "https" && next.scheme() != "https")
                    || !next.username().is_empty()
                    || next.password().is_some()
                {
                    return Err("Unsafe redirect rejected".to_owned().into());
                }
                if url.origin() != next.origin() {
                    if body.is_some() || headers.contains_key("X-FTL-SID") {
                        return Err("Authenticated cross-origin redirect rejected"
                            .to_owned()
                            .into());
                    }
                    headers.clear();
                }
                if status.as_u16() == 303
                    || (matches!(status.as_u16(), 301 | 302) && method == Method::POST)
                {
                    method = Method::GET;
                    body = None;
                }
                url = next;
                continue;
            }
            if !status.is_success() {
                let retry_after = response
                    .headers()
                    .get(RETRY_AFTER)
                    .and_then(|value| value.to_str().ok())
                    .and_then(parse_retry_after);
                return Err(HttpError {
                    message: format!("HTTP {}", status.as_u16()),
                    status: Some(status.as_u16()),
                    retry_after,
                });
            }
            // Bound external responses before parsing; the stop catalogue is well below this limit.
            let mut response = response;
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| HttpError::from("Cannot read response".to_owned()))?
            {
                if bytes.len() + chunk.len() > 8 * 1024 * 1024 {
                    return Err("Response exceeds 8 MiB".to_owned().into());
                }
                bytes.extend_from_slice(&chunk);
            }
            return Ok(bytes);
        }
        unreachable!()
    }

    pub async fn get(&self, target: &str, url: Url) -> std::result::Result<Vec<u8>, HttpError> {
        self.request(target, Method::GET, url, HeaderMap::new(), None)
            .await
    }
}

fn parse_retry_after(value: &str) -> Option<u64> {
    value.parse().ok().or_else(|| {
        chrono::DateTime::parse_from_rfc2822(value)
            .ok()
            .map(|date| (date.timestamp() - chrono::Utc::now().timestamp()).max(0) as u64)
    })
}

pub fn redacted_url(url: &Url) -> String {
    let mut safe = url.clone();
    let _ = safe.set_username("");
    let _ = safe.set_password(None);
    safe.set_fragment(None);
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(name, value)| {
            let lower = name.to_lowercase();
            let secret = [
                "password",
                "token",
                "sid",
                "auth",
                "key",
                "secret",
                "csrf",
                "credential",
            ]
            .iter()
            .any(|part| lower.contains(part));
            (
                name.into_owned(),
                if secret {
                    "REDACTED".into()
                } else {
                    value.into_owned()
                },
            )
        })
        .collect();
    if !pairs.is_empty() {
        safe.query_pairs_mut().clear().extend_pairs(pairs);
    }
    safe.to_string()
}

pub fn request_log(target: &str, method: &Method, url: &Url) -> String {
    format!(
        "[{}] {target} {method} {}",
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        redacted_url(url)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_secrets_but_preserves_request_parameters() {
        let url = Url::parse(
            "https://user:password@example.org/api?stopId=399&sid=private&apiKey=secret#token",
        )
        .unwrap();
        let log = request_log("departures", &Method::GET, &url);
        assert!(log.contains("stopId=399"));
        assert!(log.contains("GET https://example.org/api"));
        for secret in ["user", "password", "private", "secret", "#token"] {
            assert!(!log.contains(secret));
        }
    }
}
