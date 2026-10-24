use crate::{
    cache::Entry,
    config::Config,
    integrations::{Provider, key},
};
use std::collections::HashMap;

pub struct DataError {
    pub provider: Provider,
    pub message: String,
    pub retry_at: i64,
}

#[derive(Default)]
pub struct LiveData {
    pub entries: HashMap<String, Entry>,
    pub errors: HashMap<String, DataError>,
}

impl LiveData {
    pub fn entry(&self, provider: Provider, config: &Config) -> Option<&Entry> {
        self.entries.get(&key(provider, config))
    }

    pub fn error(&self, provider: Provider, config: &Config) -> Option<&DataError> {
        self.errors.get(&key(provider, config))
    }

    pub fn record_error(&mut self, key: String, error: DataError) {
        self.errors
            .retain(|_, existing| existing.provider != error.provider);
        self.errors.insert(key, error);
    }

    pub fn insert(&mut self, key: String, entry: Entry) {
        self.entries
            .retain(|_, existing| existing.data.provider() != entry.data.provider());
        self.entries.insert(key, entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.errors.clear();
    }
}
