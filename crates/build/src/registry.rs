//! Package registry for nya

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Package metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub yanked: bool,
}

/// Registry client
pub struct RegistryClient {
    index_url: String,
    cache_dir: PathBuf,
}

impl RegistryClient {
    pub fn new(index_url: String, cache_dir: PathBuf) -> Self {
        Self { index_url, cache_dir }
    }
    
    /// Search packages
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<PackageMetadata>> {
        // In a real implementation, this would query the registry API
        Ok(Vec::new())
    }
    
    /// Get package metadata
    pub fn get_metadata(&self, name: &str, version: &str) -> Result<PackageMetadata> {
        // In a real implementation, this would fetch from registry
        Err(anyhow::anyhow!("Registry not implemented"))
    }
    
    /// Download package
    pub fn download(&self, name: &str, version: &str, dest: &PathBuf) -> Result<()> {
        // In a real implementation, this would download and verify
        Err(anyhow::anyhow!("Registry not implemented"))
    }
    
    /// Publish package
    pub fn publish(&self, package_path: &PathBuf, token: &str) -> Result<()> {
        // In a real implementation, this would upload to registry
        Err(anyhow::anyhow!("Registry not implemented"))
    }
}

/// Local registry for development
pub struct LocalRegistry {
    packages_dir: PathBuf,
}

impl LocalRegistry {
    pub fn new(packages_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&packages_dir)?;
        Ok(Self { packages_dir })
    }
    
    pub fn add_package(&self, name: &str, version: &str, package_path: &PathBuf) -> Result<()> {
        let dest = self.packages_dir.join(name).join(version);
        std::fs::create_dir_all(&dest)?;
        
        // Copy package
        copy_dir(package_path, &dest)?;
        
        Ok(())
    }
    
    pub fn get_package(&self, name: &str, version: &str) -> Option<PathBuf> {
        let path = self.packages_dir.join(name).join(version);
        if path.exists() { Some(path) } else { None }
    }
}

fn copy_dir(src: &PathBuf, dest: &PathBuf) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        
        if src_path.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
            copy_dir(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}