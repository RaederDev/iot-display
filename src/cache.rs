use std::{
    any::Any,
    collections::HashMap,
    time::{Duration, SystemTime},
};

pub struct CacheEntry {
    pub value: Box<dyn Any>,
    pub ttl: Option<Duration>,
    pub created_at: SystemTime,
}

impl CacheEntry {
    pub fn new(value: Box<dyn Any>, ttl: Option<Duration>) -> Self {
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
}

impl Cache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        if let Some(entry) = self.data.get(key) {
            if entry.is_expired() {
                None
            } else {
                entry.value.downcast_ref::<T>()
            }
        } else {
            None
        }
    }

    pub fn set<T: 'static>(&mut self, key: &str, value: T, ttl: Option<Duration>) {
        self.data
            .insert(key.to_string(), CacheEntry::new(Box::new(value), ttl));
    }
}
