//! Generic caching layer for Mattermost API responses
//!
//! This module provides thread-safe, TTL-based caching to reduce redundant API calls
//! and improve performance. Caches are automatically invalidated via WebSocket events.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A cache entry with TTL expiration
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry with TTL
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    /// Check if this entry has expired
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Generic thread-safe cache with TTL-based expiration
///
/// # Type Parameters
/// * `T` - The type of value to cache (must be Clone)
///
/// # Features
/// - Thread-safe: Uses Arc<RwLock> for concurrent access
/// - TTL-based expiration: Entries automatically expire after configured duration
/// - Automatic cleanup: Expired entries are removed on access
/// - Memory efficient: Only stores unexpired entries
#[derive(Debug, Clone)]
pub struct Cache<T: Clone> {
    /// Storage for cache entries
    entries: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    /// Time-to-live for cache entries
    ttl: Duration,
}

impl<T: Clone> Cache<T> {
    /// Create a new cache with specified TTL
    ///
    /// # Arguments
    /// * `ttl` - Time-to-live duration for cache entries
    ///
    /// # Example
    /// ```ignore
    /// use std::time::Duration;
    /// let cache = Cache::<String>::new(Duration::from_secs(300)); // 5 minute TTL
    /// ```
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Get a value from the cache
    ///
    /// Returns None if:
    /// - Key does not exist
    /// - Entry has expired
    ///
    /// Expired entries are automatically removed during this operation.
    ///
    /// # Arguments
    /// * `key` - The cache key to look up
    ///
    /// # Returns
    /// The cached value if present and not expired, None otherwise
    pub async fn get(&self, key: &str) -> Option<T> {
        let entries = self.entries.read().await;

        if let Some(entry) = entries.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
            // Entry is expired, will be removed in cleanup
        }

        drop(entries);

        // Remove expired entry if found
        self.remove_if_expired(key).await;

        None
    }

    /// Set a value in the cache
    ///
    /// Stores the value with the configured TTL. If a value already exists
    /// for this key, it will be replaced.
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `value` - The value to cache
    pub async fn set(&self, key: String, value: T) {
        let mut entries = self.entries.write().await;
        entries.insert(key, CacheEntry::new(value, self.ttl));
    }

    /// Invalidate (remove) a specific cache entry
    ///
    /// This is typically called when a WebSocket event indicates
    /// that the cached data has been updated server-side.
    ///
    /// # Arguments
    /// * `key` - The cache key to invalidate
    ///
    /// # Returns
    /// true if an entry was removed, false if key didn't exist
    pub async fn invalidate(&self, key: &str) -> bool {
        let mut entries = self.entries.write().await;
        entries.remove(key).is_some()
    }

    /// Clear all entries from the cache
    ///
    /// This is useful when major structural changes occur (e.g., team changes)
    /// that may affect multiple cached entries.
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// Remove a key only if it's expired
    ///
    /// This is used internally for cleanup during get operations.
    async fn remove_if_expired(&self, key: &str) {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get(key) {
            if entry.is_expired() {
                entries.remove(key);
            }
        }
    }

    /// Clean up all expired entries
    ///
    /// This can be called periodically to free memory from expired entries
    /// that haven't been accessed recently.
    ///
    /// # Returns
    /// The number of entries removed
    pub async fn cleanup_expired(&self) -> usize {
        let mut entries = self.entries.write().await;
        let before_count = entries.len();

        // Collect keys of expired entries
        let expired_keys: Vec<String> = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        // Remove expired entries
        for key in &expired_keys {
            entries.remove(key);
        }

        before_count - entries.len()
    }

    /// Get the current number of cached entries
    ///
    /// This includes both expired and unexpired entries.
    /// Use cleanup_expired() first to get only active entries.
    ///
    /// # Returns
    /// The number of entries currently in cache
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Check if the cache is empty
    ///
    /// # Returns
    /// true if the cache contains no entries
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// A tuple of (total_entries, expired_entries)
    pub async fn stats(&self) -> (usize, usize) {
        let entries = self.entries.read().await;
        let total = entries.len();
        let expired = entries.values().filter(|e| e.is_expired()).count();
        (total, expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache = Cache::new(Duration::from_secs(300));

        cache.set("key1".to_string(), "value1".to_string()).await;

        let value = cache.get("key1").await;
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_get_nonexistent() {
        let cache: Cache<String> = Cache::new(Duration::from_secs(300));

        let value = cache.get("nonexistent").await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        // Create cache with very short TTL
        let cache = Cache::new(Duration::from_millis(100));

        cache.set("key1".to_string(), "value1".to_string()).await;

        // Value should be present immediately
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;

        // Value should be expired now
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = Cache::new(Duration::from_secs(300));

        cache.set("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        // Invalidate the entry
        let removed = cache.invalidate("key1").await;
        assert!(removed);

        // Should not be present anymore
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_invalidate_nonexistent() {
        let cache: Cache<String> = Cache::new(Duration::from_secs(300));

        let removed = cache.invalidate("nonexistent").await;
        assert!(!removed);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = Cache::new(Duration::from_secs(300));

        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;
        cache.set("key3".to_string(), "value3".to_string()).await;

        assert_eq!(cache.len().await, 3);

        cache.clear().await;

        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_cache_cleanup_expired() {
        let cache = Cache::new(Duration::from_millis(100));

        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;

        // Add a new entry that hasn't expired
        cache.set("key3".to_string(), "value3".to_string()).await;

        assert_eq!(cache.len().await, 3);

        // Cleanup expired entries
        let removed = cache.cleanup_expired().await;
        assert_eq!(removed, 2);
        assert_eq!(cache.len().await, 1);

        // key3 should still be present
        assert_eq!(cache.get("key3").await, Some("value3".to_string()));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = Cache::new(Duration::from_millis(100));

        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;

        let (total, expired) = cache.stats().await;
        assert_eq!(total, 2);
        assert_eq!(expired, 0);

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;

        let (total, expired) = cache.stats().await;
        assert_eq!(total, 2);
        assert_eq!(expired, 2);
    }

    #[tokio::test]
    async fn test_cache_replace_existing() {
        let cache = Cache::new(Duration::from_secs(300));

        cache.set("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        // Replace with new value
        cache.set("key1".to_string(), "value2".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_cache_concurrent_access() {
        use std::sync::Arc as StdArc;

        let cache = StdArc::new(Cache::new(Duration::from_secs(300)));
        let mut handles = vec![];

        // Spawn multiple tasks writing to cache
        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = tokio::spawn(async move {
                cache_clone.set(format!("key{}", i), format!("value{}", i)).await;
            });
            handles.push(handle);
        }

        // Wait for all writes to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all entries are present
        assert_eq!(cache.len().await, 10);

        // Spawn multiple tasks reading from cache
        let mut handles = vec![];
        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = tokio::spawn(async move {
                let value = cache_clone.get(&format!("key{}", i)).await;
                assert_eq!(value, Some(format!("value{}", i)));
            });
            handles.push(handle);
        }

        // Wait for all reads to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_cache_is_empty() {
        let cache: Cache<String> = Cache::new(Duration::from_secs(300));

        assert!(cache.is_empty().await);

        cache.set("key1".to_string(), "value1".to_string()).await;
        assert!(!cache.is_empty().await);

        cache.clear().await;
        assert!(cache.is_empty().await);
    }
}
