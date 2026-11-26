//! Type lowering from nya HIR to LLVM types

use inkwell::types::{BasicType, BasicTypeEnum, IntType, FloatType, PointerType, StructType, ArrayType, VectorType, FunctionType, AnyTypeEnum};
use inkwell::context::Context;
use nya_middle::hir::{HirType, IntTy, FloatTy, Mutability, HirFnSig, HirPath};

/// Type lowering context
pub struct TypeLowerer<'ctx> {
    context: &'ctx Context,
    /// Cache for named struct types
    struct_cache: std::collections::HashMap<String, StructType<'ctx>>,
}

impl<'ctx> TypeLowerer<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            struct_cache: std::collections::HashMap::new(),
        }
    }
    
    /// Lower a HIR type to LLVM type
    pub fn lower_type(&mut self, ty: &HirType) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        match ty {
            HirType::Bool => Ok(self.context.bool_type().as_basic_type_enum()),
            HirType::Int(int_ty) => Ok(self.lower_int_type(*int_ty).as_basic_type_enum()),
            HirType::Float(float_ty) => Ok(self.lower_float_type(*float_ty).as_basic_type_enum()),
            HirType::Char => Ok(self.context.i32_type().as_basic_type_enum()), // Unicode code point
            HirType::Str => Ok(self.lower_str_type().as_basic_type_enum()),
            HirType::Never => Ok(self.context.i8_type().as_basic_type_enum()), // Never returns
            HirType::Path(path) => self.lower_path_type(path),
            HirType::Tuple(types) => self.lower_tuple_type(types),
            HirType::Array(elem_ty, size) => self.lower_array_type(elem_ty, *size),
            HirType::Slice(elem_ty) => self.lower_slice_type(elem_ty),
            HirType::Ref(elem_ty, mutability) => self.lower_ref_type(elem_ty, *mutability),
            HirType::RawPtr(elem_ty, mutability) => self.lower_raw_ptr_type(elem_ty, *mutability),
            HirType::FnPtr(sig) => self.lower_fn_ptr_type(sig),
            HirType::Generic(name) => {
                // Type parameter - would be resolved during monomorphization
                Err(anyhow::anyhow!("Generic type parameter '{}' not resolved", name))
            }
            HirType::Infer => Err(anyhow::anyhow!("Type inference variable not resolved")),
        }
    }
    
    /// Lower integer type
    fn lower_int_type(&self, int_ty: IntTy) -> IntType<'ctx> {
        match int_ty {
            IntTy::I8 => self.context.i8_type(),
            IntTy::I16 => self.context.i16_type(),
            IntTy::I32 => self.context.i32_type(),
            IntTy::I64 => self.context.i64_type(),
            IntTy::I128 => self.context.i128_type(),
            IntTy::Isize => self.context.ptr_sized_int_type(&self.context.get_pointer_size()), 
            IntTy::U8 => self.context.i8_type(),
            IntTy::U16 => self.context.i16_type(),
            IntTy::U32 => self.context.i32_type(),
            IntTy::U64 => self.context.i64_type(),
            IntTy::U128 => self.context.i128_type(),
            IntTy::Usize => self.context.ptr_sized_int_type(&self.context.get_pointer_size()),
        }
    }
    
    /// Lower float type
    fn lower_float_type(&self, float_ty: FloatTy) -> FloatType<'ctx> {
        match float_ty {
            FloatTy::F32 => self.context.f32_type(),
            FloatTy::F64 => self.context.f64_type(),
        }
    }
    
    /// Lower string type (fat pointer: ptr + len)
    fn lower_str_type(&self) -> StructType<'ctx> {
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let len_type = self.context.ptr_sized_int_type(&self.context.get_pointer_size(), false);
        self.context.struct_type(&[ptr_type.into(), len_type.into()], false)
    }
    
    /// Lower path type (named struct/enum)
    fn lower_path_type(&mut self, path: &HirPath) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        let name = self.path_to_name(path);
        
        // Check cache first
        if let Some(struct_ty) = self.struct_cache.get(&name) {
            return Ok(struct_ty.as_basic_type_enum());
        }
        
        // Create opaque struct type for now
        // In a real implementation, this would be populated with actual fields
        let struct_ty = self.context.opaque_struct_type(&name);
        self.struct_cache.insert(name.clone(), struct_ty);
        
        Ok(struct_ty.as_basic_type_enum())
    }
    
    /// Convert path to name
    fn path_to_name(&self, path: &HirPath) -> String {
        path.segments.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join("::")
    }
    
    /// Lower tuple type
    fn lower_tuple_type(&mut self, types: &[HirType]) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        if types.is_empty() {
            // Unit type - use empty struct
            let unit_ty = self.context.struct_type(&[], false);
            return Ok(unit_ty.as_basic_type_enum());
        }
        
        let elem_types: Vec<BasicTypeEnum<'ctx>> = types.iter()
            .map(|t| self.lower_type(t))
            .collect::<Result<Vec<_>, _>>()?;
        
        let struct_ty = self.context.struct_type(&elem_types, false);
        Ok(struct_ty.as_basic_type_enum())
    }
    
    /// Lower array type
    fn lower_array_type(&mut self, elem_ty: &HirType, size: Option<usize>) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        let elem_llvm = self.lower_type(elem_ty)?;
        
        match size {
            Some(n) => {
                // Fixed-size array
                let array_ty = match elem_llvm {
                    BasicTypeEnum::IntType(t) => t.array_type(n as u32).as_basic_type_enum(),
                    BasicTypeEnum::FloatType(t) => t.array_type(n as u32).as_basic_type_enum(),
                    BasicTypeEnum::PointerType(t) => t.array_type(n as u32).as_basic_type_enum(),
                    BasicTypeEnum::StructType(t) => t.array_type(n as u32).as_basic_type_enum(),
                    BasicTypeEnum::ArrayType(t) => t.array_type(n as u32).as_basic_type_enum(),
                    BasicTypeEnum::VectorType(t) => t.array_type(n as u32).as_basic_type_enum(),
                };
                Ok(array_ty)
            }
            None => {
                // Slice (fat pointer)
                Ok(self.lower_slice_type(elem_ty)?)
            }
        }
    }
    
    /// Lower slice type (fat pointer: ptr + len)
    fn lower_slice_type(&mut self, elem_ty: &HirType) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        let elem_llvm = self.lower_type(elem_ty)?;
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let len_type = self.context.ptr_sized_int_type(&self.context.get_pointer_size(), false);
        
        // Create struct { ptr, len }
        let slice_ty = self.context.struct_type(&[ptr_type.into(), len_type.into()], false);
        Ok(slice_ty.as_basic_type_enum())
    }
    
    /// Lower reference type
    fn lower_ref_type(&mut self, elem_ty: &HirType, mutability: Mutability) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        let elem_llvm = self.lower_type(elem_ty)?;
        let ptr_type = match elem_llvm {
            BasicTypeEnum::IntType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::FloatType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::PointerType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::StructType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::ArrayType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::VectorType(t) => t.ptr_type(inkwell::AddressSpace::default()),
        };
        
        // For mutable references, we could add metadata
        // For now, both are just pointers
        Ok(ptr_type.as_basic_type_enum())
    }
    
    /// Lower raw pointer type
    fn lower_raw_ptr_type(&mut self, elem_ty: &HirType, _mutability: Mutability) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        let elem_llvm = self.lower_type(elem_ty)?;
        let ptr_type = match elem_llvm {
            BasicTypeEnum::IntType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::FloatType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::PointerType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::StructType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::ArrayType(t) => t.ptr_type(inkwell::AddressSpace::default()),
            BasicTypeEnum::VectorType(t) => t.ptr_type(inkwell::AddressSpace::default()),
        };
        
        Ok(ptr_type.as_basic_type_enum())
    }
    
    /// Lower function pointer type
    fn lower_fn_ptr_type(&mut self, sig: &HirFnSig) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        let param_types: Vec<BasicTypeEnum<'ctx>> = sig.params.iter()
            .map(|t| self.lower_type(t))
            .collect::<Result<Vec<_>, _>>()?;
        
        let return_type = self.lower_type(&sig.return_type)?;
        
        let fn_type = match return_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, sig.is_variadic),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, sig.is_variadic),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, sig.is_variadic),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, sig.is_variadic),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, sig.is_variadic),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, sig.is_variadic),
        };
        
        let ptr_type = fn_type.ptr_type(inkwell::AddressSpace::default());
        Ok(ptr_type.as_basic_type_enum())
    }
    
    /// Get or create a named struct type
    pub fn get_or_create_struct(&mut self, name: &str, fields: &[HirType], is_packed: bool) -> anyhow::Result<StructType<'ctx>> {
        if let Some(existing) = self.struct_cache.get(name) {
            return Ok(*existing);
        }
        
        let field_types: Vec<BasicTypeEnum<'ctx>> = fields.iter()
            .map(|t| self.lower_type(t))
            .collect::<Result<Vec<_>, _>>()?;
        
        let struct_ty = self.context.struct_type(&field_types, is_packed);
        self.struct_cache.insert(name.to_string(), struct_ty);
        
        Ok(struct_ty)
    }
    
    /// Declare a struct type (for recursive types)
    pub fn declare_struct(&mut self, name: &str) -> StructType<'ctx> {
        let struct_ty = self.context.opaque_struct_type(name);
        self.struct_cache.insert(name.to_string(), struct_ty);
        struct_ty
    }
    
    /// Define a previously declared struct type
    pub fn define_struct(&mut self, name: &str, fields: &[HirType], is_packed: bool) -> anyhow::Result<()> {
        let struct_ty = self.struct_cache.get(name)
            .ok_or_else(|| anyhow::anyhow!("Struct '{}' not declared", name))?;
        
        let field_types: Vec<BasicTypeEnum<'ctx>> = fields.iter()
            .map(|t| self.lower_type(t))
            .collect::<Result<Vec<_>, _>>()?;
        
        struct_ty.set_body(&field_types, is_packed);
        Ok(())
    }
}