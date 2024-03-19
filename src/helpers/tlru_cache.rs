//! Least-recently-used cache that supports item expiry.

use std::{
    collections::HashMap,
    hash::Hash,
    rc::Rc,
    time::{Duration, Instant},
};

/// Least-recently-used cache that supports item expiry.
pub struct TLRUCache<K, V> {
    /// Hashmap of all cache entries.
    entries: HashMap<K, CacheEntry<K, V>>,
    /// The key of the newest entry.
    newest_entry: Option<K>,

    /// The maximum number of items in the cache.
    max_items: Option<usize>,
    /// The maximum age of the item before it is removed.
    max_create_age: Option<Duration>,
    /// The maximum time since last use before the item is removed.
    max_access_age: Option<Duration>,
}

impl<K: Clone + Eq + Hash, V> TLRUCache<K, V> {
    /// Creates a new empty cache.
    pub fn new(
        max_items: Option<usize>,
        max_create_age: Option<Duration>,
        max_used_age: Option<Duration>,
    ) -> Self {
        Self {
            entries: HashMap::new(),
            newest_entry: None,

            max_items,
            max_create_age,
            max_access_age: max_used_age,
        }
    }

    /// Inserts an item into the cache.
    pub fn insert(&mut self, key: K, value: V) {
        let now = Instant::now();

        let new_node = if let Some(current_newest_key) = &self.newest_entry {
            // inserting with existing entries
            let current_newest = self.entries.get_mut(&current_newest_key).unwrap();
            let current_oldest_key = current_newest.newer.clone();
            current_newest.newer = key.clone();
            let current_oldest = self.entries.get_mut(&current_oldest_key).unwrap();
            current_oldest.older = key.clone();

            CacheEntry {
                value: Rc::new(value),
                newer: current_oldest_key.clone(),
                older: current_newest_key.clone(),
                create_time: now,
                access_time: now,
            }
        } else {
            // inserting the first entry
            CacheEntry {
                value: Rc::new(value),
                newer: key.clone(),
                older: key.clone(),
                create_time: now,
                access_time: now,
            }
        };

        self.entries.insert(key.clone(), new_node);

        // update pointer to most recent item
        self.newest_entry = Some(key);

        self.gc(false, Some(now));
    }

    /// Gets an item from the cache.
    /// Returns None if the item was not found or was expired.
    pub fn get(&mut self, key: &K) -> Option<Rc<V>> {
        let now = Instant::now();

        if self.entries.contains_key(key) {
            if self.is_entry_not_expired(&self.entries[key], now) {
                // if not currently the most recently used entry, then set as the most recently used entry
                if self.newest_entry.clone().is_some_and(|x| &x != key) {
                    // remove from current location
                    let entry = &self.entries[key];
                    let current_newer_key = entry.newer.clone();
                    let current_older_key = entry.older.clone();
                    let current_newer = self.entries.get_mut(&current_newer_key).unwrap();
                    current_newer.older = current_older_key.clone();
                    let current_older = self.entries.get_mut(&current_older_key).unwrap();
                    current_older.newer = current_newer_key;

                    // insert in newest location
                    let newest_key = self.newest_entry.clone().unwrap();
                    let newest = self.entries.get_mut(&newest_key).unwrap();
                    let oldest_key = newest.newer.clone();
                    newest.newer = key.clone();
                    let oldest = self.entries.get_mut(&oldest_key).unwrap();
                    oldest.older = key.clone();

                    let entry = self.entries.get_mut(key).unwrap();
                    entry.newer = oldest_key;
                    entry.older = newest_key;
                    self.newest_entry = Some(key.clone());
                }

                Some(Rc::clone(&self.entries[key].value))
            } else {
                // if entry expired, remove it
                self.remove(key);
                None
            }
        } else {
            None
        }
    }

    /// Explicitly removes a cache entry.
    pub fn remove(&mut self, key: &K) {
        // check if we're removing the newest item
        if let Some(newest_entry) = &self.newest_entry {
            if newest_entry == key {
                // update pointer to previous item
                let older_key = &self.entries[&key].older;
                // check if we're removing the last item since the newest entry pointer still needs to be valid after removing the entry
                if key == older_key {
                    self.newest_entry = None;
                } else {
                    self.newest_entry = Some(older_key.clone());
                }
            }
        }

        // join both sides of entry to be removed
        let entry = &self.entries[key];
        let newer_key = entry.newer.clone();
        let older_key = entry.older.clone();
        let newer = self.entries.get_mut(&newer_key).unwrap();
        newer.older = older_key.clone();
        let older = self.entries.get_mut(&older_key).unwrap();
        older.newer = newer_key;

        self.entries.remove(key);
    }

    /// Removes all entries in the cache
    pub fn clear(&mut self) {
        self.newest_entry = None;
        self.entries.clear();
    }

    /// Removes expired items from the cache and reduces the cache size to the maximum.
    /// If the `full` parameter is set to false, this only removes the expired items until it finds a non-expired item.
    /// If the `full` parameter is set to true, this removes all expired items.
    pub fn gc(&mut self, full: bool, now: Option<Instant>) {
        let now = if let Some(now) = now {
            now
        } else {
            Instant::now()
        };

        if let Some(newest_entry) = self.newest_entry.clone() {
            let mut current_entry = self.entries[&newest_entry].newer.clone();

            loop {
                // save next entry for later
                let next_entry = self.entries[&current_entry].newer.clone();

                if let Some(max_items) = self.max_items {
                    // if we're over the max size of the cache, remove the entry
                    if self.entries.len() > max_items {
                        self.remove(&current_entry);
                    }
                } else if !self.is_entry_not_expired(&self.entries[&current_entry], now) {
                    // if the entry is invalid, remove it
                    self.remove(&current_entry);
                } else if !full {
                    // entry is valid - if we are not doing a full gc, then break
                    break;
                }

                // if we've processed the last entry, then break so we don't infinitely loop
                if current_entry == newest_entry {
                    break;
                }

                current_entry = next_entry;
            }
        }
    }

    /// Checks if a cache entry is expired
    fn is_entry_not_expired(&self, cache_entry: &CacheEntry<K, V>, now: Instant) -> bool {
        // check creation age
        if let Some(max_create_age) = self.max_create_age {
            if now.duration_since(cache_entry.create_time) > max_create_age {
                return false;
            }
        }

        // check access age
        if let Some(max_access_age) = self.max_access_age {
            if now.duration_since(cache_entry.access_time) > max_access_age {
                return false;
            }
        }

        true
    }
}

/// Cache entry, containing links to the newer and older keys, and timestamp metadata to track expiry
struct CacheEntry<K, V> {
    value: Rc<V>,

    newer: K,
    older: K,

    create_time: Instant,
    access_time: Instant,
}
