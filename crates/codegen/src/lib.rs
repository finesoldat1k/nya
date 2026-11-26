//! nya-codegen: LLVM codegen backend for nya

pub mod codegen;

use anyhow::Result;
use hir::HirModule;

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