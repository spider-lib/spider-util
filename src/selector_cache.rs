use once_cell::sync::Lazy;
use parking_lot::RwLock;
use scraper::Selector;
use std::collections::HashMap;

// Global selector cache to avoid repeated compilation
static SELECTOR_CACHE: Lazy<RwLock<HashMap<String, Selector>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Get a compiled selector from the cache or compile and store it if not present
pub fn get_cached_selector(selector_str: &str) -> Option<Selector> {
    {
        let cache = SELECTOR_CACHE.read();
        if let Some(cached) = cache.get(selector_str) {
            return Some(cached.clone());
        }
    }

    match Selector::parse(selector_str) {
        Ok(selector) => {
            {
                let mut cache = SELECTOR_CACHE.write();
                if let Some(cached) = cache.get(selector_str) {
                    return Some(cached.clone());
                }
                cache.insert(selector_str.to_string(), selector.clone());
            }
            Some(selector)
        }
        Err(_) => None,
    }
}

/// Pre-warm the selector cache with commonly used selectors
pub fn prewarm_cache() {
    let common_selectors = vec![
        "a[href]",
        "link[href]",
        "script[src]",
        "img[src]",
        "audio[src]",
        "video[src]",
        "source[src]",
        "form[action]",
        "iframe[src]",
        "frame[src]",
        "embed[src]",
        "object[data]",
    ];

    for selector_str in common_selectors {
        get_cached_selector(selector_str);
    }
}

