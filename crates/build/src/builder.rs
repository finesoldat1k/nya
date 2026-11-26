//! Build execution for nya projects

use crate::manifest::{Manifest, BinConfig, Dependency};
use anyhow::Result;
use nya_frontend::compile_source;
use nya_middle::{compile_module, resolve, check, ty, borrow, hir};
use nya_backend::{compile_module as backend_compile, emit_object, emit_executable};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use indicatif::{ProgressBar, ProgressStyle};
use colored::*;

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub target_dir: PathBuf,
    pub release: bool,
    pub jobs: Option<usize>,
    pub verbose: bool,
}

/// Build artifacts
#[derive(Debug, Clone)]
pub struct Artifacts {
    pub libs: Vec<PathBuf>,
    pub bins: Vec<PathBuf>,
    pub objects: Vec<PathBuf>,
}

/// Main builder
pub struct Builder {
    config: BuildConfig,
    target_dir: PathBuf,
    deps_cache: HashMap<String, PathBuf>,
}

impl Builder {
    pub fn new(config: BuildConfig) -> Result<Self> {
        let target_dir = config.target_dir.clone();
        std::fs::create_dir_all(&target_dir)?;
        
        Ok(Self {
            config,
            target_dir,
            deps_cache: HashMap::new(),
        })
    }
    
    /// Build a project from manifest
    pub fn build(&mut self, manifest: &Manifest) -> Result<Artifacts> {
        let mut artifacts = Artifacts {
            libs: Vec::new(),
            bins: Vec::new(),
            objects: Vec::new(),
        };
        
        // Build library if present
        if let Some(lib) = &manifest.lib {
            let lib_path = self.build_lib(manifest, lib)?;
            artifacts.libs.push(lib_path);
        }
        
        // Build binaries
        for bin in &manifest.bins {
            let bin_path = self.build_bin(manifest, bin)?;
            artifacts.bins.push(bin_path);
        }
        
        Ok(artifacts)
    }
    
    /// Build library
    fn build_lib(&mut self, manifest: &Manifest, lib: &crate::manifest::LibConfig) -> Result<PathBuf> {
        let src_path = lib.path.as_ref().map(|p| PathBuf::from(p)).unwrap_or_else(|| PathBuf::from("src/lib.nya"));
        let lib_name = lib.name.as_ref().unwrap_or(&manifest.package.name);
        
        println!("{} {}", "Building".green().bold(), lib_name.cyan());
        
        // For now, compile single file
        let source = std::fs::read_to_string(&src_path)?;
        let module = compile_source(&source, &src_path.to_string_lossy())?;
        let hir_module = compile_module(module)?;
        
        // Codegen
        let output_path = self.target_dir.join(format!("lib{}.o", lib_name));
        backend_compile(hir_module)?;
        emit_object(hir_module, &output_path)?;
        
        Ok(output_path)
    }
    
    /// Build binary
    fn build_bin(&mut self, manifest: &Manifest, bin: &BinConfig) -> Result<PathBuf> {
        let src_path = &bin.path;
        let bin_name = &bin.name;
        
        println!("{} {}", "Building".green().bold(), bin_name.cyan());
        
        let source = std::fs::read_to_string(src_path)?;
        let module = compile_source(&source, &src_path.to_string_lossy())?;
        let hir_module = compile_module(module)?;
        
        // Codegen
        let output_path = self.target_dir.join(bin_name);
        backend_compile(hir_module)?;
        emit_executable(hir_module, &output_path)?;
        
        Ok(output_path)
    }
    
    /// Run tests
    pub fn test(&mut self, manifest: &Manifest) -> Result<()> {
        println!("{} {}", "Running".green().bold(), "tests".cyan());
        
        // Build test binaries
        for test in &manifest.tests {
            if !test.test { continue; }
            
            let test_name = &test.name;
            println!("  {} {}", "test".blue(), test_name);
            
            let source = std::fs::read_to_string(&test.path)?;
            let module = compile_source(&source, &test.path.to_string_lossy())?;
            let hir_module = compile_module(module)?;
            
            let output_path = self.target_dir.join(format!("test_{}", test_name));
            backend_compile(hir_module)?;
            emit_executable(hir_module, &output_path)?;
            
            // Run test
            let output = Command::new(&output_path).output()?;
            if !output.status.success() {
                anyhow::bail!("Test {} failed: {}", test_name, String::from_utf8_lossy(&output.stderr));
            }
        }
        
        println!("{} {}", "test result".green().bold(), "ok".green());
        Ok(())
    }
    
    /// Run a binary
    pub fn run(&mut self, manifest: &Manifest, bin_name: Option<&str>, args: &[String]) -> Result<()> {
        let bin = if let Some(name) = bin_name {
            manifest.bins.iter().find(|b| b.name == name)
                .ok_or_else(|| anyhow::anyhow!("Binary '{}' not found", name))?
        } else if manifest.bins.len() == 1 {
            &manifest.bins[0]
        } else {
            anyhow::bail!("Multiple binaries found, specify one with --bin");
        };
        
        // Build first
        self.build_bin(manifest, bin)?;
        
        // Run
        let bin_path = self.target_dir.join(&bin.name);
        let mut cmd = Command::new(&bin_path);
        cmd.args(args);
        
        let status = cmd.status()?;
        if !status.success() {
            anyhow::bail!("Process exited with: {}", status);
        }
        
        Ok(())
    }
    
    /// Resolve dependencies (placeholder)
    fn resolve_deps(&mut self, manifest: &Manifest) -> Result<()> {
        for (name, dep) in &manifest.dependencies {
            println!("  {} {}", "Fetching".blue(), name);
            // In real implementation, would download from registry
        }
        Ok(())
    }
}