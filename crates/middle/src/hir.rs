//! High-level Intermediate Representation (HIR) for nya
//!
//! The HIR is a simplified, desugared version of the AST that's ready for code generation.

use nya_frontend::diagnostics::Span;
use indexmap::IndexMap;
use smallvec::SmallVec;
use std::fmt;

/// HIR Module
#[derive(Debug, Clone)]
pub struct HirModule {
    pub name: String,
    pub span: Span,
    pub items: Vec<HirItem>,
    pub imports: Vec<HirImport>,
}

/// HIR Import
#[derive(Debug, Clone)]
pub struct HirImport {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub items: Vec<HirImportItem>,
}

/// HIR Import Item
#[derive(Debug, Clone)]
pub struct HirImportItem {
    pub name: String,
    pub alias: Option<String>,
}

/// HIR Item kinds
#[derive(Debug, Clone)]
pub enum HirItem {
    Function(HirFunction),
    Struct(HirStruct),
    Enum(HirEnum),
    Impl(HirImpl),
    TypeAlias(HirTypeAlias),
    Const(HirConst),
    Static(HirStatic),
    ExternBlock(HirExternBlock),
    Module(HirModule),
}

/// HIR Function
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub generics: Vec<HirGenericParam>,
    pub params: Vec<HirParam>,
    pub return_type: Option<HirType>,
    pub body: Option<HirBlock>,
    pub is_pub: bool,
    pub is_static: bool,
    pub is_extern: bool,
    pub abi: Option<String>,
    pub attributes: Vec<HirAttribute>,
}

/// HIR Generic Parameter
#[derive(Debug, Clone)]
pub struct HirGenericParam {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub bounds: Vec<HirType>,
    pub default: Option<HirType>,
}

/// HIR Parameter
#[derive(Debug, Clone)]
pub struct HirParam {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: HirType,
    pub is_mut: bool,
    pub is_ref: bool,
}

/// HIR Types
#[derive(Debug, Clone, PartialEq)]
pub enum HirType {
    Bool,
    Int(IntTy),
    Float(FloatTy),
    Char,
    Str,
    Never,
    Path(HirPath),
    Tuple(Vec<HirType>),
    Array(Box<HirType>, Option<usize>),
    Slice(Box<HirType>),
    Ref(Box<HirType>, Mutability),
    RawPtr(Box<HirType>, Mutability),
    FnPtr(HirFnSig),
    Generic(String),
    Infer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntTy {
    I8, I16, I32, I64, I128, Isize,
    U8, U16, U32, U64, U128, Usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatTy {
    F32, F64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

/// HIR Function Signature
#[derive(Debug, Clone)]
pub struct HirFnSig {
    pub params: Vec<HirType>,
    pub return_type: Box<HirType>,
    pub is_variadic: bool,
    pub abi: Option<String>,
}

/// HIR Path
#[derive(Debug, Clone, PartialEq)]
pub struct HirPath {
    pub segments: Vec<HirPathSegment>,
    pub is_absolute: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirPathSegment {
    pub name: String,
    pub type_args: Vec<HirType>,
}

/// HIR Block
#[derive(Debug, Clone)]
pub struct HirBlock {
    pub id: usize,
    pub span: Span,
    pub stmts: Vec<HirStmt>,
    pub expr: Option<Box<HirExpr>>,
}

/// HIR Statements
#[derive(Debug, Clone)]
pub enum HirStmt {
    Let(HirLetStmt),
    Expr(HirExpr),
    Item(HirItem),
    Macro(HirMacroCall),
}

/// HIR Let Statement
#[derive(Debug, Clone)]
pub struct HirLetStmt {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: Option<HirType>,
    pub init: Option<HirExpr>,
    pub is_mut: bool,
    pub is_ref: bool,
}

/// HIR Expressions
#[derive(Debug, Clone)]
pub struct HirExpr {
    pub id: usize,
    pub span: Span,
    pub kind: HirExprKind,
    pub ty: Option<HirType>,
}

#[derive(Debug, Clone)]
pub enum HirExprKind {
    Bool(bool),
    Integer(u64),
    Float(f64),
    String(String),
    Char(char),
    Path(HirPath),
    Unary(UnOp, Box<HirExpr>),
    Binary(BinOp, Box<HirExpr>, Box<HirExpr>),
    Call(Box<HirExpr>, Vec<HirExpr>),
    MethodCall(Box<HirExpr>, String, Vec<HirType>, Vec<HirExpr>),
    Field(Box<HirExpr>, String),
    Index(Box<HirExpr>, Box<HirExpr>),
    Range(Box<HirExpr>, Box<HirExpr>, RangeEnd),
    Struct(HirPath, Vec<HirFieldExpr>),
    Array(Vec<HirExpr>),
    Tuple(Vec<HirExpr>),
    Block(HirBlock),
    If(Box<HirExpr>, HirBlock, Option<Box<HirExpr>>),
    Match(Box<HirExpr>, Vec<HirArm>),
    Loop(HirBlock, Option<String>),
    While(Box<HirExpr>, HirBlock, Option<String>),
    For(String, Box<HirExpr>, HirBlock, Option<String>),
    Break(Option<String>, Option<Box<HirExpr>>),
    Continue(Option<String>),
    Return(Option<Box<HirExpr>>),
    Try(Box<HirExpr>),
    Closure(HirClosure),
    Cast(Box<HirExpr>, HirType),
    Assign(Box<HirExpr>, Box<HirExpr>),
    AssignOp(BinOp, Box<HirExpr>, Box<HirExpr>),
    Ref(Box<HirExpr>, Mutability),
    Deref(Box<HirExpr>),
    Group(Box<HirExpr>),
}

#[derive(Debug, Clone)]
pub struct HirFieldExpr {
    pub name: String,
    pub expr: HirExpr,
}

#[derive(Debug, Clone)]
pub struct HirArm {
    pub id: usize,
    pub span: Span,
    pub pattern: HirPattern,
    pub guard: Option<HirExpr>,
    pub body: HirExpr,
}

#[derive(Debug, Clone)]
pub enum HirPattern {
    Wildcard,
    Ident(String),
    Literal(HirExprKind),
    Path(HirPath),
    Tuple(Vec<HirPattern>),
    Struct(HirPath, Vec<HirFieldPattern>),
    Or(Vec<HirPattern>),
    Rest,
    Ref(Box<HirPattern>, Mutability),
}

#[derive(Debug, Clone)]
pub struct HirFieldPattern {
    pub name: String,
    pub pattern: HirPattern,
}

#[derive(Debug, Clone)]
pub struct HirClosure {
    pub params: Vec<HirParam>,
    pub return_type: Option<HirType>,
    pub body: HirBlock,
    pub is_move: bool,
}

/// HIR Unary Operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
    Deref,
    Ref,
    RefMut,
}

/// HIR Binary Operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
    BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign,
}

/// HIR Range End
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeEnd {
    Included,
    Excluded,
}

/// HIR Struct
#[derive(Debug, Clone)]
pub struct HirStruct {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub generics: Vec<HirGenericParam>,
    pub fields: Vec<HirStructField>,
    pub is_pub: bool,
    pub attributes: Vec<HirAttribute>,
}

#[derive(Debug, Clone)]
pub struct HirStructField {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: HirType,
    pub is_pub: bool,
    pub attributes: Vec<HirAttribute>,
}

/// HIR Enum
#[derive(Debug, Clone)]
pub struct HirEnum {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub generics: Vec<HirGenericParam>,
    pub variants: Vec<HirEnumVariant>,
    pub is_pub: bool,
    pub attributes: Vec<HirAttribute>,
}

#[derive(Debug, Clone)]
pub struct HirEnumVariant {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub data: HirEnumVariantData,
    pub discriminant: Option<HirExpr>,
    pub attributes: Vec<HirAttribute>,
}

#[derive(Debug, Clone)]
pub enum HirEnumVariantData {
    Unit,
    Tuple(Vec<HirType>),
    Struct(Vec<HirStructField>),
}

/// HIR Impl
#[derive(Debug, Clone)]
pub struct HirImpl {
    pub id: usize,
    pub span: Span,
    pub generics: Vec<HirGenericParam>,
    pub trait_ref: Option<HirPath>,
    pub self_ty: HirType,
    pub items: Vec<HirImplItem>,
    pub attributes: Vec<HirAttribute>,
}

#[derive(Debug, Clone)]
pub enum HirImplItem {
    Function(HirFunction),
    Const(HirConst),
    TypeAlias(HirTypeAlias),
}

/// HIR Type Alias
#[derive(Debug, Clone)]
pub struct HirTypeAlias {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub generics: Vec<HirGenericParam>,
    pub ty: HirType,
    pub is_pub: bool,
    pub attributes: Vec<HirAttribute>,
}

/// HIR Const
#[derive(Debug, Clone)]
pub struct HirConst {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: HirType,
    pub value: HirExpr,
    pub is_pub: bool,
    pub attributes: Vec<HirAttribute>,
}

/// HIR Static
#[derive(Debug, Clone)]
pub struct HirStatic {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: HirType,
    pub value: HirExpr,
    pub is_pub: bool,
    pub is_mut: bool,
    pub attributes: Vec<HirAttribute>,
}

/// HIR Extern Block
#[derive(Debug, Clone)]
pub struct HirExternBlock {
    pub id: usize,
    pub span: Span,
    pub abi: String,
    pub items: Vec<HirExternItem>,
}

#[derive(Debug, Clone)]
pub enum HirExternItem {
    Function(HirFunction),
    Static(HirStatic),
}

/// HIR Attribute
#[derive(Debug, Clone)]
pub struct HirAttribute {
    pub id: usize,
    pub span: Span,
    pub path: HirPath,
    pub args: Vec<HirAttributeArg>,
}

#[derive(Debug, Clone)]
pub enum HirAttributeArg {
    Ident(String),
    Literal(HirExprKind),
    NameValue(String, HirExpr),
}

/// HIR Macro Call
#[derive(Debug, Clone)]
pub struct HirMacroCall {
    pub id: usize,
    pub span: Span,
    pub path: HirPath,
    pub args: Vec<HirMacroArg>,
}

#[derive(Debug, Clone)]
pub enum HirMacroArg {
    Expr(HirExpr),
    Stmt(HirStmt),
    Pat(HirPattern),
    Ty(HirType),
    Block(HirBlock),
}

/// Lower from checked AST to HIR
pub fn lower(module: CheckedModule) -> anyhow::Result<HirModule> {
    let mut lowerer = Lowerer::new();
    lowerer.lower_module(module)
}

struct Lowerer {
    next_id: usize,
}

impl Lowerer {
    fn new() -> Self {
        Self { next_id: 0 }
    }
    
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    fn lower_module(&mut self, module: CheckedModule) -> anyhow::Result<HirModule> {
        let mut items = Vec::new();
        for item in module.items {
            items.push(self.lower_item(item)?);
        }
        
        Ok(HirModule {
            name: module.name,
            span: module.span,
            items,
            imports: module.imports.into_iter().map(|i| self.lower_import(i)).collect(),
        })
    }
    
    fn lower_import(&mut self, import: ResolvedImport) -> HirImport {
        HirImport {
            path: import.path,
            alias: import.alias,
            items: import.items.into_iter().map(|item| HirImportItem {
                name: item.name,
                alias: item.alias,
            }).collect(),
        }
    }
    
    fn lower_item(&mut self, item: CheckedItem) -> anyhow::Result<HirItem> {
        Ok(match item {
            CheckedItem::Function(f) => HirItem::Function(self.lower_function(f)?),
            CheckedItem::Struct(s) => HirItem::Struct(self.lower_struct(s)?),
            CheckedItem::Enum(e) => HirItem::Enum(self.lower_enum(e)?),
            CheckedItem::Impl(i) => HirItem::Impl(self.lower_impl(i)?),
            CheckedItem::TypeAlias(t) => HirItem::TypeAlias(self.lower_type_alias(t)?),
            CheckedItem::Const(c) => HirItem::Const(self.lower_const(c)?),
            CheckedItem::Static(s) => HirItem::Static(self.lower_static(s)?),
            CheckedItem::ExternBlock(e) => HirItem::ExternBlock(self.lower_extern_block(e)?),
            CheckedItem::Module(m) => HirItem::Module(self.lower_module(m)?),
        })
    }
    
    fn lower_function(&mut self, func: CheckedFunction) -> anyhow::Result<HirFunction> {
        Ok(HirFunction {
            id: self.next_id(),
            span: func.span,
            name: func.name,
            generics: func.generics.into_iter().map(|g| self.lower_generic_param(g)).collect(),
            params: func.params.into_iter().map(|p| self.lower_param(p)).collect(),
            return_type: func.return_type.map(|t| self.lower_type(t)),
            body: func.body.map(|b| self.lower_block(b)),
            is_pub: func.is_pub,
            is_static: func.is_static,
            is_extern: func.is_extern,
            abi: func.abi,
            attributes: func.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
        })
    }
    
    fn lower_generic_param(&mut self, g: ResolvedGenericParam) -> HirGenericParam {
        HirGenericParam {
            id: self.next_id(),
            span: g.span,
            name: g.name,
            bounds: g.bounds.into_iter().map(|t| self.lower_type(t)).collect(),
            default: g.default.map(|t| self.lower_type(t)),
        }
    }
    
    fn lower_param(&mut self, p: ResolvedParam) -> HirParam {
        HirParam {
            id: self.next_id(),
            span: p.span,
            name: p.name,
            ty: self.lower_type(p.ty),
            is_mut: p.is_mut,
            is_ref: p.is_ref,
        }
    }
    
    fn lower_type(&mut self, ty: ResolvedType) -> HirType {
        match ty {
            ResolvedType::Bool => HirType::Bool,
            ResolvedType::Int(t) => HirType::Int(t),
            ResolvedType::Float(t) => HirType::Float(t),
            ResolvedType::Char => HirType::Char,
            ResolvedType::Str => HirType::Str,
            ResolvedType::Never => HirType::Never,
            ResolvedType::Path(p) => HirType::Path(self.lower_path(p)),
            ResolvedType::Tuple(types) => HirType::Tuple(types.into_iter().map(|t| self.lower_type(t)).collect()),
            ResolvedType::Array(t, size) => HirType::Array(Box::new(self.lower_type(*t)), size),
            ResolvedType::Slice(t) => HirType::Slice(Box::new(self.lower_type(*t))),
            ResolvedType::Ref(t, m) => HirType::Ref(Box::new(self.lower_type(*t)), m),
            ResolvedType::RawPtr(t, m) => HirType::RawPtr(Box::new(self.lower_type(*t)), m),
            ResolvedType::FnPtr(sig) => HirType::FnPtr(HirFnSig {
                params: sig.params.into_iter().map(|t| self.lower_type(t)).collect(),
                return_type: Box::new(self.lower_type(*sig.return_type)),
                is_variadic: sig.is_variadic,
                abi: sig.abi,
            }),
            ResolvedType::Generic(name) => HirType::Generic(name),
            ResolvedType::Infer => HirType::Infer,
        }
    }
    
    fn lower_path(&mut self, path: ResolvedPath) -> HirPath {
        HirPath {
            segments: path.segments.into_iter().map(|seg| HirPathSegment {
                name: seg.name,
                type_args: seg.type_args.into_iter().map(|t| self.lower_type(t)).collect(),
            }).collect(),
            is_absolute: path.is_absolute,
        }
    }
    
    fn lower_block(&mut self, block: CheckedBlock) -> HirBlock {
        HirBlock {
            id: self.next_id(),
            span: block.span,
            stmts: block.stmts.into_iter().map(|s| self.lower_stmt(s)).collect(),
            expr: block.expr.map(|e| Box::new(self.lower_expr(*e))),
        }
    }
    
    fn lower_stmt(&mut self, stmt: CheckedStmt) -> HirStmt {
        match stmt {
            CheckedStmt::Let(l) => HirStmt::Let(self.lower_let_stmt(l)),
            CheckedStmt::Expr(e) => HirStmt::Expr(self.lower_expr(e)),
            CheckedStmt::Item(i) => HirStmt::Item(self.lower_item(i).unwrap()),
            CheckedStmt::Macro(m) => HirStmt::Macro(self.lower_macro(m)),
        }
    }
    
    fn lower_let_stmt(&mut self, l: CheckedLetStmt) -> HirLetStmt {
        HirLetStmt {
            id: self.next_id(),
            span: l.span,
            name: l.name,
            ty: l.ty.map(|t| self.lower_type(t)),
            init: l.init.map(|e| self.lower_expr(e)),
            is_mut: l.is_mut,
            is_ref: l.is_ref,
        }
    }
    
    fn lower_expr(&mut self, expr: CheckedExpr) -> HirExpr {
        let kind = match expr.kind {
            CheckedExprKind::Bool(v) => HirExprKind::Bool(v),
            CheckedExprKind::Integer(v) => HirExprKind::Integer(v),
            CheckedExprKind::Float(v) => HirExprKind::Float(v),
            CheckedExprKind::String(v) => HirExprKind::String(v),
            CheckedExprKind::Char(v) => HirExprKind::Char(v),
            CheckedExprKind::Path(p) => HirExprKind::Path(self.lower_path(p)),
            CheckedExprKind::Unary(op, e) => HirExprKind::Unary(op, Box::new(self.lower_expr(*e))),
            CheckedExprKind::Binary(op, l, r) => HirExprKind::Binary(op, Box::new(self.lower_expr(*l)), Box::new(self.lower_expr(*r))),
            CheckedExprKind::Call(callee, args) => HirExprKind::Call(Box::new(self.lower_expr(*callee)), args.into_iter().map(|a| self.lower_expr(a)).collect()),
            CheckedExprKind::MethodCall(receiver, method, type_args, args) => HirExprKind::MethodCall(
                Box::new(self.lower_expr(*receiver)),
                method,
                type_args.into_iter().map(|t| self.lower_type(t)).collect(),
                args.into_iter().map(|a| self.lower_expr(a)).collect(),
            ),
            CheckedExprKind::Field(e, field) => HirExprKind::Field(Box::new(self.lower_expr(*e)), field),
            CheckedExprKind::Index(e, idx) => HirExprKind::Index(Box::new(self.lower_expr(*e)), Box::new(self.lower_expr(*idx))),
            CheckedExprKind::Range(start, end, end_kind) => HirExprKind::Range(
                Box::new(self.lower_expr(*start)),
                Box::new(self.lower_expr(*end)),
                end_kind,
            ),
            CheckedExprKind::Struct(path, fields) => HirExprKind::Struct(
                self.lower_path(path),
                fields.into_iter().map(|f| HirFieldExpr {
                    name: f.name,
                    expr: self.lower_expr(f.expr),
                }).collect(),
            ),
            CheckedExprKind::Array(elems) => HirExprKind::Array(elems.into_iter().map(|e| self.lower_expr(e)).collect()),
            CheckedExprKind::Tuple(elems) => HirExprKind::Tuple(elems.into_iter().map(|e| self.lower_expr(e)).collect()),
            CheckedExprKind::Block(b) => HirExprKind::Block(self.lower_block(b)),
            CheckedExprKind::If(cond, then_branch, else_branch) => HirExprKind::If(
                Box::new(self.lower_expr(*cond)),
                self.lower_block(then_branch),
                else_branch.map(|e| Box::new(self.lower_expr(*e))),
            ),
            CheckedExprKind::Match(scrutinee, arms) => HirExprKind::Match(
                Box::new(self.lower_expr(*scrutinee)),
                arms.into_iter().map(|a| self.lower_arm(a)).collect(),
            ),
            CheckedExprKind::Loop(body, label) => HirExprKind::Loop(self.lower_block(body), label),
            CheckedExprKind::While(cond, body, label) => HirExprKind::While(
                Box::new(self.lower_expr(*cond)),
                self.lower_block(body),
                label,
            ),
            CheckedExprKind::For(var, iter, body, label) => HirExprKind::For(
                var,
                Box::new(self.lower_expr(*iter)),
                self.lower_block(body),
                label,
            ),
            CheckedExprKind::Break(label, value) => HirExprKind::Break(label, value.map(|v| Box::new(self.lower_expr(*v)))),
            CheckedExprKind::Continue(label) => HirExprKind::Continue(label),
            CheckedExprKind::Return(value) => HirExprKind::Return(value.map(|v| Box::new(self.lower_expr(*v)))),
            CheckedExprKind::Try(e) => HirExprKind::Try(Box::new(self.lower_expr(*e))),
            CheckedExprKind::Closure(c) => HirExprKind::Closure(self.lower_closure(c)),
            CheckedExprKind::Cast(e, ty) => HirExprKind::Cast(Box::new(self.lower_expr(*e)), self.lower_type(ty)),
            CheckedExprKind::Assign(l, r) => HirExprKind::Assign(Box::new(self.lower_expr(*l)), Box::new(self.lower_expr(*r))),
            CheckedExprKind::AssignOp(op, l, r) => HirExprKind::AssignOp(op, Box::new(self.lower_expr(*l)), Box::new(self.lower_expr(*r))),
            CheckedExprKind::Ref(e, m) => HirExprKind::Ref(Box::new(self.lower_expr(*e)), m),
            CheckedExprKind::Deref(e) => HirExprKind::Deref(Box::new(self.lower_expr(*e))),
            CheckedExprKind::Group(e) => HirExprKind::Group(Box::new(self.lower_expr(*e))),
        };
        
        HirExpr {
            id: self.next_id(),
            span: expr.span,
            kind,
            ty: expr.ty.map(|t| self.lower_type(t)),
        }
    }
    
    fn lower_arm(&mut self, arm: CheckedArm) -> HirArm {
        HirArm {
            id: self.next_id(),
            span: arm.span,
            pattern: self.lower_pattern(arm.pattern),
            guard: arm.guard.map(|g| self.lower_expr(g)),
            body: self.lower_expr(arm.body),
        }
    }
    
    fn lower_pattern(&mut self, pat: CheckedPattern) -> HirPattern {
        match pat {
            CheckedPattern::Wildcard => HirPattern::Wildcard,
            CheckedPattern::Ident(name) => HirPattern::Ident(name),
            CheckedPattern::Literal(e) => HirPattern::Literal(self.lower_expr_kind(e)),
            CheckedPattern::Path(p) => HirPattern::Path(self.lower_path(p)),
            CheckedPattern::Tuple(pats) => HirPattern::Tuple(pats.into_iter().map(|p| self.lower_pattern(p)).collect()),
            CheckedPattern::Struct(path, fields) => HirPattern::Struct(
                self.lower_path(path),
                fields.into_iter().map(|f| HirFieldPattern {
                    name: f.name,
                    pattern: self.lower_pattern(f.pattern),
                }).collect(),
            ),
            CheckedPattern::Or(pats) => HirPattern::Or(pats.into_iter().map(|p| self.lower_pattern(p)).collect()),
            CheckedPattern::Rest => HirPattern::Rest,
            CheckedPattern::Ref(p, m) => HirPattern::Ref(Box::new(self.lower_pattern(*p)), m),
        }
    }
    
    fn lower_expr_kind(&mut self, kind: CheckedExprKind) -> HirExprKind {
        match kind {
            CheckedExprKind::Bool(v) => HirExprKind::Bool(v),
            CheckedExprKind::Integer(v) => HirExprKind::Integer(v),
            CheckedExprKind::Float(v) => HirExprKind::Float(v),
            CheckedExprKind::String(v) => HirExprKind::String(v),
            CheckedExprKind::Char(v) => HirExprKind::Char(v),
            CheckedExprKind::Path(p) => HirExprKind::Path(self.lower_path(p)),
            _ => HirExprKind::Integer(0),
        }
    }
    
    fn lower_closure(&mut self, c: CheckedClosure) -> HirClosure {
        HirClosure {
            params: c.params.into_iter().map(|p| self.lower_param(p)).collect(),
            return_type: c.return_type.map(|t| self.lower_type(t)),
            body: self.lower_block(c.body),
            is_move: c.is_move,
        }
    }
    
    fn lower_macro(&mut self, m: CheckedMacroCall) -> HirMacroCall {
        HirMacroCall {
            id: self.next_id(),
            span: m.span,
            path: self.lower_path(m.path),
            args: m.args.into_iter().map(|a| self.lower_macro_arg(a)).collect(),
        }
    }
    
    fn lower_macro_arg(&mut self, arg: CheckedMacroArg) -> HirMacroArg {
        match arg {
            CheckedMacroArg::Expr(e) => HirMacroArg::Expr(self.lower_expr(e)),
            CheckedMacroArg::Stmt(s) => HirMacroArg::Stmt(self.lower_stmt(s)),
            CheckedMacroArg::Pat(p) => HirMacroArg::Pat(self.lower_pattern(p)),
            CheckedMacroArg::Ty(t) => HirMacroArg::Ty(self.lower_type(t)),
            CheckedMacroArg::Block(b) => HirMacroArg::Block(self.lower_block(b)),
        }
    }
    
    fn lower_struct(&mut self, s: CheckedStruct) -> anyhow::Result<HirStruct> {
        Ok(HirStruct {
            id: self.next_id(),
            span: s.span,
            name: s.name,
            generics: s.generics.into_iter().map(|g| self.lower_generic_param(g)).collect(),
            fields: s.fields.into_iter().map(|f| HirStructField {
                id: self.next_id(),
                span: f.span,
                name: f.name,
                ty: self.lower_type(f.ty),
                is_pub: f.is_pub,
                attributes: f.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
            }).collect(),
            is_pub: s.is_pub,
            attributes: s.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
        })
    }
    
    fn lower_enum(&mut self, e: CheckedEnum) -> anyhow::Result<HirEnum> {
        Ok(HirEnum {
            id: self.next_id(),
            span: e.span,
            name: e.name,
            generics: e.generics.into_iter().map(|g| self.lower_generic_param(g)).collect(),
            variants: e.variants.into_iter().map(|v| self.lower_enum_variant(v)).collect(),
            is_pub: e.is_pub,
            attributes: e.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
        })
    }
    
    fn lower_enum_variant(&mut self, v: ResolvedEnumVariant) -> HirEnumVariant {
        HirEnumVariant {
            id: self.next_id(),
            span: v.span,
            name: v.name,
            data: match v.data {
                ResolvedEnumVariantData::Unit => HirEnumVariantData::Unit,
                ResolvedEnumVariantData::Tuple(types) => HirEnumVariantData::Tuple(types.into_iter().map(|t| self.lower_type(t)).collect()),
                ResolvedEnumVariantData::Struct(fields) => HirEnumVariantData::Struct(fields.into_iter().map(|f| HirStructField {
                    id: self.next_id(),
                    span: f.span,
                    name: f.name,
                    ty: self.lower_type(f.ty),
                    is_pub: f.is_pub,
                    attributes: f.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
                }).collect()),
            },
            discriminant: v.discriminant.map(|d| self.lower_expr(d)),
            attributes: v.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
        }
    }
    
    fn lower_impl(&mut self, i: CheckedImpl) -> anyhow::Result<HirImpl> {
        Ok(HirImpl {
            id: self.next_id(),
            span: i.span,
            generics: i.generics.into_iter().map(|g| self.lower_generic_param(g)).collect(),
            trait_ref: i.trait_ref.map(|t| self.lower_path(t)),
            self_ty: self.lower_type(i.self_ty),
            items: i.items.into_iter().map(|item| match item {
                CheckedImplItem::Function(f) => HirImplItem::Function(self.lower_function(f).unwrap()),
                CheckedImplItem::Const(c) => HirImplItem::Const(self.lower_const(c).unwrap()),
                CheckedImplItem::TypeAlias(t) => HirImplItem::TypeAlias(self.lower_type_alias(t).unwrap()),
            }).collect(),
            attributes: i.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
        })
    }
    
    fn lower_type_alias(&mut self, t: CheckedTypeAlias) -> anyhow::Result<HirTypeAlias> {
        Ok(HirTypeAlias {
            id: self.next_id(),
            span: t.span,
            name: t.name,
            generics: t.generics.into_iter().map(|g| self.lower_generic_param(g)).collect(),
            ty: self.lower_type(t.ty),
            is_pub: t.is_pub,
            attributes: t.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
        })
    }
    
    fn lower_const(&mut self, c: CheckedConst) -> anyhow::Result<HirConst> {
        Ok(HirConst {
            id: self.next_id(),
            span: c.span,
            name: c.name,
            ty: self.lower_type(c.ty),
            value: self.lower_expr(c.value),
            is_pub: c.is_pub,
            attributes: c.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
        })
    }
    
    fn lower_static(&mut self, s: CheckedStatic) -> anyhow::Result<HirStatic> {
        Ok(HirStatic {
            id: self.next_id(),
            span: s.span,
            name: s.name,
            ty: self.lower_type(s.ty),
            value: self.lower_expr(s.value),
            is_pub: s.is_pub,
            is_mut: s.is_mut,
            attributes: s.attributes.into_iter().map(|a| self.lower_attribute(a)).collect(),
        })
    }
    
    fn lower_extern_block(&mut self, e: CheckedExternBlock) -> anyhow::Result<HirExternBlock> {
        Ok(HirExternBlock {
            id: self.next_id(),
            span: e.span,
            abi: e.abi,
            items: e.items.into_iter().map(|item| match item {
                CheckedExternItem::Function(f) => HirExternItem::Function(self.lower_function(f).unwrap()),
                CheckedExternItem::Static(s) => HirExternItem::Static(self.lower_static(s).unwrap()),
            }).collect(),
        })
    }
    
    fn lower_attribute(&mut self, attr: Attribute) -> HirAttribute {
        HirAttribute {
            id: self.next_id(),
            span: attr.span,
            path: self.lower_path(attr.path),
            args: attr.args.into_iter().map(|arg| match arg {
                AttributeArg::Ident(name) => HirAttributeArg::Ident(name),
                AttributeArg::Literal(e) => HirAttributeArg::Literal(self.lower_expr_kind(e)),
                AttributeArg::NameValue(name, e) => HirAttributeArg::NameValue(name, self.lower_expr(e)),
            }).collect(),
        }
    }
}

// Type aliases for the checked AST types used in lowering
use crate::resolver::{ResolvedModule, ResolvedItem, ResolvedFunction, ResolvedStruct, ResolvedEnum, ResolvedImpl, ResolvedTypeAlias, ResolvedConst, ResolvedStatic, ResolvedExternBlock, ResolvedExternItem, ResolvedImport, ResolvedImportItem, ResolvedGenericParam, ResolvedParam, ResolvedStructField, ResolvedEnumVariant, ResolvedEnumVariantData, ResolvedBlock, ResolvedStmt, ResolvedLetStmt, ResolvedExpr, ResolvedExprKind, ResolvedFieldExpr, ResolvedArm, ResolvedPattern, ResolvedFieldPattern, ResolvedClosure, ResolvedMacroCall, ResolvedMacroArg, ResolvedPath, ResolvedPathSegment, ResolvedType, IntTy, FloatTy, Mutability, ResolvedFnSig, UnOp, BinOp, RangeEnd, Attribute, AttributeArg};
use crate::ty::{TypedModule, TypedItem, TypedFunction, TypedStruct, TypedEnum, TypedImpl, TypedImplItem, TypedTypeAlias, TypedConst, TypedStatic, TypedExternBlock, TypedExternItem, TypedBlock, TypedStmt, TypedLetStmt, TypedExpr, TypedExprKind, TypedFieldExpr, TypedArm, TypedPattern, TypedFieldPattern, TypedClosure, TypedMacroCall, TypedMacroArg, TypedPath, TypedPathSegment};
use crate::borrow::{CheckedModule, CheckedItem, CheckedFunction, CheckedStruct, CheckedEnum, CheckedImpl, CheckedImplItem, CheckedTypeAlias, CheckedConst, CheckedStatic, CheckedExternBlock, CheckedExternItem, CheckedBlock, CheckedStmt, CheckedLetStmt, CheckedExpr, CheckedExprKind, CheckedFieldExpr, CheckedArm, CheckedPattern, CheckedFieldPattern, CheckedClosure, CheckedMacroCall, CheckedMacroArg, CheckedPath, CheckedPathSegment, CheckedParam};