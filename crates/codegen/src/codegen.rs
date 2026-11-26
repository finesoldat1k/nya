//! Code generation from HIR to LLVM IR

use crate::context::CodegenContext;
use crate::types::TypeLowerer;
use hir::*;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue, IntValue, FloatValue};
use inkwell::builder::Builder;
use inkwell::module::Module as LlvmModule;
use inkwell::types::{BasicTypeEnum, FunctionType, StructType, IntType, FloatType, PointerType};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use anyhow::Result;

/// Main codegen struct
pub struct Codegen<'ctx> {
    context: CodegenContext,
    builder: Builder<'ctx>,
    type_lowerer: TypeLowerer<'ctx>,
    
    functions: HashMap<String, FunctionValue<'ctx>>,
    globals: HashMap<String, PointerValue<'ctx>>,
    locals: HashMap<String, PointerValue<'ctx>>,
    
    current_function: Option<FunctionValue<'ctx>>,
    current_block: Option<inkwell::basic_block::BasicBlock<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new() -> Result<Self> {
        let context = CodegenContext::new(None)?;
        let builder = context.context().create_builder();
        let type_lowerer = TypeLowerer::new(context.context());
        
        Ok(Self {
            context,
            builder,
            type_lowerer,
            functions: HashMap::new(),
            globals: HashMap::new(),
            locals: HashMap::new(),
            current_function: None,
            current_block: None,
        })
    }
    
    pub fn compile_module(&mut self, module: HirModule) -> Result<()> {
        for item in &module.items {
            self.declare_item(item)?;
        }
        
        for item in &module.items {
            self.define_item(item)?;
        }
        
        self.context.verify()?;
        Ok(())
    }
    
    pub fn emit_object(&self, path: &str) -> Result<()> {
        self.context.emit_object(std::path::Path::new(path))
    }
    
    pub fn emit_executable(&self, path: &str) -> Result<()> {
        self.emit_object(path)
    }
    
    fn declare_item(&mut self, item: &HirItem) -> Result<()> {
        match item {
            HirItem::Function(f) => self.declare_function(f)?,
            HirItem::Struct(s) => self.declare_struct(s)?,
            HirItem::Enum(e) => self.declare_enum(e)?,
            HirItem::Static(s) => self.declare_static(s)?,
            HirItem::ExternBlock(e) => self.declare_extern_block(e)?,
            _ => {}
        }
        Ok(())
    }
    
    fn define_item(&mut self, item: &HirItem) -> Result<()> {
        match item {
            HirItem::Function(f) => self.define_function(f)?,
            HirItem::Struct(s) => self.define_struct(s)?,
            HirItem::Static(s) => self.define_static(s)?,
            _ => {}
        }
        Ok(())
    }
    
    fn declare_function(&mut self, func: &HirFunction) -> Result<()> {
        let fn_type = self.lower_function_type(func)?;
        let linkage = if func.is_extern {
            inkwell::module::Linkage::External
        } else if func.is_pub {
            inkwell::module::Linkage::External
        } else {
            inkwell::module::Linkage::Internal
        };
        
        let fn_value = self.context.module().add_function(&func.name, fn_type, Some(linkage));
        
        if let Some(abi) = &func.abi {
            if abi == "C" {
                fn_value.set_call_conventions(inkwell::CallingConvention::C);
            }
        }
        
        self.functions.insert(func.name.clone(), fn_value);
        Ok(())
    }
    
    fn define_function(&mut self, func: &HirFunction) -> Result<()> {
        let fn_value = self.functions.get(&func.name).copied()
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not declared", func.name))?;
        
        if func.is_extern {
            return Ok(());
        }
        
        let body = func.body.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Function '{}' has no body", func.name))?;
        
        let entry = self.context.context().append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);
        
        self.current_function = Some(fn_value);
        self.current_block = Some(entry);
        
        for (i, param) in func.params.iter().enumerate() {
            let param_val = fn_value.get_nth_param(i as u32)
                .ok_or_else(|| anyhow::anyhow!("Missing parameter {}", i))?;
            
            let param_ty = self.type_lowerer.lower_type(&param.ty)?;
            let alloca = self.builder.build_alloca(param_ty, &param.name)?;
            self.builder.build_store(alloca, param_val)?;
            
            self.locals.insert(param.name.clone(), alloca);
        }
        
        self.compile_block(body)?;
        
        if !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
            if let Some(ret_ty) = &func.return_type {
                if !matches!(ret_ty, HirType::Tuple(types) if types.is_empty()) {
                    let ret_llvm = self.type_lowerer.lower_type(ret_ty)?;
                    let zero = match ret_llvm {
                        BasicTypeEnum::IntType(t) => t.const_zero().as_basic_value_enum(),
                        BasicTypeEnum::FloatType(t) => t.const_zero().as_basic_value_enum(),
                        BasicTypeEnum::PointerType(t) => t.const_zero().as_basic_value_enum(),
                        _ => self.context.context().i8_type().const_zero().as_basic_value_enum(),
                    };
                    self.builder.build_return(Some(&zero))?;
                } else {
                    self.builder.build_return(None)?;
                }
            } else {
                self.builder.build_return(None)?;
            }
        }
        
        self.current_function = None;
        self.current_block = None;
        self.locals.clear();
        
        Ok(())
    }
    
    fn lower_function_type(&mut self, func: &HirFunction) -> Result<FunctionType<'ctx>> {
        let param_types: Vec<BasicTypeEnum<'ctx>> = func.params.iter()
            .map(|p| self.type_lowerer.lower_type(&p.ty))
            .collect::<Result<Vec<_>, _>>()?;
        
        let return_type = match &func.return_type {
            Some(ty) => self.type_lowerer.lower_type(ty)?,
            None => self.context.context().void_type().as_basic_type_enum(),
        };
        
        let fn_type = match return_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VoidType(t) => t.fn_type(&param_types, false),
        };
        
        Ok(fn_type)
    }
    
    fn declare_struct(&mut self, s: &HirStruct) -> Result<()> {
        let name = s.name.clone();
        self.type_lowerer.declare_struct(&name);
        Ok(())
    }
    
    fn define_struct(&mut self, s: &HirStruct) -> Result<()> {
        let name = s.name.clone();
        let field_types: Vec<HirType> = s.fields.iter().map(|f| f.ty.clone()).collect();
        self.type_lowerer.define_struct(&name, &field_types, false)?;
        Ok(())
    }
    
    fn declare_enum(&mut self, e: &HirEnum) -> Result<()> {
        let name = e.name.clone();
        self.type_lowerer.declare_struct(&name);
        Ok(())
    }
    
    fn declare_static(&mut self, s: &HirStatic) -> Result<()> {
        let ty = self.type_lowerer.lower_type(&s.ty)?;
        let global = self.context.module().add_global(ty, None, &s.name);
        global.set_linkage(if s.is_pub {
            inkwell::module::Linkage::External
        } else {
            inkwell::module::Linkage::Internal
        });
        global.set_constant(!s.is_mut);
        self.globals.insert(s.name.clone(), global.as_pointer_value());
        Ok(())
    }
    
    fn define_static(&mut self, s: &HirStatic) -> Result<()> {
        let global = self.globals.get(&s.name).copied()
            .ok_or_else(|| anyhow::anyhow!("Static '{}' not declared", s.name))?;
        
        let init = self.compile_expr(&s.value)?;
        self.builder.build_store(global, init)?;
        
        Ok(())
    }
    
    fn declare_extern_block(&mut self, e: &HirExternBlock) -> Result<()> {
        for item in &e.items {
            match item {
                HirExternItem::Function(f) => self.declare_function(f)?,
                HirExternItem::Static(s) => self.declare_static(s)?,
            }
        }
        Ok(())
    }
    
    fn compile_block(&mut self, block: &HirBlock) -> Result<()> {
        for stmt in &block.stmts {
            self.compile_stmt(stmt)?;
        }
        
        if let Some(expr) = &block.expr {
            self.compile_expr(expr)?;
        }
        
        Ok(())
    }
    
    fn compile_stmt(&mut self, stmt: &HirStmt) -> Result<()> {
        match stmt {
            HirStmt::Let(l) => self.compile_let_stmt(l)?,
            HirStmt::Expr(e) => { self.compile_expr(e)?; },
            HirStmt::Item(_) => {},
            HirStmt::Macro(_) => {},
        }
        Ok(())
    }
    
    fn compile_let_stmt(&mut self, l: &HirLetStmt) -> Result<()> {
        let ty = l.ty.as_ref().map(|t| self.type_lowerer.lower_type(t))
            .transpose()?
            .unwrap_or_else(|| self.context.context().i8_type().as_basic_type_enum());
        
        let alloca = self.builder.build_alloca(ty, &l.name)?;
        
        if let Some(init) = &l.init {
            let value = self.compile_expr(init)?;
            self.builder.build_store(alloca, value)?;
        }
        
        self.locals.insert(l.name.clone(), alloca);
        Ok(())
    }
    
    fn compile_expr(&mut self, expr: &HirExpr) -> Result<BasicValueEnum<'ctx>> {
        match &expr.kind {
            HirExprKind::Bool(v) => Ok(self.context.context().bool_type().const_int(*v as u64, false).as_basic_value_enum()),
            HirExprKind::Integer(v) => Ok(self.context.context().i64_type().const_int(*v, false).as_basic_value_enum()),
            HirExprKind::Float(v) => Ok(self.context.context().f64_type().const_float(*v).as_basic_value_enum()),
            HirExprKind::String(s) => self.compile_string_literal(s),
            HirExprKind::Char(c) => Ok(self.context.context().i32_type().const_int(*c as u32 as u64, false).as_basic_value_enum()),
            HirExprKind::Path(p) => self.compile_path(p),
            HirExprKind::Unary(op, e) => self.compile_unary_op(*op, e),
            HirExprKind::Binary(op, l, r) => self.compile_binary_op(*op, l, r),
            HirExprKind::Call(callee, args) => self.compile_call(callee, args),
            HirExprKind::MethodCall(receiver, method, type_args, args) => self.compile_method_call(receiver, method, type_args, args),
            HirExprKind::Field(e, field) => self.compile_field_access(e, field),
            HirExprKind::Index(e, idx) => self.compile_index(e, idx),
            HirExprKind::Block(b) => self.compile_block_expr(b),
            HirExprKind::If(cond, then_branch, else_branch) => self.compile_if(cond, then_branch, else_branch.as_deref()),
            HirExprKind::Return(value) => self.compile_return(value.as_deref()),
            HirExprKind::Assign(l, r) => self.compile_assign(l, r),
            HirExprKind::Ref(e, m) => self.compile_ref(e, *m),
            HirExprKind::Deref(e) => self.compile_deref(e),
            _ => Err(anyhow::anyhow!("Expression not yet implemented: {:?}", expr.kind)),
        }
    }
    
    fn compile_string_literal(&mut self, s: &str) -> Result<BasicValueEnum<'ctx>> {
        let str_val = self.builder.build_global_string_ptr(s, "str_literal")?;
        
        let str_ty = self.type_lowerer.lower_type(&HirType::Str)?;
        let str_struct = match str_ty {
            BasicTypeEnum::StructType(t) => t,
            _ => return Err(anyhow::anyhow!("Str type not a struct")),
        };
        
        let ptr = str_val.as_pointer_value();
        let len = self.context.context().i64_type().const_int(s.len() as u64, false);
        
        let str_val = str_struct.const_named_struct(&[
            ptr.into(),
            len.into(),
        ]);
        
        Ok(str_val.as_basic_value_enum())
    }
    
    fn compile_path(&mut self, path: &HirPath) -> Result<BasicValueEnum<'ctx>> {
        if path.segments.len() == 1 {
            let name = &path.segments[0].name;
            
            if let Some(alloca) = self.locals.get(name) {
                let ty = self.type_lowerer.lower_type(&HirType::Infer)?;
                return Ok(self.builder.build_load(ty, *alloca, name)?.as_basic_value_enum());
            }
            
            if let Some(global) = self.globals.get(name) {
                let ty = self.type_lowerer.lower_type(&HirType::Infer)?;
                return Ok(self.builder.build_load(ty, *global, name)?.as_basic_value_enum());
            }
            
            if let Some(func) = self.functions.get(name) {
                return Ok(func.as_global_value().as_basic_value_enum());
            }
        }
        
        Err(anyhow::anyhow!("Unknown path: {:?}", path))
    }
    
    fn compile_unary_op(&mut self, op: UnOp, expr: &HirExpr) -> Result<BasicValueEnum<'ctx>> {
        let val = self.compile_expr(expr)?;
        
        match op {
            UnOp::Neg => {
                if val.is_int_value() {
                    Ok(self.builder.build_int_neg(val.into_int_value(), "neg")?.as_basic_value_enum())
                } else if val.is_float_value() {
                    Ok(self.builder.build_float_neg(val.into_float_value(), "neg")?.as_basic_value_enum())
                } else {
                    Err(anyhow::anyhow!("Cannot negate non-numeric type"))
                }
            }
            UnOp::Not => {
                if val.is_int_value() {
                    Ok(self.builder.build_not(val.into_int_value(), "not")?.as_basic_value_enum())
                } else {
                    Err(anyhow::anyhow!("Cannot logical not non-bool type"))
                }
            }
            UnOp::BitNot => {
                if val.is_int_value() {
                    Ok(self.builder.build_not(val.into_int_value(), "bitnot")?.as_basic_value_enum())
                } else {
                    Err(anyhow::anyhow!("Cannot bitwise not non-int type"))
                }
            }
            UnOp::Deref => {
                if val.is_pointer_value() {
                    let ptr = val.into_pointer_value();
                    let pointee_ty = self.context.context().i8_type();
                    Ok(self.builder.build_load(pointee_ty, ptr, "deref")?.as_basic_value_enum())
                } else {
                    Err(anyhow::anyhow!("Cannot deref non-pointer"))
                }
            }
            UnOp::Ref | UnOp::RefMut => {
                Err(anyhow::anyhow!("Ref operation not implemented in this context"))
            }
        }
    }
    
    fn compile_binary_op(&mut self, op: BinOp, lhs: &HirExpr, rhs: &HirExpr) -> Result<BasicValueEnum<'ctx>> {
        let l = self.compile_expr(lhs)?;
        let r = self.compile_expr(rhs)?;
        
        if l.is_int_value() && r.is_int_value() {
            let l = l.into_int_value();
            let r = r.into_int_value();
            
            let result = match op {
                BinOp::Add => self.builder.build_int_add(l, r, "add")?,
                BinOp::Sub => self.builder.build_int_sub(l, r, "sub")?,
                BinOp::Mul => self.builder.build_int_mul(l, r, "mul")?,
                BinOp::Div => self.builder.build_int_signed_div(l, r, "div")?,
                BinOp::Rem => self.builder.build_int_signed_rem(l, r, "rem")?,
                BinOp::BitAnd => self.builder.build_and(l, r, "and")?,
                BinOp::BitOr => self.builder.build_or(l, r, "or")?,
                BinOp::BitXor => self.builder.build_xor(l, r, "xor")?,
                BinOp::Shl => self.builder.build_left_shift(l, r, "shl")?,
                BinOp::Shr => self.builder.build_right_shift(l, r, false, "shr")?,
                BinOp::Eq => self.builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eq")?,
                BinOp::Ne => self.builder.build_int_compare(inkwell::IntPredicate::NE, l, r, "ne")?,
                BinOp::Lt => self.builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "lt")?,
                BinOp::Gt => self.builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, "gt")?,
                BinOp::Le => self.builder.build_int_compare(inkwell::IntPredicate::SLE, l, r, "le")?,
                BinOp::Ge => self.builder.build_int_compare(inkwell::IntPredicate::SGE, l, r, "ge")?,
                _ => return Err(anyhow::anyhow!("Unsupported integer binary op: {:?}", op)),
            };
            
            return Ok(result.as_basic_value_enum());
        }
        
        if l.is_float_value() && r.is_float_value() {
            let l = l.into_float_value();
            let r = r.into_float_value();
            
            let result = match op {
                BinOp::Add => self.builder.build_float_add(l, r, "add")?,
                BinOp::Sub => self.builder.build_float_sub(l, r, "sub")?,
                BinOp::Mul => self.builder.build_float_mul(l, r, "mul")?,
                BinOp::Div => self.builder.build_float_div(l, r, "div")?,
                BinOp::Rem => self.builder.build_float_rem(l, r, "rem")?,
                BinOp::Eq => self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eq")?,
                BinOp::Ne => self.builder.build_float_compare(inkwell::FloatPredicate::ONE, l, r, "ne")?,
                BinOp::Lt => self.builder.build_float_compare(inkwell::FloatPredicate::OLT, l, r, "lt")?,
                BinOp::Gt => self.builder.build_float_compare(inkwell::FloatPredicate::OGT, l, r, "gt")?,
                BinOp::Le => self.builder.build_float_compare(inkwell::FloatPredicate::OLE, l, r, "le")?,
                BinOp::Ge => self.builder.build_float_compare(inkwell::FloatPredicate::OGE, l, r, "ge")?,
                _ => return Err(anyhow::anyhow!("Unsupported float binary op: {:?}", op)),
            };
            
            return Ok(result.as_basic_value_enum());
        }
        
        Err(anyhow::anyhow!("Binary op type mismatch"))
    }
    
    fn compile_call(&mut self, callee: &HirExpr, args: &[HirExpr]) -> Result<BasicValueEnum<'ctx>> {
        let callee_val = self.compile_expr(callee)?;
        
        let arg_vals: Vec<BasicValueEnum<'ctx>> = args.iter()
            .map(|a| self.compile_expr(a))
            .collect::<Result<Vec<_>, _>>()?;
        
        if callee_val.is_pointer_value() {
            let fn_ptr = callee_val.into_pointer_value();
            let call = self.builder.build_indirect_call(
                self.context.context().void_type().fn_type(&[], false),
                fn_ptr,
                &arg_vals,
                "call"
            )?;
            
            if let Some(result) = call.try_as_basic_value().left() {
                Ok(result)
            } else {
                Ok(self.context.context().i8_type().const_zero().as_basic_value_enum())
            }
        } else {
            Err(anyhow::anyhow!("Callee is not a function pointer"))
        }
    }
    
    fn compile_method_call(&mut self, receiver: &HirExpr, method: &str, _type_args: &[HirType], args: &[HirExpr]) -> Result<BasicValueEnum<'ctx>> {
        let receiver_val = self.compile_expr(receiver)?;
        let arg_vals: Vec<BasicValueEnum<'ctx>> = args.iter()
            .map(|a| self.compile_expr(a))
            .collect::<Result<Vec<_>, _>>()?;
        
        Err(anyhow::anyhow!("Method call not fully implemented"))
    }
    
    fn compile_field_access(&mut self, expr: &HirExpr, field: &str) -> Result<BasicValueEnum<'ctx>> {
        let val = self.compile_expr(expr)?;
        
        if val.is_pointer_value() {
            let ptr = val.into_pointer_value();
            let field_idx = 0;
            let gep = self.builder.build_struct_gep(self.context.context().i8_type(), ptr, field_idx, field)?;
            let ty = self.context.context().i8_type();
            Ok(self.builder.build_load(ty, gep, field)?.as_basic_value_enum())
        } else {
            Err(anyhow::anyhow!("Field access on non-pointer"))
        }
    }
    
    fn compile_index(&mut self, expr: &HirExpr, idx: &HirExpr) -> Result<BasicValueEnum<'ctx>> {
        let val = self.compile_expr(expr)?;
        let idx_val = self.compile_expr(idx)?;
        
        if val.is_pointer_value() && idx_val.is_int_value() {
            let ptr = val.into_pointer_value();
            let idx = idx_val.into_int_value();
            let gep = self.builder.build_gep(self.context.context().i8_type(), ptr, &[idx], "index")?;
            let ty = self.context.context().i8_type();
            Ok(self.builder.build_load(ty, gep, "index")?.as_basic_value_enum())
        } else {
            Err(anyhow::anyhow!("Index on non-array/pointer"))
        }
    }
    
    fn compile_block_expr(&mut self, block: &HirBlock) -> Result<BasicValueEnum<'ctx>> {
        self.compile_block(block)?;
        Ok(self.context.context().i8_type().const_zero().as_basic_value_enum())
    }
    
    fn compile_if(&mut self, cond: &HirExpr, then_branch: &HirBlock, else_branch: Option<&HirExpr>) -> Result<BasicValueEnum<'ctx>> {
        let cond_val = self.compile_expr(cond)?;
        let cond_int = cond_val.into_int_value();
        
        let current_fn = self.current_function.unwrap();
        let then_bb = self.context.context().append_basic_block(current_fn, "then");
        let else_bb = self.context.context().append_basic_block(current_fn, "else");
        let merge_bb = self.context.context().append_basic_block(current_fn, "merge");
        
        self.builder.build_conditional_branch(cond_int, then_bb, else_bb)?;
        
        self.builder.position_at_end(then_bb);
        self.compile_block(then_branch)?;
        self.builder.build_unconditional_branch(merge_bb)?;
        
        self.builder.position_at_end(else_bb);
        if let Some(else_expr) = else_branch {
            self.compile_expr(else_expr)?;
        }
        self.builder.build_unconditional_branch(merge_bb)?;
        
        self.builder.position_at_end(merge_bb);
        
        Ok(self.context.context().i8_type().const_zero().as_basic_value_enum())
    }
    
    fn compile_return(&mut self, value: Option<&HirExpr>) -> Result<BasicValueEnum<'ctx>> {
        if let Some(v) = value {
            let val = self.compile_expr(v)?;
            self.builder.build_return(Some(&val))?;
        } else {
            self.builder.build_return(None)?;
        }
        Ok(self.context.context().i8_type().const_zero().as_basic_value_enum())
    }
    
    fn compile_assign(&mut self, lhs: &HirExpr, rhs: &HirExpr) -> Result<BasicValueEnum<'ctx>> {
        let rhs_val = self.compile_expr(rhs)?;
        
        if let HirExprKind::Path(path) = &lhs.kind {
            if path.segments.len() == 1 {
                let name = &path.segments[0].name;
                if let Some(alloca) = self.locals.get(name) {
                    self.builder.build_store(*alloca, rhs_val)?;
                    return Ok(rhs_val);
                }
            }
        }
        
        Err(anyhow::anyhow!("Invalid assignment target"))
    }
    
    fn compile_ref(&mut self, expr: &HirExpr, _mutability: Mutability) -> Result<BasicValueEnum<'ctx>> {
        if let HirExprKind::Path(path) = &expr.kind {
            if path.segments.len() == 1 {
                let name = &path.segments[0].name;
                if let Some(alloca) = self.locals.get(name) {
                    return Ok(alloca.as_basic_value_enum());
                }
            }
        }
        
        Err(anyhow::anyhow!("Cannot take reference of non-lvalue"))
    }
    
    fn compile_deref(&mut self, expr: &HirExpr) -> Result<BasicValueEnum<'ctx>> {
        let val = self.compile_expr(expr)?;
        
        if val.is_pointer_value() {
            let ptr = val.into_pointer_value();
            let ty = self.context.context().i8_type();
            Ok(self.builder.build_load(ty, ptr, "deref")?.as_basic_value_enum())
        } else {
            Err(anyhow::anyhow!("Cannot deref non-pointer"))
        }
    }
}