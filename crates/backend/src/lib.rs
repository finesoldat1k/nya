//! nya-backend: LLVM codegen backend for nya
//!
//! This crate lowers HIR to LLVM IR using inkwell.

pub mod codegen;
pub mod types;
pub mod context;

use anyhow::Result;
use nya_middle::hir::HirModule;

/// Compile a HIR module to LLVM IR
pub fn compile_module(module: HirModule) -> Result<()> {
    let mut codegen = codegen::Codegen::new()?;
    codegen.compile_module(module)?;
    Ok(())
}

/// Emit object file
pub fn emit_object(module: HirModule, output_path: &str) -> Result<()> {
    let mut codegen = codegen::Codegen::new()?;
    codegen.compile_module(module)?;
    codegen.emit_object(output_path)
}

/// Emit executable
pub fn emit_executable(module: HirModule, output_path: &str) -> Result<()> {
    let mut codegen = codegen::Codegen::new()?;
    codegen.compile_module(module)?;
    codegen.emit_executable(output_path)
}