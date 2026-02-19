use std::collections::HashMap;

use crate::{Latitude, Longitude, forecast::essential};

#[derive(Debug)]
pub struct Cache {
    cache: HashMap<CacheIndex, CacheEntry>,
    ttl: chrono::TimeDelta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CacheIndex(pub i16, pub i16);

#[derive(Debug, PartialEq, PartialOrd)]
struct CacheEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub forecast: essential::Forecast,
}

impl Cache {
    pub fn new(ttl: chrono::TimeDelta) -> Self {
        Self {
            ttl,
            cache: HashMap::with_capacity(16 * ttl.num_hours() as usize),
        }
    }

    /// Lookup a `Forecast` in `Cache`.
    ///
    /// This will return `None` if either there is no cached Forecast or if the forecast is expired.
    pub fn lookup(&mut self, lat: Latitude, lon: Longitude) -> Option<essential::Forecast> {
        let cache_index = CacheIndex::new(lat, lon);
        if let Some(cache_entry) = self.cache.get(&cache_index) {
            if chrono::Utc::now() - cache_entry.timestamp < self.ttl {
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
    pub fn cache(&mut self, lat: Latitude, lon: Longitude, forecast: essential::Forecast) {
        let cache_index = CacheIndex::new(lat, lon);
        let _ = self.cache.insert(cache_index, CacheEntry::new(forecast));
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

impl CacheEntry {
    pub fn new(forecast: essential::Forecast) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            forecast,
        }
    }
}
