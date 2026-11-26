//! nya-frontend: Frontend for the nya programming language
//!
//! This crate contains the lexer, parser, AST, and diagnostics.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod diagnostics;

use anyhow::Result;

/// Compile a nya source file to an AST
pub fn compile_source(source: &str, filename: &str) -> Result<ast::Module> {
    let tokens = lexer::tokenize(source, filename)?;
    let module = parser::parse(tokens, filename)?;
    Ok(module)
}