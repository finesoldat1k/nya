//! LLVM context management for nya

use inkwell::context::Context;
use inkwell::module::Module as LlvmModule;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, CodeModel, RelocMode, FileType};
use inkwell::OptimizationLevel;
use std::path::Path;

/// Wrapper around LLVM context and module
pub struct CodegenContext {
    pub context: Context,
    pub module: LlvmModule,
    pub target_machine: TargetMachine,
}

impl CodegenContext {
    /// Create a new codegen context for the target triple
    pub fn new(target_triple: Option<&str>) -> anyhow::Result<Self> {
        // Initialize LLVM targets
        Target::initialize_all(&InitializationConfig::default());
        
        let context = Context::create();
        let module = context.create_module("nya_module");
        
        // Determine target triple
        let triple = target_triple
            .map(|s| s.to_string())
            .unwrap_or_else(|| inkwell::targets::TargetMachine::get_default_triple().to_string());
        
        let target = Target::from_triple(&inkwell::targets::TargetTriple::create(&triple))?;
        
        let target_machine = target
            .create_target_machine(
                &inkwell::targets::TargetTriple::create(&triple),
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| anyhow::anyhow!("Failed to create target machine"))?;
        
        module.set_triple(&triple);
        
        Ok(Self {
            context,
            module,
            target_machine,
        })
    }
    
    /// Get the LLVM context
    pub fn context(&self) -> &Context {
        &self.context
    }
    
    /// Get the LLVM module
    pub fn module(&self) -> &LlvmModule {
        &self.module
    }
    
    /// Get the target machine
    pub fn target_machine(&self) -> &TargetMachine {
        &self.target_machine
    }
    
    /// Emit object file
    pub fn emit_object(&self, path: &Path) -> anyhow::Result<()> {
        self.target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| anyhow::anyhow!("Failed to emit object file: {}", e))
    }
    
    /// Emit assembly file
    pub fn emit_assembly(&self, path: &Path) -> anyhow::Result<()> {
        self.target_machine
            .write_to_file(&self.module, FileType::Assembly, path)
            .map_err(|e| anyhow::anyhow!("Failed to emit assembly file: {}", e))
    }
    
    /// Emit LLVM IR bitcode
    pub fn emit_bitcode(&self, path: &Path) -> anyhow::Result<()> {
        self.module.write_bitcode_to_path(path);
        Ok(())
    }
    
    /// Emit LLVM IR text
    pub fn emit_ir(&self, path: &Path) -> anyhow::Result<()> {
        use std::fs::File;
        use std::io::Write;
        
        let ir = self.module.print_to_string().to_string();
        let mut file = File::create(path)?;
        file.write_all(ir.as_bytes())?;
        Ok(())
    }
    
    /// Verify the module
    pub fn verify(&self) -> anyhow::Result<()> {
        self.module.verify().map_err(|e| anyhow::anyhow!("Module verification failed: {}", e))
    }
    
    /// Set optimization level
    pub fn set_optimization_level(&self, level: OptimizationLevel) {
        // Note: This would require recreating the target machine
        // For now, we set it at creation time
    }
}