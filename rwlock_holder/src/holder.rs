use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct Holder<K, V> {
    map: RwLock<HashMap<K, (V, DateTime<Utc>)>>,
    expire_interval: std::time::Duration,
}

impl<K, V> Holder<K, V>
where
    K: std::hash::Hash + Eq,
    V: Clone,
{
    pub fn new(expire_interval: std::time::Duration) -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            expire_interval,
        }
    }

    pub fn get(&self, key: &K, now: DateTime<Utc>) -> Option<V> {
        let map = self.map.read().unwrap_or_else(|e| e.into_inner());
        let (value, timestamp) = map.get(key)?;
        let interval = chrono::Duration::from_std(self.expire_interval).ok()?;
        if now.signed_duration_since(*timestamp) < interval {
            Some(value.clone())
        } else {
            None
        }
    }

    pub fn insert(&self, key: K, value: V, now: DateTime<Utc>) {
        let mut map = self.map.write().unwrap_or_else(|e| e.into_inner());
        if let Some((_, timestamp)) = map.get(&key)
            && let Ok(interval) = chrono::Duration::from_std(self.expire_interval)
            && now.signed_duration_since(*timestamp) < interval
        {
            return;
        }
        map.insert(key, (value, now));
    }
}
