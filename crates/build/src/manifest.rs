//! nya.toml manifest handling

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Package manifest (nya.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Package,
    pub dependencies: HashMap<String, Dependency>,
    pub dev_dependencies: HashMap<String, Dependency>,
    pub build_dependencies: HashMap<String, Dependency>,
    pub target: Option<TargetConfig>,
    pub workspace: Option<WorkspaceConfig>,
    pub lib: Option<LibConfig>,
    pub bins: Vec<BinConfig>,
    pub tests: Vec<TestConfig>,
    pub benches: Vec<BenchConfig>,
    pub examples: Vec<ExampleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub edition: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(DetailedDependency),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDependency {
    pub version: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub path: Option<PathBuf>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub package: Option<String>,
    pub default_features: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub target_triple: Option<String>,
    pub linker: Option<String>,
    pub link_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub members: Vec<String>,
    pub exclude: Vec<String>,
    pub default_members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibConfig {
    pub name: Option<String>,
    pub path: Option<PathBuf>,
    pub crate_type: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinConfig {
    pub name: String,
    pub path: PathBuf,
    pub required_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub name: String,
    pub path: PathBuf,
    pub required_features: Vec<String>,
    pub test: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchConfig {
    pub name: String,
    pub path: PathBuf,
    pub required_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleConfig {
    pub name: String,
    pub path: PathBuf,
    pub required_features: Vec<String>,
}

impl Manifest {
    /// Load manifest from nya.toml
    pub fn load(project_root: &Path) -> Result<Self> {
        let manifest_path = project_root.join("nya.toml");
        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", manifest_path.display(), e))?;
        
        let manifest: Manifest = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", manifest_path.display(), e))?;
        
        Ok(manifest)
    }
    
    /// Save manifest to nya.toml
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let manifest_path = project_root.join("nya.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize manifest: {}", e))?;
        
        std::fs::write(&manifest_path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", manifest_path.display(), e))?;
        
        Ok(())
    }
    
    /// Add a dependency
    pub fn add_dependency(&mut self, dep_spec: &str) -> Result<()> {
        // Parse dependency spec: name@version or just name
        let (name, version) = if let Some(at_idx) = dep_spec.find('@') {
            (&dep_spec[..at_idx], &dep_spec[at_idx + 1..])
        } else {
            (dep_spec, "*")
        };
        
        self.dependencies.insert(
            name.to_string(),
            Dependency::Detailed(DetailedDependency {
                version: version.to_string(),
                features: Vec::new(),
                optional: false,
                path: None,
                git: None,
                branch: None,
                tag: None,
                rev: None,
                package: None,
                default_features: true,
            }),
        );
        
        Ok(())
    }
    
    /// Create a new project
    pub fn create_new_project(name: &str, path: &Path, bin: bool) -> Result<()> {
        std::fs::create_dir_all(path)?;
        
        let mut manifest = Manifest {
            package: Package {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                edition: Some("2024".to_string()),
                authors: Vec::new(),
                description: None,
                license: Some("MIT OR Apache-2.0".to_string()),
                repository: None,
                homepage: None,
                documentation: None,
                keywords: Vec::new(),
                categories: Vec::new(),
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            build_dependencies: HashMap::new(),
            target: None,
            workspace: None,
            lib: if !bin { Some(LibConfig {
                name: Some(name.to_string()),
                path: Some(PathBuf::from("src/lib.nya")),
                crate_type: vec!["lib".to_string()],
            })} else { None },
            bins: if bin { vec![BinConfig {
                name: name.to_string(),
                path: PathBuf::from("src/main.nya"),
                required_features: Vec::new(),
            }] } else { Vec::new() },
            tests: Vec::new(),
            benches: Vec::new(),
            examples: Vec::new(),
        };
        
        // Create src directory
        let src_dir = path.join("src");
        std::fs::create_dir_all(&src_dir)?;
        
        // Create main file
        if bin {
            let main_content = r#"fn main() {
    println!("Hello, nya! 🐱")
}
"#;
            std::fs::write(src_dir.join("main.nya"), main_content)?;
        } else {
            let lib_content = r#"pub fn hello() -> String {
    "Hello, nya! 🐱".to_string()
}
"#;
            std::fs::write(src_dir.join("lib.nya"), lib_content)?;
        }
        
        // Create .gitignore
        let gitignore = r#"target/
*.nya~
"#;
        std::fs::write(path.join(".gitignore"), gitignore)?;
        
        // Save manifest
        manifest.save(path)?;
        
        Ok(())
    }
}