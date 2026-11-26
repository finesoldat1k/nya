//! Dependency resolver for nya

use crate::manifest::{Dependency, DetailedDependency};
use anyhow::Result;
use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet};

/// Resolved dependency graph
#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    pub nodes: HashMap<String, ResolvedPackage>,
    pub edges: Vec<(String, String)>, // (from, to)
}

/// Resolved package
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: Version,
    pub dependencies: HashMap<String, String>, // name -> version requirement
}

/// Dependency resolver
pub struct Resolver {
    registry_index: HashMap<String, Vec<Version>>, // package -> available versions
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            registry_index: HashMap::new(),
        }
    }
    
    /// Add package versions to index
    pub fn add_package(&mut self, name: String, versions: Vec<Version>) {
        self.registry_index.insert(name, versions);
    }
    
    /// Resolve dependencies
    pub fn resolve(&self, deps: &HashMap<String, Dependency>) -> Result<ResolvedGraph> {
        let mut graph = ResolvedGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        };
        
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        
        // Add root dependencies
        for (name, dep) in deps {
            let req = self.dep_to_req(dep)?;
            stack.push((name.clone(), req));
        }
        
        while let Some((name, req)) = stack.pop() {
            if visited.contains(&name) {
                continue;
            }
            visited.insert(name.clone());
            
            // Find best matching version
            let version = self.select_version(&name, &req)?;
            
            graph.nodes.insert(name.clone(), ResolvedPackage {
                name: name.clone(),
                version: version.clone(),
                dependencies: HashMap::new(), // Would be populated from registry
            });
            
            // Add dependencies of this package (simplified)
            // In real implementation, would fetch from registry
        }
        
        Ok(graph)
    }
    
    fn dep_to_req(&self, dep: &Dependency) -> Result<VersionReq> {
        match dep {
            Dependency::Simple(v) => VersionReq::parse(v),
            Dependency::Detailed(d) => VersionReq::parse(&d.version),
        }.map_err(|e| anyhow::anyhow!("Invalid version requirement: {}", e))
    }
    
    fn select_version(&self, name: &str, req: &VersionReq) -> Result<Version> {
        let versions = self.registry_index.get(name)
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in registry", name))?;
        
        let matching: Vec<_> = versions.iter()
            .filter(|v| req.matches(v))
            .collect();
        
        if matching.is_empty() {
            anyhow::bail!("No version of '{}' matches requirement '{}'", name, req);
        }
        
        // Select highest matching version
        Ok(matching.into_iter().max().unwrap().clone())
    }
}