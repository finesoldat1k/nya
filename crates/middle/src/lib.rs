//! nya-middle: Middle-end for the nya programming language
//!
//! This crate contains the resolver, type checker, borrow checker, and HIR.

pub mod resolver;
pub mod ty;
pub mod borrow;
pub mod hir;

use anyhow::Result;
use nya_frontend::ast::Module;

/// Compile a nya module through the middle-end
pub fn compile_module(module: Module) -> Result<hir::HirModule> {
    // Name resolution
    let resolved = resolver::resolve(module)?;
    
    // Type checking
    let typed = ty::check(resolved)?;
    
    // Borrow checking
    let checked = borrow::check(typed)?;
    
    // Lower to HIR
    let hir = hir::lower(checked)?;
    
    Ok(hir)
}