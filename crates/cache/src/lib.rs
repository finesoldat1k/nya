//! Build cache for nya

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Build cache entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub output_path: PathBuf,
    pub mtime: SystemTime,
    pub deps_hash: String,
}

/// Build cache
pub struct BuildCache {
    cache_dir: PathBuf,
    entries: HashMap<String, CacheEntry>,
}

impl BuildCache {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;
        
        let index_path = cache_dir.join("index.json");
        let entries = if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };
        
        Ok(Self { cache_dir, entries })
    }
    
    pub fn get(&self, key: &str) -> Option<&CacheEntry> {
        self.entries.get(key)
    }
    
    pub fn insert(&mut self, key: String, entry: CacheEntry) -> Result<()> {
        self.entries.insert(key, entry);
        self.save()
    }
    
    pub fn save(&self) -> Result<()> {
        let index_path = self.cache_dir.join("index.json");
        let content = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(index_path, content)?;
        Ok(())
    }
    
    pub fn compute_key(&self, source: &str, flags: &[String]) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(source.as_bytes());
        for flag in flags {
            hasher.update(flag.as_bytes());
        }
        hex::encode(hasher.finalize().as_bytes())
    }
}