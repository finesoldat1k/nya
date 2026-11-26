//! Name resolution for nya

use crate::hir::{HirModule, HirItem, HirItemKind, HirExpr, HirExprKind, HirType, HirPath, HirPat, HirStmt, HirArm};
use nya_frontend::ast::{Module, Item, Import, ImportItem, Function, Struct, Enum, Impl, TypeAlias, Const, Static, ExternBlock, ExternItem, Attribute, MacroCall, Block, Stmt, LetStmt, Expr, ExprKind, Pattern, MatchArm, FieldExpr, FieldPattern, Closure, Param, GenericParam, Type, Path, PathSegment, StructField, EnumVariant, EnumVariantData, ImplItem, MacroArg};
use nya_frontend::diagnostics::{Diagnostic, DiagnosticBag, Label, Severity, Span};
use indexmap::IndexMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::rc::Rc;

/// Resolved item kinds
#[derive(Debug, Clone)]
pub enum ResolvedItem {
    Function(ResolvedFunction),
    Struct(ResolvedStruct),
    Enum(ResolvedEnum),
    Impl(ResolvedImpl),
    TypeAlias(ResolvedTypeAlias),
    Const(ResolvedConst),
    Static(ResolvedStatic),
    ExternBlock(ResolvedExternBlock),
    Module(ResolvedModule),
}

#[derive(Debug, Clone)]
pub struct ResolvedFunction {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub generics: Vec<ResolvedGenericParam>,
    pub params: Vec<ResolvedParam>,
    pub return_type: Option<ResolvedType>,
    pub body: Option<ResolvedBlock>,
    pub is_pub: bool,
    pub is_static: bool,
    pub is_extern: bool,
    pub abi: Option<String>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct ResolvedStruct {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub generics: Vec<ResolvedGenericParam>,
    pub fields: Vec<ResolvedStructField>,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct ResolvedEnum {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub generics: Vec<ResolvedGenericParam>,
    pub variants: Vec<ResolvedEnumVariant>,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct ResolvedImpl {
    pub id: usize,
    pub span: Span,
    pub generics: Vec<ResolvedGenericParam>,
    pub trait_ref: Option<ResolvedPath>,
    pub self_ty: ResolvedType,
    pub items: Vec<ResolvedImplItem>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum ResolvedImplItem {
    Function(ResolvedFunction),
    Const(ResolvedConst),
    TypeAlias(ResolvedTypeAlias),
}

#[derive(Debug, Clone)]
pub struct ResolvedTypeAlias {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub generics: Vec<ResolvedGenericParam>,
    pub ty: ResolvedType,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConst {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: ResolvedType,
    pub value: ResolvedExpr,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct ResolvedStatic {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: ResolvedType,
    pub value: ResolvedExpr,
    pub is_pub: bool,
    pub is_mut: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct ResolvedExternBlock {
    pub id: usize,
    pub span: Span,
    pub abi: String,
    pub items: Vec<ResolvedExternItem>,
}

#[derive(Debug, Clone)]
pub enum ResolvedExternItem {
    Function(ResolvedFunction),
    Static(ResolvedStatic),
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub id: usize,
    pub name: String,
    pub span: Span,
    pub items: Vec<ResolvedItem>,
    pub imports: Vec<ResolvedImport>,
}

#[derive(Debug, Clone)]
pub struct ResolvedImport {
    pub id: usize,
    pub span: Span,
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub items: Vec<ResolvedImportItem>,
}

#[derive(Debug, Clone)]
pub struct ResolvedImportItem {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedGenericParam {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub bounds: Vec<ResolvedType>,
    pub default: Option<ResolvedType>,
}

#[derive(Debug, Clone)]
pub struct ResolvedParam {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: ResolvedType,
    pub is_mut: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedStructField {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: ResolvedType,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct ResolvedEnumVariant {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub data: ResolvedEnumVariantData,
    pub discriminant: Option<ResolvedExpr>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum ResolvedEnumVariantData {
    Unit,
    Tuple(Vec<ResolvedType>),
    Struct(Vec<ResolvedStructField>),
}

#[derive(Debug, Clone)]
pub struct ResolvedBlock {
    pub id: usize,
    pub span: Span,
    pub stmts: Vec<ResolvedStmt>,
    pub expr: Option<Box<ResolvedExpr>>,
}

#[derive(Debug, Clone)]
pub enum ResolvedStmt {
    Let(ResolvedLetStmt),
    Expr(ResolvedExpr),
    Item(ResolvedItem),
    Macro(ResolvedMacroCall),
}

#[derive(Debug, Clone)]
pub struct ResolvedLetStmt {
    pub id: usize,
    pub span: Span,
    pub name: String,
    pub ty: Option<ResolvedType>,
    pub init: Option<ResolvedExpr>,
    pub is_mut: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedExpr {
    pub id: usize,
    pub span: Span,
    pub kind: ResolvedExprKind,
    pub ty: Option<ResolvedType>,
}

#[derive(Debug, Clone)]
pub enum ResolvedExprKind {
    Bool(bool),
    Integer(u64),
    Float(f64),
    String(String),
    Char(char),
    Path(ResolvedPath),
    Unary(UnOp, Box<ResolvedExpr>),
    Binary(BinOp, Box<ResolvedExpr>, Box<ResolvedExpr>),
    Call(Box<ResolvedExpr>, Vec<ResolvedExpr>),
    MethodCall(Box<ResolvedExpr>, String, Vec<ResolvedType>, Vec<ResolvedExpr>),
    Field(Box<ResolvedExpr>, String),
    Index(Box<ResolvedExpr>, Box<ResolvedExpr>),
    Range(Box<ResolvedExpr>, Box<ResolvedExpr>, RangeEnd),
    Struct(ResolvedPath, Vec<ResolvedFieldExpr>),
    Array(Vec<ResolvedExpr>),
    Tuple(Vec<ResolvedExpr>),
    Block(ResolvedBlock),
    If(Box<ResolvedExpr>, ResolvedBlock, Option<Box<ResolvedExpr>>),
    Match(Box<ResolvedExpr>, Vec<ResolvedArm>),
    Loop(ResolvedBlock, Option<String>),
    While(Box<ResolvedExpr>, ResolvedBlock, Option<String>),
    For(String, Box<ResolvedExpr>, ResolvedBlock, Option<String>),
    Break(Option<String>, Option<Box<ResolvedExpr>>),
    Continue(Option<String>),
    Return(Option<Box<ResolvedExpr>>),
    Try(Box<ResolvedExpr>),
    Closure(ResolvedClosure),
    Cast(Box<ResolvedExpr>, ResolvedType),
    Assign(Box<ResolvedExpr>, Box<ResolvedExpr>),
    AssignOp(BinOp, Box<ResolvedExpr>, Box<ResolvedExpr>),
    Ref(Box<ResolvedExpr>, Mutability),
    Deref(Box<ResolvedExpr>),
    Group(Box<ResolvedExpr>),
}

#[derive(Debug, Clone)]
pub struct ResolvedFieldExpr {
    pub name: String,
    pub expr: ResolvedExpr,
}

#[derive(Debug, Clone)]
pub struct ResolvedArm {
    pub id: usize,
    pub span: Span,
    pub pattern: ResolvedPattern,
    pub guard: Option<ResolvedExpr>,
    pub body: ResolvedExpr,
}

#[derive(Debug, Clone)]
pub enum ResolvedPattern {
    Wildcard,
    Ident(String),
    Literal(ResolvedExprKind),
    Path(ResolvedPath),
    Tuple(Vec<ResolvedPattern>),
    Struct(ResolvedPath, Vec<ResolvedFieldPattern>),
    Or(Vec<ResolvedPattern>),
    Rest,
    Ref(Box<ResolvedPattern>, Mutability),
}

#[derive(Debug, Clone)]
pub struct ResolvedFieldPattern {
    pub name: String,
    pub pattern: ResolvedPattern,
}

#[derive(Debug, Clone)]
pub struct ResolvedClosure {
    pub params: Vec<ResolvedParam>,
    pub return_type: Option<ResolvedType>,
    pub body: ResolvedBlock,
    pub is_move: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedMacroCall {
    pub id: usize,
    pub span: Span,
    pub path: ResolvedPath,
    pub args: Vec<ResolvedMacroArg>,
}

#[derive(Debug, Clone)]
pub enum ResolvedMacroArg {
    Expr(ResolvedExpr),
    Stmt(ResolvedStmt),
    Pat(ResolvedPattern),
    Ty(ResolvedType),
    Block(ResolvedBlock),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPath {
    pub segments: Vec<ResolvedPathSegment>,
    pub is_absolute: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPathSegment {
    pub name: String,
    pub type_args: Vec<ResolvedType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedType {
    Bool,
    Int(IntTy),
    Float(FloatTy),
    Char,
    Str,
    Never,
    Path(ResolvedPath),
    Tuple(Vec<ResolvedType>),
    Array(Box<ResolvedType>, Option<usize>),
    Slice(Box<ResolvedType>),
    Ref(Box<ResolvedType>, Mutability),
    RawPtr(Box<ResolvedType>, Mutability),
    FnPtr(ResolvedFnSig),
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

#[derive(Debug, Clone)]
pub struct ResolvedFnSig {
    pub params: Vec<ResolvedType>,
    pub return_type: Box<ResolvedType>,
    pub is_variadic: bool,
    pub abi: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
    Deref,
    Ref,
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
    BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeEnd {
    Included,
    Excluded,
}

/// Name resolution context
pub struct Resolver {
    module: Module,
    scopes: Vec<Scope>,
    current_module: String,
    diagnostics: DiagnosticBag,
    next_id: usize,
}

struct Scope {
    items: IndexMap<String, ResolvedItem>,
    parent: Option<usize>,
}

impl Resolver {
    pub fn new(module: Module) -> Self {
        Self {
            module,
            scopes: vec![Scope { items: IndexMap::new(), parent: None }],
            current_module: "main".to_string(),
            diagnostics: DiagnosticBag::new(),
            next_id: 0,
        }
    }
    
    pub fn resolve(mut self) -> Result<ResolvedModule, DiagnosticBag> {
        let module = self.resolve_module();
        
        if self.diagnostics.has_errors() {
            Err(self.diagnostics)
        } else {
            Ok(module)
        }
    }
    
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            items: IndexMap::new(),
            parent: Some(self.scopes.len() - 1),
        });
    }
    
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    
    fn declare(&mut self, name: String, item: ResolvedItem) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.items.insert(name, item);
        }
    }
    
    fn resolve_name(&self, name: &str) -> Option<&ResolvedItem> {
        for scope in self.scopes.iter().rev() {
            if let Some(item) = scope.items.get(name) {
                return Some(item);
            }
        }
        None
    }
    
    fn resolve_module(&mut self) -> ResolvedModule {
        let start_span = self.module.span;
        let mut items = Vec::new();
        let mut imports = Vec::new();
        
        // First pass: declare all items
        for item in &self.module.items {
            self.declare_item(item);
        }
        
        // Second pass: resolve imports
        for import in &self.module.imports {
            imports.push(self.resolve_import(import));
        }
        
        // Third pass: resolve item bodies
        for item in &self.module.items {
            items.push(self.resolve_item(item));
        }
        
        ResolvedModule {
            id: self.next_id(),
            name: self.module.name.clone(),
            span: start_span,
            items,
            imports,
        }
    }
    
    fn declare_item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => {
                self.declare(f.name.clone(), ResolvedItem::Function(ResolvedFunction {
                    id: self.next_id(),
                    span: f.span,
                    name: f.name.clone(),
                    generics: Vec::new(),
                    params: Vec::new(),
                    return_type: None,
                    body: None,
                    is_pub: f.is_pub,
                    is_static: f.is_static,
                    is_extern: f.is_extern,
                    abi: f.abi.clone(),
                    attributes: f.attributes.clone(),
                }));
            }
            Item::Struct(s) => {
                self.declare(s.name.clone(), ResolvedItem::Struct(ResolvedStruct {
                    id: self.next_id(),
                    span: s.span,
                    name: s.name.clone(),
                    generics: Vec::new(),
                    fields: Vec::new(),
                    is_pub: s.is_pub,
                    attributes: s.attributes.clone(),
                }));
            }
            Item::Enum(e) => {
                self.declare(e.name.clone(), ResolvedItem::Enum(ResolvedEnum {
                    id: self.next_id(),
                    span: e.span,
                    name: e.name.clone(),
                    generics: Vec::new(),
                    variants: Vec::new(),
                    is_pub: e.is_pub,
                    attributes: e.attributes.clone(),
                }));
            }
            Item::Impl(i) => {
                // Impl blocks don't introduce names
            }
            Item::TypeAlias(t) => {
                self.declare(t.name.clone(), ResolvedItem::TypeAlias(ResolvedTypeAlias {
                    id: self.next_id(),
                    span: t.span,
                    name: t.name.clone(),
                    generics: Vec::new(),
                    ty: ResolvedType::Infer,
                    is_pub: t.is_pub,
                    attributes: t.attributes.clone(),
                }));
            }
            Item::Const(c) => {
                self.declare(c.name.clone(), ResolvedItem::Const(ResolvedConst {
                    id: self.next_id(),
                    span: c.span,
                    name: c.name.clone(),
                    ty: ResolvedType::Infer,
                    value: ResolvedExpr { id: self.next_id(), span: c.span, kind: ResolvedExprKind::Integer(0), ty: None },
                    is_pub: c.is_pub,
                    attributes: c.attributes.clone(),
                }));
            }
            Item::Static(s) => {
                self.declare(s.name.clone(), ResolvedItem::Static(ResolvedStatic {
                    id: self.next_id(),
                    span: s.span,
                    name: s.name.clone(),
                    ty: ResolvedType::Infer,
                    value: ResolvedExpr { id: self.next_id(), span: s.span, kind: ResolvedExprKind::Integer(0), ty: None },
                    is_pub: s.is_pub,
                    is_mut: s.is_mut,
                    attributes: s.attributes.clone(),
                }));
            }
            Item::ExternBlock(e) => {
                // Extern blocks don't introduce names directly
            }
            Item::Module(m) => {
                self.declare(m.name.clone(), ResolvedItem::Module(ResolvedModule {
                    id: self.next_id(),
                    name: m.name.clone(),
                    span: m.span,
                    items: Vec::new(),
                    imports: Vec::new(),
                }));
            }
        }
    }
    
    fn resolve_import(&mut self, import: &Import) -> ResolvedImport {
        ResolvedImport {
            id: self.next_id(),
            span: import.span,
            path: import.path.clone(),
            alias: import.alias.clone(),
            items: import.items.iter().map(|item| ResolvedImportItem {
                id: self.next_id(),
                span: item.span,
                name: item.name.clone(),
                alias: item.alias.clone(),
            }).collect(),
        }
    }
    
    fn resolve_item(&mut self, item: &Item) -> ResolvedItem {
        match item {
            Item::Fn(f) => ResolvedItem::Function(self.resolve_function(f)),
            Item::Struct(s) => ResolvedItem::Struct(self.resolve_struct(s)),
            Item::Enum(e) => ResolvedItem::Enum(self.resolve_enum(e)),
            Item::Impl(i) => ResolvedItem::Impl(self.resolve_impl(i)),
            Item::TypeAlias(t) => ResolvedItem::TypeAlias(self.resolve_type_alias(t)),
            Item::Const(c) => ResolvedItem::Const(self.resolve_const(c)),
            Item::Static(s) => ResolvedItem::Static(self.resolve_static(s)),
            Item::ExternBlock(e) => ResolvedItem::ExternBlock(self.resolve_extern_block(e)),
            Item::Module(m) => ResolvedItem::Module(self.resolve_module_item(m)),
        }
    }
    
    fn resolve_function(&mut self, func: &Function) -> ResolvedFunction {
        self.push_scope();
        
        // Declare generic params
        let generics = func.generics.iter().map(|g| self.resolve_generic_param(g)).collect();
        
        // Declare parameters
        let params = func.params.iter().map(|p| self.resolve_param(p)).collect();
        
        let return_type = func.return_type.as_ref().map(|t| self.resolve_type(t));
        
        let body = func.body.as_ref().map(|b| self.resolve_block(b));
        
        self.pop_scope();
        
        ResolvedFunction {
            id: self.next_id(),
            span: func.span,
            name: func.name.clone(),
            generics,
            params,
            return_type,
            body,
            is_pub: func.is_pub,
            is_static: func.is_static,
            is_extern: func.is_extern,
            abi: func.abi.clone(),
            attributes: func.attributes.clone(),
        }
    }
    
    fn resolve_struct(&mut self, s: &Struct) -> ResolvedStruct {
        self.push_scope();
        
        let generics = s.generics.iter().map(|g| self.resolve_generic_param(g)).collect();
        
        let fields = s.fields.iter().map(|f| ResolvedStructField {
            id: self.next_id(),
            span: f.span,
            name: f.name.clone(),
            ty: self.resolve_type(&f.ty),
            is_pub: f.is_pub,
            attributes: f.attributes.clone(),
        }).collect();
        
        self.pop_scope();
        
        ResolvedStruct {
            id: self.next_id(),
            span: s.span,
            name: s.name.clone(),
            generics,
            fields,
            is_pub: s.is_pub,
            attributes: s.attributes.clone(),
        }
    }
    
    fn resolve_enum(&mut self, e: &Enum) -> ResolvedEnum {
        self.push_scope();
        
        let generics = e.generics.iter().map(|g| self.resolve_generic_param(g)).collect();
        
        let variants = e.variants.iter().map(|v| self.resolve_enum_variant(v)).collect();
        
        self.pop_scope();
        
        ResolvedEnum {
            id: self.next_id(),
            span: e.span,
            name: e.name.clone(),
            generics,
            variants,
            is_pub: e.is_pub,
            attributes: e.attributes.clone(),
        }
    }
    
    fn resolve_enum_variant(&mut self, v: &EnumVariant) -> ResolvedEnumVariant {
        let data = match &v.data {
            EnumVariantData::Unit => ResolvedEnumVariantData::Unit,
            EnumVariantData::Tuple(types) => ResolvedEnumVariantData::Tuple(types.iter().map(|t| self.resolve_type(t)).collect()),
            EnumVariantData::Struct(fields) => ResolvedEnumVariantData::Struct(fields.iter().map(|f| ResolvedStructField {
                id: self.next_id(),
                span: f.span,
                name: f.name.clone(),
                ty: self.resolve_type(&f.ty),
                is_pub: f.is_pub,
                attributes: f.attributes.clone(),
            }).collect()),
        };
        
        let discriminant = v.discriminant.as_ref().map(|d| self.resolve_expr(d));
        
        ResolvedEnumVariant {
            id: self.next_id(),
            span: v.span,
            name: v.name.clone(),
            data,
            discriminant,
            attributes: v.attributes.clone(),
        }
    }
    
    fn resolve_impl(&mut self, i: &Impl) -> ResolvedImpl {
        self.push_scope();
        
        let generics = i.generics.iter().map(|g| self.resolve_generic_param(g)).collect();
        let trait_ref = i.trait_ref.as_ref().map(|t| self.resolve_path(t));
        let self_ty = self.resolve_type(&i.self_ty);
        
        let items = i.items.iter().map(|item| match item {
            ImplItem::Fn(f) => ResolvedImplItem::Function(self.resolve_function(f)),
            ImplItem::Const(c) => ResolvedImplItem::Const(self.resolve_const(c)),
            ImplItem::TypeAlias(t) => ResolvedImplItem::TypeAlias(self.resolve_type_alias(t)),
        }).collect();
        
        self.pop_scope();
        
        ResolvedImpl {
            id: self.next_id(),
            span: i.span,
            generics,
            trait_ref,
            self_ty,
            items,
            attributes: i.attributes.clone(),
        }
    }
    
    fn resolve_type_alias(&mut self, t: &TypeAlias) -> ResolvedTypeAlias {
        self.push_scope();
        
        let generics = t.generics.iter().map(|g| self.resolve_generic_param(g)).collect();
        let ty = self.resolve_type(&t.ty);
        
        self.pop_scope();
        
        ResolvedTypeAlias {
            id: self.next_id(),
            span: t.span,
            name: t.name.clone(),
            generics,
            ty,
            is_pub: t.is_pub,
            attributes: t.attributes.clone(),
        }
    }
    
    fn resolve_const(&mut self, c: &Const) -> ResolvedConst {
        let ty = self.resolve_type(&c.ty);
        let value = self.resolve_expr(&c.value);
        
        ResolvedConst {
            id: self.next_id(),
            span: c.span,
            name: c.name.clone(),
            ty,
            value,
            is_pub: c.is_pub,
            attributes: c.attributes.clone(),
        }
    }
    
    fn resolve_static(&mut self, s: &Static) -> ResolvedStatic {
        let ty = self.resolve_type(&s.ty);
        let value = self.resolve_expr(&s.value);
        
        ResolvedStatic {
            id: self.next_id(),
            span: s.span,
            name: s.name.clone(),
            ty,
            value,
            is_pub: s.is_pub,
            is_mut: s.is_mut,
            attributes: s.attributes.clone(),
        }
    }
    
    fn resolve_extern_block(&mut self, e: &ExternBlock) -> ResolvedExternBlock {
        let items = e.items.iter().map(|item| match item {
            ExternItem::Fn(f) => ResolvedExternItem::Function(self.resolve_function(f)),
            ExternItem::Static(s) => ResolvedExternItem::Static(self.resolve_static(s)),
        }).collect();
        
        ResolvedExternBlock {
            id: self.next_id(),
            span: e.span,
            abi: e.abi.clone(),
            items,
        }
    }
    
    fn resolve_module_item(&mut self, m: &Module) -> ResolvedModule {
        let old_module = self.current_module.clone();
        self.current_module = m.name.clone();
        
        let module = self.resolve_module();
        
        self.current_module = old_module;
        module
    }
    
    fn resolve_generic_param(&mut self, g: &GenericParam) -> ResolvedGenericParam {
        let bounds = g.bounds.iter().map(|b| self.resolve_type(b)).collect();
        let default = g.default.as_ref().map(|d| self.resolve_type(d));
        
        ResolvedGenericParam {
            id: self.next_id(),
            span: g.span,
            name: g.name.clone(),
            bounds,
            default,
        }
    }
    
    fn resolve_param(&mut self, p: &Param) -> ResolvedParam {
        ResolvedParam {
            id: self.next_id(),
            span: p.span,
            name: p.name.clone(),
            ty: self.resolve_type(&p.ty),
            is_mut: p.is_mut,
            is_ref: p.is_ref,
        }
    }
    
    fn resolve_block(&mut self, block: &Block) -> ResolvedBlock {
        self.push_scope();
        
        let stmts = block.stmts.iter().map(|s| self.resolve_stmt(s)).collect();
        let expr = block.expr.as_ref().map(|e| Box::new(self.resolve_expr(e)));
        
        self.pop_scope();
        
        ResolvedBlock {
            id: self.next_id(),
            span: block.span,
            stmts,
            expr,
        }
    }
    
    fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolvedStmt {
        match stmt {
            Stmt::Let(l) => ResolvedStmt::Let(self.resolve_let_stmt(l)),
            Stmt::Expr(e) => ResolvedStmt::Expr(self.resolve_expr(e)),
            Stmt::Item(i) => ResolvedStmt::Item(self.resolve_item(i)),
            Stmt::Macro(m) => ResolvedStmt::Macro(self.resolve_macro(m)),
        }
    }
    
    fn resolve_let_stmt(&mut self, l: &LetStmt) -> ResolvedLetStmt {
        let ty = l.ty.as_ref().map(|t| self.resolve_type(t));
        let init = l.init.as_ref().map(|e| self.resolve_expr(e));
        
        ResolvedLetStmt {
            id: self.next_id(),
            span: l.span,
            name: l.name.clone(),
            ty,
            init,
            is_mut: l.is_mut,
            is_ref: l.is_ref,
        }
    }
    
    fn resolve_expr(&mut self, expr: &Expr) -> ResolvedExpr {
        let kind = match &expr.kind {
            ExprKind::Bool(v) => ResolvedExprKind::Bool(*v),
            ExprKind::Integer(v) => ResolvedExprKind::Integer(*v),
            ExprKind::Float(v) => ResolvedExprKind::Float(*v),
            ExprKind::String(v) => ResolvedExprKind::String(v.clone()),
            ExprKind::Char(v) => ResolvedExprKind::Char(*v),
            ExprKind::Path(p) => ResolvedExprKind::Path(self.resolve_path(p)),
            ExprKind::Unary(op, e) => ResolvedExprKind::Unary(*op, Box::new(self.resolve_expr(e))),
            ExprKind::Binary(op, l, r) => ResolvedExprKind::Binary(*op, Box::new(self.resolve_expr(l)), Box::new(self.resolve_expr(r))),
            ExprKind::Call(callee, args) => ResolvedExprKind::Call(Box::new(self.resolve_expr(callee)), args.iter().map(|a| self.resolve_expr(a)).collect()),
            ExprKind::MethodCall(receiver, method, type_args, args) => ResolvedExprKind::MethodCall(
                Box::new(self.resolve_expr(receiver)),
                method.clone(),
                type_args.iter().map(|t| self.resolve_type(t)).collect(),
                args.iter().map(|a| self.resolve_expr(a)).collect(),
            ),
            ExprKind::Field(e, field) => ResolvedExprKind::Field(Box::new(self.resolve_expr(e)), field.clone()),
            ExprKind::Index(e, idx) => ResolvedExprKind::Index(Box::new(self.resolve_expr(e)), Box::new(self.resolve_expr(idx))),
            ExprKind::Range(start, end, end_kind) => ResolvedExprKind::Range(
                Box::new(self.resolve_expr(start)),
                Box::new(self.resolve_expr(end)),
                *end_kind,
            ),
            ExprKind::Struct(path, fields) => ResolvedExprKind::Struct(
                self.resolve_path(path),
                fields.iter().map(|f| ResolvedFieldExpr { name: f.name.clone(), expr: self.resolve_expr(&f.expr) }).collect(),
            ),
            ExprKind::Array(elems) => ResolvedExprKind::Array(elems.iter().map(|e| self.resolve_expr(e)).collect()),
            ExprKind::Tuple(elems) => ResolvedExprKind::Tuple(elems.iter().map(|e| self.resolve_expr(e)).collect()),
            ExprKind::Block(b) => ResolvedExprKind::Block(self.resolve_block(b)),
            ExprKind::If(cond, then_branch, else_branch) => ResolvedExprKind::If(
                Box::new(self.resolve_expr(cond)),
                self.resolve_block(then_branch),
                else_branch.as_ref().map(|e| Box::new(self.resolve_expr(e))),
            ),
            ExprKind::Match(scrutinee, arms) => ResolvedExprKind::Match(
                Box::new(self.resolve_expr(scrutinee)),
                arms.iter().map(|a| self.resolve_arm(a)).collect(),
            ),
            ExprKind::Loop(body, label) => ResolvedExprKind::Loop(self.resolve_block(body), label.clone()),
            ExprKind::While(cond, body, label) => ResolvedExprKind::While(
                Box::new(self.resolve_expr(cond)),
                self.resolve_block(body),
                label.clone(),
            ),
            ExprKind::For(var, iter, body, label) => ResolvedExprKind::For(
                var.clone(),
                Box::new(self.resolve_expr(iter)),
                self.resolve_block(body),
                label.clone(),
            ),
            ExprKind::Break(label, value) => ResolvedExprKind::Break(label.clone(), value.as_ref().map(|v| Box::new(self.resolve_expr(v)))),
            ExprKind::Continue(label) => ResolvedExprKind::Continue(label.clone()),
            ExprKind::Return(value) => ResolvedExprKind::Return(value.as_ref().map(|v| Box::new(self.resolve_expr(v)))),
            ExprKind::Try(e) => ResolvedExprKind::Try(Box::new(self.resolve_expr(e))),
            ExprKind::Closure(c) => ResolvedExprKind::Closure(self.resolve_closure(c)),
            ExprKind::Cast(e, ty) => ResolvedExprKind::Cast(Box::new(self.resolve_expr(e)), self.resolve_type(ty)),
            ExprKind::Assign(l, r) => ResolvedExprKind::Assign(Box::new(self.resolve_expr(l)), Box::new(self.resolve_expr(r))),
            ExprKind::AssignOp(op, l, r) => ResolvedExprKind::AssignOp(*op, Box::new(self.resolve_expr(l)), Box::new(self.resolve_expr(r))),
            ExprKind::Ref(e, mutability) => ResolvedExprKind::Ref(Box::new(self.resolve_expr(e)), *mutability),
            ExprKind::Deref(e) => ResolvedExprKind::Deref(Box::new(self.resolve_expr(e))),
            ExprKind::Group(e) => ResolvedExprKind::Group(Box::new(self.resolve_expr(e))),
        };
        
        ResolvedExpr {
            id: self.next_id(),
            span: expr.span,
            kind,
            ty: None, // Will be filled by type checker
        }
    }
    
    fn resolve_arm(&mut self, arm: &MatchArm) -> ResolvedArm {
        let pattern = self.resolve_pattern(&arm.pattern);
        let guard = arm.guard.as_ref().map(|g| self.resolve_expr(g));
        let body = self.resolve_expr(&arm.body);
        
        ResolvedArm {
            id: self.next_id(),
            span: arm.span,
            pattern,
            guard,
            body,
        }
    }
    
    fn resolve_pattern(&mut self, pat: &Pattern) -> ResolvedPattern {
        match pat {
            Pattern::Wildcard => ResolvedPattern::Wildcard,
            Pattern::Ident(name) => ResolvedPattern::Ident(name.clone()),
            Pattern::Literal(e) => ResolvedPattern::Literal(self.resolve_expr_kind(e)),
            Pattern::Path(p) => ResolvedPattern::Path(self.resolve_path(p)),
            Pattern::Tuple(pats) => ResolvedPattern::Tuple(pats.iter().map(|p| self.resolve_pattern(p)).collect()),
            Pattern::Struct(path, fields) => ResolvedPattern::Struct(
                self.resolve_path(path),
                fields.iter().map(|f| ResolvedFieldPattern { name: f.name.clone(), pattern: self.resolve_pattern(&f.pattern) }).collect(),
            ),
            Pattern::Or(pats) => ResolvedPattern::Or(pats.iter().map(|p| self.resolve_pattern(p)).collect()),
            Pattern::Rest => ResolvedPattern::Rest,
            Pattern::Ref(p, mutability) => ResolvedPattern::Ref(Box::new(self.resolve_pattern(p)), *mutability),
        }
    }
    
    fn resolve_expr_kind(&mut self, kind: &ExprKind) -> ResolvedExprKind {
        match kind {
            ExprKind::Bool(v) => ResolvedExprKind::Bool(*v),
            ExprKind::Integer(v) => ResolvedExprKind::Integer(*v),
            ExprKind::Float(v) => ResolvedExprKind::Float(*v),
            ExprKind::String(v) => ResolvedExprKind::String(v.clone()),
            ExprKind::Char(v) => ResolvedExprKind::Char(*v),
            ExprKind::Path(p) => ResolvedExprKind::Path(self.resolve_path(p)),
            _ => ResolvedExprKind::Integer(0), // Simplified
        }
    }
    
    fn resolve_closure(&mut self, c: &Closure) -> ResolvedClosure {
        self.push_scope();
        
        let params = c.params.iter().map(|p| self.resolve_param(p)).collect();
        let return_type = c.return_type.as_ref().map(|t| self.resolve_type(t));
        let body = self.resolve_block(&c.body);
        
        self.pop_scope();
        
        ResolvedClosure {
            params,
            return_type,
            body,
            is_move: c.is_move,
        }
    }
    
    fn resolve_macro(&mut self, m: &MacroCall) -> ResolvedMacroCall {
        ResolvedMacroCall {
            id: self.next_id(),
            span: m.span,
            path: self.resolve_path(&m.path),
            args: m.args.iter().map(|a| self.resolve_macro_arg(a)).collect(),
        }
    }
    
    fn resolve_macro_arg(&mut self, arg: &MacroArg) -> ResolvedMacroArg {
        match arg {
            MacroArg::Expr(e) => ResolvedMacroArg::Expr(self.resolve_expr(e)),
            MacroArg::Stmt(s) => ResolvedMacroArg::Stmt(self.resolve_stmt(s)),
            MacroArg::Pat(p) => ResolvedMacroArg::Pat(self.resolve_pattern(p)),
            MacroArg::Ty(t) => ResolvedMacroArg::Ty(self.resolve_type(t)),
            MacroArg::Block(b) => ResolvedMacroArg::Block(self.resolve_block(b)),
        }
    }
    
    fn resolve_path(&mut self, path: &Path) -> ResolvedPath {
        ResolvedPath {
            segments: path.segments.iter().map(|seg| ResolvedPathSegment {
                name: seg.name.clone(),
                type_args: seg.type_args.iter().map(|t| self.resolve_type(t)).collect(),
            }).collect(),
            is_absolute: path.is_absolute,
        }
    }
    
    fn resolve_type(&mut self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Bool => ResolvedType::Bool,
            Type::Int(t) => ResolvedType::Int(*t),
            Type::Float(t) => ResolvedType::Float(*t),
            Type::Char => ResolvedType::Char,
            Type::Str => ResolvedType::Str,
            Type::Never => ResolvedType::Never,
            Type::Path(p) => ResolvedType::Path(self.resolve_path(p)),
            Type::Tuple(types) => ResolvedType::Tuple(types.iter().map(|t| self.resolve_type(t)).collect()),
            Type::Array(t, size) => ResolvedType::Array(Box::new(self.resolve_type(t)), *size),
            Type::Slice(t) => ResolvedType::Slice(Box::new(self.resolve_type(t))),
            Type::Ref(t, m) => ResolvedType::Ref(Box::new(self.resolve_type(t)), *m),
            Type::RawPtr(t, m) => ResolvedType::RawPtr(Box::new(self.resolve_type(t)), *m),
            Type::FnPtr(sig) => ResolvedType::FnPtr(ResolvedFnSig {
                params: sig.params.iter().map(|p| self.resolve_type(p)).collect(),
                return_type: Box::new(self.resolve_type(&sig.return_type)),
                is_variadic: sig.is_variadic,
                abi: sig.abi.clone(),
            }),
            Type::Generic(name) => ResolvedType::Generic(name.clone()),
            Type::Infer => ResolvedType::Infer,
        }
    }
}

/// Public resolve function
pub fn resolve(module: Module) -> Result<ResolvedModule, DiagnosticBag> {
    let resolver = Resolver::new(module);
    resolver.resolve()
}