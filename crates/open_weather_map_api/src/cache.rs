use std::{collections::HashMap, fmt::Debug, hash::Hash, num::NonZero};

use crate::{Latitude, Longitude};

#[derive(Debug)]
pub struct Cache<F>
where
    F: Debug + Clone,
{
    cache: HashMap<CacheIndex, CacheEntry<F>>,
    ttl: chrono::TimeDelta,
    soft_cache_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CacheIndex(pub i16, pub i16);

#[derive(Debug, PartialEq, PartialOrd)]
struct CacheEntry<F>
where
    F: Debug + Clone,
{
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub forecast: F,
}

impl<F> Cache<F>
where
    F: Debug + Clone,
{
    pub fn new(ttl: chrono::TimeDelta, soft_cache_limit: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(soft_cache_limit),
            ttl,
            soft_cache_limit,
        }
    }

    /// Lookup a `Forecast` in `Cache`.
    ///
    /// This will return `None` if either there is no cached Forecast or if the forecast is expired.
    pub fn lookup(&mut self, lat: Latitude, lon: Longitude) -> Option<F> {
        let cache_index = CacheIndex::new(lat, lon);
        if let Some(cache_entry) = self.cache.get(&cache_index) {
            if self.is_timestamp_valid(&cache_entry.timestamp) {
                return Some(cache_entry.forecast.clone());
            } else {
                let _ = self.cache.remove(&cache_index);
            };
        };

        None
    }

    /// Cache a `Forecast`.
    ///
    /// This will set or replace a Forecast.
    pub fn cache(&mut self, lat: Latitude, lon: Longitude, forecast: F) {
        let cache_index = CacheIndex::new(lat, lon);
        let _ = self.cache.insert(cache_index, CacheEntry::new(forecast));

        let _ = self.check_cleanup();
    }

    fn is_timestamp_valid(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> bool {
        chrono::Utc::now() - timestamp < self.ttl
    }

    /// Check if the cache len greater or equals to the `soft_len_limit` and clean the cache if its the case.
    ///
    /// Return `Some<usize>` if the cleanup was performed and how many entries where cleaned.
    /// Return `None` if the cleanup did get skipped.
    pub fn check_cleanup(&mut self) -> Option<usize> {
        if self.cache.len() >= self.soft_cache_limit {
            let cleanup = self.cleanup();
            self.cache.shrink_to(self.soft_cache_limit);
            Some(cleanup)
        } else {
            None
        }
    }

    /// Clean the cache and return how many entries where cleaned.
    ///
    /// Goes through all items and purges expired.
    pub fn cleanup(&mut self) -> usize {
        let expired_keys: Vec<CacheIndex> = self
            .cache
            .iter()
            .filter_map(|(key, cached)| {
                if !self.is_timestamp_valid(&cached.timestamp) {
                    Some(key)
                } else {
                    None
                }
            })
            .cloned()
            .collect();

        let mut count = 0;
        for k in expired_keys.iter() {
            let _ = self.cache.remove(k);
            count += 1;
        }

        tracing::debug!("Cache: Cleanded up {} entries.", count);
        if self.cache.len() > self.soft_cache_limit {
            tracing::warn!(
                "Cache: Overflowing soft len limit: Len: {} > Soft Limit: {}",
                self.cache.len(),
                self.soft_cache_limit
            );
        };

        count
    }
}

impl CacheIndex {
    fn new(lat: Latitude, lon: Longitude) -> CacheIndex {
        Self((lat * 100.0) as i16, (lon * 100.0) as i16)
    }

    fn as_coords(&self) -> (Latitude, Longitude) {
        (self.0 as Latitude / 100.0, self.1 as Longitude / 100.0)
    }
}

impl<F> CacheEntry<F>
where
    F: Debug + Clone,
{
    pub fn new(forecast: F) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            forecast,
        }
    }
}
