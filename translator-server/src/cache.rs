// File: translator-server/src/cache.rs

use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use crate::state::SharedState;

/// Generate a cache key for a translation.
/// Key format: `translation:{sha256(text)}:{src_lang}:{tgt_lang}`
fn cache_key(text: &str, src_lang: &str, tgt_lang: &str) -> String {
    let hash = Sha256::digest(text.as_bytes());
    format!("translation:{:x}:{}:{}", hash, src_lang, tgt_lang)
}

pub struct TranslationCache;

impl TranslationCache {
    /// Get a cached translation if it exists.
    pub async fn get(
        state: &SharedState,
        text: &str,
        src_lang: &str,
        tgt_lang: &str,
    ) -> Option<String> {
        let key = cache_key(text, src_lang, tgt_lang);
        let result: Result<Option<String>, _> = state.redis.clone().get(&key).await;

        match result {
            Ok(Some(cached)) => {
                tracing::debug!(key = %key, "Cache HIT");
                Some(cached)
            }
            Ok(None) => {
                tracing::debug!(key = %key, "Cache MISS");
                None
            }
            Err(e) => {
                tracing::warn!(error = %e, "Redis GET failed");
                None
            }
        }
    }

    /// Store a translation in the cache.
    pub async fn set(
        state: &SharedState,
        text: &str,
        src_lang: &str,
        tgt_lang: &str,
        translation: &str,
    ) {
        let key = cache_key(text, src_lang, tgt_lang);
        let ttl = state.config.cache_ttl_seconds;

        let result: Result<(), _> = state
            .redis
            .clone()
            .set_ex(&key, translation, ttl)
            .await;

        if let Err(e) = result {
            tracing::warn!(error = %e, "Redis SET failed");
        }
    }

    /// Get multiple cached translations at once using MGET.
    pub async fn get_batch(
        state: &SharedState,
        texts: &[String],
        src_lang: &str,
        tgt_lang: &str,
    ) -> Vec<Option<String>> {
        if texts.is_empty() {
            return Vec::new();
        }

        let keys: Vec<String> = texts
            .iter()
            .map(|t| cache_key(t, src_lang, tgt_lang))
            .collect();

        let result: Result<Vec<Option<String>>, _> = state.redis.clone().mget(&keys).await;

        match result {
            Ok(values) => {
                let hits = values.iter().filter(|v| v.is_some()).count();
                tracing::debug!(hits = hits, misses = values.len() - hits, "Batch cache lookup");
                values
            }
            Err(e) => {
                tracing::warn!(error = %e, "Redis MGET failed");
                vec![None; texts.len()]
            }
        }
    }

    /// Store multiple translations using Redis pipeline.
    pub async fn set_batch(
        state: &SharedState,
        texts: &[String],
        src_lang: &str,
        tgt_lang: &str,
        translations: &[String],
    ) {
        if texts.is_empty() {
            return;
        }

        let ttl = state.config.cache_ttl_seconds;
        let mut pipe = redis::pipe();

        for (text, translation) in texts.iter().zip(translations.iter()) {
            let key = cache_key(text, src_lang, tgt_lang);
            pipe.set_ex(&key, translation, ttl);
        }

        let result: Result<(), _> = pipe.query_async(&mut state.redis.clone()).await;

        if let Err(e) = result {
            tracing::warn!(error = %e, "Redis pipeline SET failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_deterministic() {
        let key1 = cache_key("Hello", "eng_Latn", "fra_Latn");
        let key2 = cache_key("Hello", "eng_Latn", "fra_Latn");
        assert_eq!(key1, key2);
    }

    #[test]
    fn cache_key_differs_for_different_texts() {
        let key1 = cache_key("Hello", "eng_Latn", "fra_Latn");
        let key2 = cache_key("World", "eng_Latn", "fra_Latn");
        assert_ne!(key1, key2);
    }

    #[test]
    fn cache_key_differs_for_different_languages() {
        let key1 = cache_key("Hello", "eng_Latn", "fra_Latn");
        let key2 = cache_key("Hello", "eng_Latn", "spa_Latn");
        assert_ne!(key1, key2);
    }
}