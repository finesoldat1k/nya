# Getting Started with nya

Welcome to nya! This guide will walk you through creating your first nya project.

## Prerequisites

- **Rust 1.75+** - Install from [rustup.rs](https://rustup.rs)
- **LLVM 17+** - Required for code generation
  - Ubuntu/Debian: `apt install llvm-17-dev clang`
  - macOS: `brew install llvm@17`
  - Windows: Download from [llvm.org](https://llvm.org)

## Installation

### From Source

```bash
git clone https://github.com/finesoldat1k/nya
cd nya
cargo install --path crates/driver
```

### Verify Installation

```bash
nya --version
# nya 0.1.0
```

## Your First Project

### Create a Binary Project

```bash
nya new hello --bin
cd hello
```

This creates:
```
hello/
├── nya.toml          # Project manifest
├── src/
│   └── main.nya      # Entry point
└── .gitignore
```

### Project Manifest (nya.toml)

```toml
[package]
name = "hello"
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"

[dependencies]
# Add dependencies here
# nya = "0.1"
```

### Hello World

Edit `src/main.nya`:

```nya
fn main() {
    println!("Hello, nya! 🐱")
}
```

### Build and Run

```bash
# Development build
nya build
nya run

# Release build (optimized)
nya build --release
nya run
```

Output:
```
Building hello v0.1.0
   Compiling core v0.1.0
   Compiling hello v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2.3s

$ nya run
Hello, nya! 🐱
```

## Creating a Library

```bash
nya new mylib
cd mylib
```

This creates a library project with `src/lib.nya`:

```nya
// src/lib.nya
pub fn greet(name: String) -> String {
    format!("Hello, {}! 🐱", name)
}

pub struct Cat {
    pub name: String,
    pub age: u8,
}

impl Cat {
    pub fn meow(name: String, age: u8) -> Self {
        Self { name, age, lives: 9 }
    }

    pub fn speak(&self) {
        println!("{} says: Nya~!", self.name)
    }
}
```

### Building the Library

```bash
nya build
```

This produces `target/libmylib.a` (static library).

## Adding Dependencies

```bash
nya add nya-collections@0.1
```

This adds to `nya.toml`:

```toml
[dependencies]
nya-collections = "0.1"
```

Then use in your code:

```nya
use nya_collections::HashMap

fn main() {
    var map = HashMap::new()
    map.insert("key", "value")
    println!("{}", map.get("key").unwrap())
}
```

## Running Tests

Create test files in `tests/`:

```nya
// tests/basic.nya
#[test]
fn test_cat_creation() {
    var cat = Cat::meow("Mochi".to_string(), 3)
    assert_eq!(cat.name, "Mochi")
    assert_eq!(cat.age, 3)
    assert_eq!(cat.lives, 9)
}

#[test]
fn test_cat_birthday() {
    var cat = Cat::meow("Mochi".to_string(), 14)
    cat.birthday()
    assert_eq!(cat.age, 15)
    assert_eq!(cat.lives, 9)
    
    cat.birthday()  // Age 16, loses a life
    assert_eq!(cat.lives, 8)
}
```

Run tests:

```bash
nya test
```

Output:
```
Running tests...
test test_cat_creation ... ok
test test_cat_birthday ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Project Layout

```
my-project/
├── nya.toml              # Project configuration
├── src/
│   ├── main.nya          # Binary entry point (for bins)
│   └── lib.nya           # Library root (for libs)
├── tests/
│   └── *.nya             # Integration tests
├── examples/
│   └── *.nya             # Example binaries
├── benches/
│   └── *.nya             # Benchmarks
├── target/               # Build artifacts (gitignored)
│   ├── debug/
│   └── release/
└── .gitignore
```

## Build Configuration

### Release Builds

```bash
nya build --release
```

Optimizations:
- LLVM O3 optimization
- LTO (Link Time Optimization)
- Strip symbols
- Panic = abort

### Cross Compilation

```bash
# Target ARM64 Linux
nya build --target aarch64-unknown-linux-gnu

# Target WASM
nya build --target wasm32-unknown-unknown
```

### Build Profiles

Configure in `nya.toml`:

```toml
[profile.dev]
opt-level = 1
debug = true

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
```

## IDE Support

### VS Code

Install the **nya Language Support** extension (coming soon) for:
- Syntax highlighting
- Error diagnostics
- Go to definition
- Completion

### Other Editors

Use **rust-analyzer** with the nya project - it understands the Rust-based compiler.

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| `llvm-config not found` | Install LLVM development packages |
| `linker 'cc' not found` | Install build-essential / Xcode Command Line Tools |
| `nya: command not found` | Add `~/.cargo/bin` to PATH |
| Slow builds | Use `nya build --release` for incremental builds |

### Getting Help

- [GitHub Discussions](https://github.com/nya-lang/nya/discussions)
- [Discord](https://discord.gg/nya-lang)
- [GitHub Issues](https://github.com/nya-lang/nya/issues)

## Next Steps

- Read the [Language Guide](language-guide.md) for complete syntax reference
- Explore the [Standard Library](std.md)
- Check out [Examples](https://github.com/nya-lang/nya/tree/main/examples)
- Join the community on [Discord](https://discord.gg/nya-lang)

---

*Happy coding! 🐱*