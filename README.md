# nya Programming Language

> A cute, fast, and safe systems programming language 🐱

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/nya-lang/nya)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org)

---

nya is a compiled systems programming language written in Rust, targeting maximum performance with a "cute" developer experience. It combines Rust's memory safety (simplified borrowing) with TypeScript-like ergonomics.

## Features

- **🐱 Cute Syntax** - `meow` constructors, `nya` keywords, emoji in errors
- **🚀 Maximum Performance** - LLVM backend, zero-cost abstractions
- **🔒 Memory Safe** - Simplified borrow checker (ownership + borrowing, no explicit lifetimes)
- **📦 Modern Tooling** - Built-in package manager (`nya build`, `nya test`, `nya run`)
- **🎯 Cross-Platform** - x86_64, x86, ARM, ARM64 via LLVM
- **🧩 TypeScript-like Ergonomics** - Type inference, generics, pattern matching

## Quick Example

```nya
// main.nya
struct Cat {
    name: String,
    age: u8,
    lives: u8 = 9,
}

impl Cat {
    // Cute constructor name!
    fn meow(name: String, age: u8) -> Self {
        Self { name, age, lives: 9 }
    }

    pub fn speak(&self) {
        println!("{} says: Nya~!", self.name)
    }

    pub fn birthday(&mut self) {
        self.age += 1
        if self.age > 15 { self.lives -= 1 }
    }

    static fn species() -> String { "Felis catus".to_string() }
}

fn main() {
    var kitty = Cat::meow("Mochi".to_string(), 3)
    kitty.speak()
    kitty.birthday()
    
    const PI = 3.14159
    var result = compute(42)?
    println!("Result: {}", result)
}

fn compute(x: i32) -> Result<i32, String> {
    if x < 0 { return Err("negative".to_string()) }
    Ok(x * 2)
}
```

```bash
$ nya run
Mochi says: Nya~!
Result: 84
```

## Installation

```bash
# From source (requires Rust 1.75+)
git clone https://github.com/nya-lang/nya
cd nya
cargo install --path crates/driver

# Or download pre-built binaries from releases
```

## Documentation

- [Getting Started Guide](docs/getting-started.md) - Your first nya project
- [Language Guide](docs/language-guide.md) - Complete language reference
- [Standard Library](docs/std.md) - Core library documentation
- [Contributing](CONTRIBUTING.md) - How to contribute

## Project Structure

```
nya/
├── crates/
│   ├── core/         # Core library (Option, Result, Iterator, etc.)
│   ├── stdlib/       # Standard library (collections, IO, fs, etc.)
│   ├── lexer/        # Tokenizer with cute keywords
│   ├── parser/       # Recursive descent parser
│   ├── ast/          # Abstract Syntax Tree
│   ├── diagnostics/  # Error reporting
│   ├── resolver/     # Name resolution
│   ├── ty/           # Type checking
│   ├── borrow/       # Borrow checking
│   ├── hir/          # High-level IR
│   ├── types/        # LLVM type lowering
│   ├── context/      # LLVM context management
│   ├── codegen/      # LLVM code generation
│   ├── manifest/     # nya.toml parsing
│   ├── builder/      # Build execution
│   ├── cache/        # Build caching
│   ├── dep_resolver/ # Dependency resolution
│   ├── registry/     # Package registry
│   ├── driver/       # CLI driver (nya CLI)
│   └── test/         # Test framework
```

## Design Philosophy

| Principle | Description |
|-----------|-------------|
| **Cute** | Language keywords and errors are delightful |
| **Fast** | LLVM backend, zero-cost abstractions |
| **Safe** | Simplified borrow checker prevents memory bugs |
| **Ergonomic** | Type inference, pattern matching, modern syntax |
| **Practical** | Built-in tooling, FFI, cross-compilation |

## Roadmap

- [x] Core language (lexer, parser, AST)
- [x] Type checking & borrow checking
- [x] LLVM codegen (basic)
- [x] Build system & package manager
- [ ] Async/await with green threads
- [ ] Trait system
- [ ] WASM target
- [ ] LSP server
- [ ] Self-hosting compiler

## License

Licensed under either of:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

## Community

- [Discord](https://discord.gg/nya-lang) - Chat with nya developers
- [GitHub Discussions](https://github.com/nya-lang/nya/discussions) - Ask questions
- [Twitter](https://twitter.com/nya_lang) - Updates and announcements

---

*Made with 💖 by the nya community*