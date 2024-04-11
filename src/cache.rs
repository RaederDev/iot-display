use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use anyhow::{bail, Error};
use dirs::cache_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub value: String,
    pub ttl: Option<Duration>,
    pub created_at: SystemTime,
}

impl CacheEntry {
    pub fn new(value: String, ttl: Option<Duration>) -> Self {
        Self {
            value,
            ttl,
            created_at: SystemTime::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed().unwrap_or_default() >= ttl
        } else {
            false
        }
    }
}

pub struct Cache {
    data: HashMap<String, CacheEntry>,
    persist: bool,
    cache_name: String,
}

impl Cache {
    pub fn new(persist: bool, cache_name: String) -> Self {
        Self {
            data: HashMap::new(),
            persist,
            cache_name,
        }
    }

    fn get_cache_path(&self) -> Option<PathBuf> {
        cache_dir().map(|p| p.join("iot-display").join(&self.cache_name))
    }

    pub fn load_cache_from_disk(&mut self) -> Result<(), Error> {
        if let Some(path) = self.get_cache_path() {
            let data = match fs::read_to_string(path) {
                Ok(data) => data,
                Err(_) => String::new(),
            };
            self.data = serde_json::from_str(&data)?;
            return Ok(());
        }
        bail!("Could not get cache path")
    }

    pub fn save_cache_to_disk(&self) -> Result<(), Error> {
        if let Some(path) = self.get_cache_path() {
            println!("Saving cache to disk: {:?}", path);
            let data = serde_json::to_string(&self.data)?;
            fs::write(path, data)?;
            return Ok(());
        }
        bail!("Could not get cache path")
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(entry) = self.data.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            } else {
                self.remove(key);
            }
        }
        None
    }

    pub fn set(&mut self, key: &str, value: String, ttl: Option<Duration>) {
        self.data
            .insert(key.to_string(), CacheEntry::new(value, ttl));
    }

    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }
}
