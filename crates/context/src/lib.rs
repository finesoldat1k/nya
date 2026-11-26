//! LLVM context management for nya

use inkwell::context::Context;
use inkwell::module::Module as LlvmModule;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, CodeModel, RelocMode, FileType, TargetTriple};
use inkwell::OptimizationLevel;
use std::path::Path;

/// Wrapper around LLVM module and target machine
pub struct CodegenContext {
    module: LlvmModule<'static>,
    target_machine: TargetMachine,
}

impl CodegenContext {
    /// Create a new codegen context for the target triple
    pub fn new(target_triple: Option<&str>) -> anyhow::Result<Self> {
        Target::initialize_all(&InitializationConfig::default());
        
        // Create a context that we leak to get 'static lifetime
        let context = Box::leak(Box::new(Context::create()));
        let module = context.create_module("nya_module");
        
        let triple = target_triple
            .map(|s| s.to_string())
            .unwrap_or_else(|| TargetMachine::get_default_triple().to_string());
        
        let target_triple = TargetTriple::create(&triple);
        let target = Target::from_triple(&target_triple)?;
        
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| anyhow::anyhow!("Failed to create target machine"))?;
        
        module.set_triple(&target_triple);
        
        Ok(Self {
            module,
            target_machine,
        })
    }
    
    pub fn module(&self) -> &LlvmModule<'static> {
        &self.module
    }
    
    pub fn target_machine(&self) -> &TargetMachine {
        &self.target_machine
    }
    
    pub fn emit_object(&self, path: &Path) -> anyhow::Result<()> {
        self.target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| anyhow::anyhow!("Failed to emit object file: {}", e))
    }
    
    pub fn emit_assembly(&self, path: &Path) -> anyhow::Result<()> {
        self.target_machine
            .write_to_file(&self.module, FileType::Assembly, path)
            .map_err(|e| anyhow::anyhow!("Failed to emit assembly file: {}", e))
    }
    
    pub fn emit_bitcode(&self, path: &Path) -> anyhow::Result<()> {
        self.module.write_bitcode_to_path(path);
        Ok(())
    }
    
    pub fn emit_ir(&self, path: &Path) -> anyhow::Result<()> {
        use std::fs::File;
        use std::io::Write;
        
        let ir = self.module.print_to_string().to_string();
        let mut file = File::create(path)?;
        file.write_all(ir.as_bytes())?;
        Ok(())
    }
    
    pub fn verify(&self) -> anyhow::Result<()> {
        self.module.verify().map_err(|e| anyhow::anyhow!("Module verification failed: {}", e))
    }
}