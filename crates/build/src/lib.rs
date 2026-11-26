//! nya-build: Build system and package manager for nya
//!
//! This crate handles project building, dependency resolution, and package management.

pub mod manifest;
pub mod resolver;
pub mod builder;
pub mod cache;
pub mod registry;

use anyhow::Result;
use std::path::Path;

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub target_dir: std::path::PathBuf,
    pub release: bool,
    pub jobs: Option<usize>,
    pub verbose: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target_dir: std::path::PathBuf::from("target"),
            release: false,
            jobs: None,
            verbose: false,
        }
    }
}

/// Build a nya project
pub fn build_project(project_root: &Path, config: BuildConfig) -> Result<()> {
    let manifest = manifest::Manifest::load(project_root)?;
    let mut builder = builder::Builder::new(config)?;
    builder.build(&manifest)
}

/// Create a new nya project
pub fn new_project(name: &str, path: &Path, bin: bool) -> Result<()> {
    manifest::create_new_project(name, path, bin)
}

/// Add a dependency
pub fn add_dependency(project_root: &Path, dep: &str) -> Result<()> {
    let mut manifest = manifest::Manifest::load(project_root)?;
    manifest.add_dependency(dep)?;
    manifest.save(project_root)
}

/// Run tests
pub fn run_tests(project_root: &Path, config: BuildConfig) -> Result<()> {
    let manifest = manifest::Manifest::load(project_root)?;
    let mut builder = builder::Builder::new(config)?;
    builder.test(&manifest)
}

/// Run a binary
pub fn run_binary(project_root: &Path, bin_name: Option<&str>, args: &[String], config: BuildConfig) -> Result<()> {
    let manifest = manifest::Manifest::load(project_root)?;
    let mut builder = builder::Builder::new(config)?;
    builder.run(&manifest, bin_name, args)
}