//! Type lowering from nya HIR to LLVM types - minimal stub

use inkwell::types::{BasicTypeEnum, IntType, FloatType, PointerType, StructType, AnyTypeEnum, BasicType};
use inkwell::context::Context;
use anyhow::Result;

/// Minimal HIR type for type lowering
#[derive(Debug, Clone)]
pub enum HirType {
    Bool,
    Int,
    Float,
    Char,
    Str,
    Never,
    Path(String),
    Tuple(Vec<HirType>),
    Array(Box<HirType>, Option<usize>),
    Slice(Box<HirType>),
    Ref(Box<HirType>),
    RawPtr(Box<HirType>),
    FnPtr,
    Generic(String),
    Infer,
}

/// Type lowering context
pub struct TypeLowerer<'ctx> {
    context: &'ctx Context,
    struct_cache: std::collections::HashMap<String, StructType<'ctx>>,
}

impl<'ctx> TypeLowerer<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            struct_cache: std::collections::HashMap::new(),
        }
    }
    
    pub fn lower_type(&mut self, _ty: &HirType) -> Result<BasicTypeEnum<'ctx>> {
        // Stub - just return i32 for everything
        Ok(self.context.i32_type().as_basic_type_enum())
    }
    
    pub fn get_or_create_struct(&mut self, name: &str, _fields: &[HirType], _is_packed: bool) -> Result<StructType<'ctx>> {
        if let Some(existing) = self.struct_cache.get(name) {
            return Ok(*existing);
        }
        
        let struct_ty = self.context.opaque_struct_type(name);
        self.struct_cache.insert(name.to_string(), struct_ty);
        Ok(struct_ty)
    }
    
    pub fn declare_struct(&mut self, name: &str) -> StructType<'ctx> {
        let struct_ty = self.context.opaque_struct_type(name);
        self.struct_cache.insert(name.to_string(), struct_ty);
        struct_ty
    }
    
    pub fn define_struct(&mut self, name: &str, _fields: &[HirType], _is_packed: bool) -> Result<()> {
        if let Some(struct_ty) = self.struct_cache.get(name) {
            // In a real implementation, we'd set the body here
            let _ = struct_ty;
        }
        Ok(())
    }
}