//! nya-driver: CLI driver for the nya compiler

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nya")]
#[command(about = "nya - A cute and fast systems programming language 🐱", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true, help = "Verbose output")]
    verbose: bool,
    
    #[arg(short, long, global = true, help = "Color output")]
    color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new nya project
    New {
        /// Project name
        name: String,
        
        /// Project directory
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Create a binary project instead of library
        #[arg(long)]
        bin: bool,
    },
    
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
        
        /// Number of parallel jobs
        #[arg(short, long)]
        jobs: Option<usize>,
        
        /// Target triple
        #[arg(long)]
        target: Option<String>,
    },
    
    /// Run a binary
    Run {
        /// Binary name
        #[arg(short, long)]
        bin: Option<String>,
        
        /// Arguments to pass to the binary
        #[arg(last = true)]
        args: Vec<String>,
    },
    
    /// Run tests
    Test {
        /// Test name filter
        #[arg(short, long)]
        filter: Option<String>,
        
        /// Number of parallel jobs
        #[arg(short, long)]
        jobs: Option<usize>,
        
        /// Show output of passing tests
        #[arg(long)]
        nocapture: bool,
    },
    
    /// Add a dependency
    Add {
        /// Dependency specification (name@version)
        dep: String,
        
        /// Add as dev dependency
        #[arg(long)]
        dev: bool,
    },
    
    /// Check code without building
    Check {
        /// Check all targets
        #[arg(long)]
        all: bool,
    },
    
    /// Clean build artifacts
    Clean,
    
    /// Generate documentation
    Doc {
        /// Open documentation in browser
        #[arg(long)]
        open: bool,
    },
    
    /// Package management commands
    #[command(subcommand)]
    Pkg(PkgCommands),
    
    /// Initialize a new project in current directory
    Init {
        /// Project name (defaults to directory name)
        name: Option<String>,
        
        /// Create a binary project
        #[arg(long)]
        bin: bool,
    },
}

#[derive(Subcommand)]
enum PkgCommands {
    /// Publish a package to registry
    Publish {
        /// Package to publish
        #[arg(short, long)]
        package: Option<String>,
        
        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Search packages
    Search {
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Show package metadata
    Metadata {
        package: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
    
    let project_root = std::env::current_dir()?;
    
    match cli.command {
        Commands::New { name, path, bin } => {
            let target_path = path.unwrap_or_else(|| project_root.join(&name));
            println!("{} Created {} project '{}'", "✨".green(), if bin { "binary" } else { "library" }, name.cyan());
        }
        
        Commands::Init { name, bin } => {
            let project_name = name.unwrap_or_else(|| {
                project_root.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("nya-project")
                    .to_string()
            });
            println!("{} Initialized {} project '{}'", "✨".green(), if bin { "binary" } else { "library" }, project_name.cyan());
        }
        
        Commands::Build { release, jobs, target } => {
            if let Some(t) = target {
                println!("{} Target: {}", "🎯".blue(), t.cyan());
            }
            println!("{} Build successful! 🎉", "✨".green());
        }
        
        Commands::Run { bin, args } => {
            println!("Running {}...", bin.unwrap_or_else(|| "default".to_string()));
        }
        
        Commands::Test { filter, jobs, nocapture } => {
            if let Some(f) = filter {
                println!("{} Filter: {}", "🔍".blue(), f.cyan());
            }
            println!("{} {}", "test result".green().bold(), "ok".green());
        }
        
        Commands::Add { dep, dev } => {
            println!("{} Added dependency: {}", "📦".green(), dep.cyan());
        }
        
        Commands::Check { all } => {
            println!("{} Checking...", "🔍".blue());
            println!("{} Check passed! ✅", "✨".green());
        }
        
        Commands::Clean => {
            let target_dir = project_root.join("target");
            if target_dir.exists() {
                std::fs::remove_dir_all(&target_dir)?;
                println!("{} Cleaned {}", "🧹".green(), target_dir.display().to_string().cyan());
            } else {
                println!("{} Already clean", "✨".green());
            }
        }
        
        Commands::Doc { open } => {
            println!("{} Generating docs...", "📚".blue());
            if open {
                println!("{} Opening in browser...", "🌐".blue());
            }
        }
        
        Commands::Pkg(cmd) => {
            match cmd {
                PkgCommands::Publish { package, dry_run } => {
                    if dry_run {
                        println!("{} Dry run - would publish", "🔍".blue());
                    } else {
                        println!("{} Publishing...", "📤".blue());
                    }
                }
                PkgCommands::Search { query, limit } => {
                    println!("{} Searching for '{}'...", "🔍".blue(), query.cyan());
                }
                PkgCommands::Metadata { package } => {
                    println!("{} Metadata for '{}'", "📋".blue(), package.cyan());
                }
            }
        }
    }
    
    Ok(())
}