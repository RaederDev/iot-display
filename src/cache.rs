use crate::{
    Result,
    integrations::{Data, Provider},
    storage,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub fetched_at: i64,
    pub data: Data,
}

impl Entry {
    pub fn fresh(&self, now: i64, seconds: u64) -> bool {
        self.fetched_at <= now
            && now.saturating_sub(self.fetched_at) < seconds.min(i64::MAX as u64) as i64
            && match &self.data {
                // Daily highs and lows expire at midnight in the weather location.
                Data::Weather(weather) => chrono::DateTime::from_timestamp(now, 0)
                    .is_some_and(|time| weather.current_day(time)),
                _ => true,
            }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct File {
    version: u32,
    entries: BTreeMap<String, Entry>,
}

pub struct Cache {
    path: PathBuf,
    entries: BTreeMap<String, Entry>,
}

impl Cache {
    pub fn load(path: PathBuf) -> Self {
        let file = storage::read_json::<File>(&path)
            .ok()
            .filter(|file| file.version == 1);
        let mut cache = Self {
            path,
            entries: file.map(|file| file.entries).unwrap_or_default(),
        };
        cache.entries.retain(|_, entry| {
            entry.data.valid()
                && entry.fetched_at <= chrono::Utc::now().timestamp()
                && entry.data.provider().persistent_cache()
        });
        cache.bound();
        cache
    }

    pub fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: String, entry: Entry) -> Result<()> {
        if !entry.data.valid() || !entry.data.provider().persistent_cache() {
            return Err("Invalid persistent cache data".into());
        }
        let provider = entry.data.provider();
        // Replace the previous configuration even when both fetch timestamps are identical.
        if provider != Provider::Geocoding {
            self.entries
                .retain(|_, existing| existing.data.provider() != provider);
        }
        self.entries.insert(key, entry);
        self.bound();
        self.save()
    }

    fn bound(&mut self) {
        // Keep the most recent live configurations and a bounded set of location searches.
        for provider in [
            Provider::Weather,
            Provider::Departures,
            Provider::PiHole,
            Provider::Catalogue,
            Provider::Geocoding,
        ] {
            let limit = if provider == Provider::Geocoding {
                100
            } else {
                1
            };
            let mut entries: Vec<_> = self
                .entries
                .iter()
                .filter(|(_, entry)| entry.data.provider() == provider)
                .map(|(key, entry)| (key.clone(), entry.fetched_at))
                .collect();
            entries.sort_by_key(|(_, time)| *time);
            let remove = entries.len().saturating_sub(limit);
            for (key, _) in entries.into_iter().take(remove) {
                self.entries.remove(&key);
            }
        }
    }

    pub fn remove_provider(&mut self, provider: Provider) -> Result<()> {
        self.entries
            .retain(|_, entry| entry.data.provider() != provider);
        self.save()
    }

    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save()
    }

    fn save(&self) -> Result<()> {
        storage::write_json(
            &self.path,
            &File {
                version: 1,
                entries: self.entries.clone(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(time: i64) -> Entry {
        Entry {
            fetched_at: time,
            data: Data::PiHole(crate::integrations::pi_hole::Stats {
                queries: 10,
                blocked: 2,
                unique_domains: 4,
            }),
        }
    }

    #[test]
    fn expires_and_survives_restart_without_extending_lifetime() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("app/cache.json");
        let now = chrono::Utc::now().timestamp();
        let mut cache = Cache::load(path.clone());
        cache.insert("pi-hole:host".into(), stats(now)).unwrap();
        let cache = Cache::load(path);
        let entry = cache.get("pi-hole:host").unwrap();
        assert!(entry.fresh(now + 59, 60));
        assert!(!entry.fresh(now + 60, 60));
        assert!(!entry.fresh(now - 1, 60));
    }

    #[test]
    fn corrupt_cache_is_ignored_and_search_storage_is_bounded() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("cache.json");
        std::fs::write(&path, b"invalid").unwrap();
        let mut cache = Cache::load(path);
        assert!(cache.get("anything").is_none());
        for index in 0..105 {
            cache.entries.insert(
                format!("geo:{index}"),
                Entry {
                    fetched_at: index,
                    data: Data::Geocoding(vec![]),
                },
            );
        }
        cache.bound();
        assert_eq!(cache.entries.len(), 100);
        assert!(!cache.entries.contains_key("geo:0"));
        cache.save().unwrap();
        let cache = Cache::load(cache.path.clone());
        assert_eq!(cache.entries.len(), 100);
    }

    #[test]
    fn parameter_changes_replace_the_previous_identity_even_with_equal_timestamps() {
        let temp = tempfile::tempdir().unwrap();
        let mut cache = Cache::load(temp.path().join("cache.json"));
        let now = chrono::Utc::now().timestamp();
        cache.insert("pi-hole:z".into(), stats(now)).unwrap();
        cache.insert("pi-hole:a".into(), stats(now)).unwrap();
        assert!(cache.get("pi-hole:z").is_none());
        assert!(cache.get("pi-hole:a").is_some());
        assert!(
            cache
                .insert(
                    "invalid".into(),
                    Entry {
                        fetched_at: now,
                        data: Data::Catalogue(vec![])
                    }
                )
                .is_err()
        );
        assert!(cache.get("pi-hole:a").is_some());
    }
}
