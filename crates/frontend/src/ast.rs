//! Abstract Syntax Tree for nya

use crate::diagnostics::Span;
use indexmap::IndexMap;
use smallvec::SmallVec;
use std::fmt;

/// A unique identifier for AST nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Module - top-level container
#[derive(Debug, Clone)]
pub struct Module {
    pub id: NodeId,
    pub name: String,
    pub span: Span,
    pub items: Vec<Item>,
    pub imports: Vec<Import>,
}

/// Import statement
#[derive(Debug, Clone)]
pub struct Import {
    pub id: NodeId,
    pub span: Span,
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub items: Vec<ImportItem>,
}

/// Individual import item (for `use path::{item1, item2}`)
#[derive(Debug, Clone)]
pub struct ImportItem {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub alias: Option<String>,
}

/// Top-level items
#[derive(Debug, Clone)]
pub enum Item {
    Fn(Function),
    Struct(Struct),
    Enum(Enum),
    Impl(Impl),
    TypeAlias(TypeAlias),
    Const(Const),
    Static(Static),
    ExternBlock(ExternBlock),
    Module(Module),
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Option<Block>,
    pub is_pub: bool,
    pub is_static: bool,
    pub is_extern: bool,
    pub abi: Option<String>,
    pub attributes: Vec<Attribute>,
}

/// Generic type parameter
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub bounds: Vec<Type>,
    pub default: Option<Type>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub ty: Type,
    pub is_mut: bool,
    pub is_ref: bool,
}

/// Type representation
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive types
    Bool,
    Int(IntTy),
    Float(FloatTy),
    Char,
    Str,
    Never,
    
    // Named types
    Path(Path),
    
    // Compound types
    Tuple(Vec<Type>),
    Array(Box<Type>, Option<usize>), // None = slice
    Slice(Box<Type>),
    Ref(Box<Type>, Mutability),
    RawPtr(Box<Type>, Mutability),
    FnPtr(FnSig),
    
    // Generics
    Generic(String),
    
    // Inference placeholder
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

/// Function signature for function pointers
#[derive(Debug, Clone)]
pub struct FnSig {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
    pub is_variadic: bool,
    pub abi: Option<String>,
}

/// Path (for type names, function calls, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub is_absolute: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    pub name: String,
    pub type_args: Vec<Type>,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub id: NodeId,
    pub span: Span,
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
}

/// Statements
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    Item(Item),
    Macro(MacroCall),
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub is_mut: bool,
    pub is_ref: bool,
}

/// Expressions
#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub span: Span,
    pub kind: ExprKind,
    pub ty: Option<Type>, // Filled during type checking
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    // Literals
    Bool(bool),
    Integer(u64),
    Float(f64),
    String(String),
    Char(char),
    
    // Paths (variables, functions, constants)
    Path(Path),
    
    // Unary operations
    Unary(UnOp, Box<Expr>),
    
    // Binary operations
    Binary(BinOp, Box<Expr>, Box<Expr>),
    
    // Function call
    Call(Box<Expr>, Vec<Expr>),
    
    // Method call
    MethodCall(Box<Expr>, String, Vec<Type>, Vec<Expr>),
    
    // Field access
    Field(Box<Expr>, String),
    
    // Index access
    Index(Box<Expr>, Box<Expr>),
    
    // Range expressions
    Range(Box<Expr>, Box<Expr>, RangeEnd),
    
    // Struct/Enum construction
    Struct(Path, Vec<FieldExpr>),
    
    // Array literal
    Array(Vec<Expr>),
    
    // Tuple literal
    Tuple(Vec<Expr>),
    
    // Block expression
    Block(Block),
    
    // If expression
    If(Box<Expr>, Block, Option<Box<Expr>>),
    
    // Match expression
    Match(Box<Expr>, Vec<MatchArm>),
    
    // Loop expressions
    Loop(Block, Option<String>),
    While(Box<Expr>, Block, Option<String>),
    For(String, Box<Expr>, Block, Option<String>),
    
    // Break/Continue
    Break(Option<String>, Option<Box<Expr>>),
    Continue(Option<String>),
    
    // Return
    Return(Option<Box<Expr>>),
    
    // Try operator (?)
    Try(Box<Expr>),
    
    // Closure
    Closure(Closure),
    
    // Cast
    Cast(Box<Expr>, Type),
    
    // Assignment
    Assign(Box<Expr>, Box<Expr>),
    
    // Compound assignment
    AssignOp(BinOp, Box<Expr>, Box<Expr>),
    
    // Reference
    Ref(Box<Expr>, Mutability),
    
    // Dereference
    Deref(Box<Expr>),
    
    // Grouped expression
    Group(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeEnd {
    Included,
    Excluded,
}

#[derive(Debug, Clone)]
pub struct FieldExpr {
    pub name: String,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub id: NodeId,
    pub span: Span,
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Literal(ExprKind),
    Path(Path),
    Tuple(Vec<Pattern>),
    Struct(Path, Vec<FieldPattern>),
    Or(Vec<Pattern>),
    Rest,
    Ref(Box<Pattern>, Mutability),
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Pattern,
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_move: bool,
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
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct Struct {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub ty: Type,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct Enum {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub data: EnumVariantData,
    pub discriminant: Option<Expr>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum EnumVariantData {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

/// Impl block
#[derive(Debug, Clone)]
pub struct Impl {
    pub id: NodeId,
    pub span: Span,
    pub generics: Vec<GenericParam>,
    pub trait_ref: Option<Path>,
    pub self_ty: Type,
    pub items: Vec<ImplItem>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum ImplItem {
    Fn(Function),
    Const(Const),
    TypeAlias(TypeAlias),
}

/// Type alias
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub ty: Type,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

/// Const definition
#[derive(Debug, Clone)]
pub struct Const {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

/// Static definition
#[derive(Debug, Clone)]
pub struct Static {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub is_pub: bool,
    pub is_mut: bool,
    pub attributes: Vec<Attribute>,
}

/// Extern block
#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub id: NodeId,
    pub span: Span,
    pub abi: String,
    pub items: Vec<ExternItem>,
}

#[derive(Debug, Clone)]
pub enum ExternItem {
    Fn(Function),
    Static(Static),
}

/// Attribute (like #[test], #[inline], etc.)
#[derive(Debug, Clone)]
pub struct Attribute {
    pub id: NodeId,
    pub span: Span,
    pub path: Path,
    pub args: Vec<AttributeArg>,
}

#[derive(Debug, Clone)]
pub enum AttributeArg {
    Ident(String),
    Literal(ExprKind),
    NameValue(String, Expr),
}

/// Macro call
#[derive(Debug, Clone)]
pub struct MacroCall {
    pub id: NodeId,
    pub span: Span,
    pub path: Path,
    pub args: Vec<MacroArg>,
}

#[derive(Debug, Clone)]
pub enum MacroArg {
    Expr(Expr),
    Stmt(Stmt),
    Pat(Pattern),
    Ty(Type),
    Block(Block),
}

/// Visitor pattern for AST traversal
pub trait AstVisitor {
    fn visit_module(&mut self, module: &Module) {
        walk_module(self, module);
    }
    
    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item);
    }
    
    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }
    
    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }
    
    fn visit_type(&mut self, ty: &Type) {
        walk_type(self, ty);
    }
    
    fn visit_pattern(&mut self, pat: &Pattern) {
        walk_pattern(self, pat);
    }
}

/// Walk functions for default visitor behavior
pub fn walk_module<V: AstVisitor>(visitor: &mut V, module: &Module) {
    for import in &module.imports {
        walk_import(visitor, import);
    }
    for item in &module.items {
        visitor.visit_item(item);
    }
}

pub fn walk_import<V: AstVisitor>(visitor: &mut V, import: &Import) {
    for item in &import.items {
        // Import items don't have sub-nodes to walk
    }
}

pub fn walk_item<V: AstVisitor>(visitor: &mut V, item: &Item) {
    match item {
        Item::Fn(f) => walk_function(visitor, f),
        Item::Struct(s) => walk_struct(visitor, s),
        Item::Enum(e) => walk_enum(visitor, e),
        Item::Impl(i) => walk_impl(visitor, i),
        Item::TypeAlias(t) => walk_type_alias(visitor, t),
        Item::Const(c) => walk_const(visitor, c),
        Item::Static(s) => walk_static(visitor, s),
        Item::ExternBlock(e) => walk_extern_block(visitor, e),
        Item::Module(m) => visitor.visit_module(m),
    }
}

pub fn walk_function<V: AstVisitor>(visitor: &mut V, func: &Function) {
    for param in &func.params {
        visitor.visit_type(&param.ty);
    }
    if let Some(ret) = &func.return_type {
        visitor.visit_type(ret);
    }
    if let Some(body) = &func.body {
        visitor.visit_block(body);
    }
    for attr in &func.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_struct<V: AstVisitor>(visitor: &mut V, s: &Struct) {
    for field in &s.fields {
        visitor.visit_type(&field.ty);
        for attr in &field.attributes {
            walk_attribute(visitor, attr);
        }
    }
    for attr in &s.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_enum<V: AstVisitor>(visitor: &mut V, e: &Enum) {
    for variant in &e.variants {
        walk_enum_variant(visitor, variant);
    }
    for attr in &e.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_enum_variant<V: AstVisitor>(visitor: &mut V, v: &EnumVariant) {
    match &v.data {
        EnumVariantData::Unit => {}
        EnumVariantData::Tuple(types) => {
            for ty in types {
                visitor.visit_type(ty);
            }
        }
        EnumVariantData::Struct(fields) => {
            for field in fields {
                visitor.visit_type(&field.ty);
                for attr in &field.attributes {
                    walk_attribute(visitor, attr);
                }
            }
        }
    }
    if let Some(disc) = &v.discriminant {
        visitor.visit_expr(disc);
    }
    for attr in &v.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_impl<V: AstVisitor>(visitor: &mut V, i: &Impl) {
    visitor.visit_type(&i.self_ty);
    if let Some(trait_ref) = &i.trait_ref {
        walk_path(visitor, trait_ref);
    }
    for item in &i.items {
        match item {
            ImplItem::Fn(f) => walk_function(visitor, f),
            ImplItem::Const(c) => walk_const(visitor, c),
            ImplItem::TypeAlias(t) => walk_type_alias(visitor, t),
        }
    }
    for attr in &i.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_type_alias<V: AstVisitor>(visitor: &mut V, t: &TypeAlias) {
    visitor.visit_type(&t.ty);
    for attr in &t.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_const<V: AstVisitor>(visitor: &mut V, c: &Const) {
    visitor.visit_type(&c.ty);
    visitor.visit_expr(&c.value);
    for attr in &c.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_static<V: AstVisitor>(visitor: &mut V, s: &Static) {
    visitor.visit_type(&s.ty);
    visitor.visit_expr(&s.value);
    for attr in &s.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_extern_block<V: AstVisitor>(visitor: &mut V, e: &ExternBlock) {
    for item in &e.items {
        match item {
            ExternItem::Fn(f) => walk_function(visitor, f),
            ExternItem::Static(s) => walk_static(visitor, s),
        }
    }
}

pub fn walk_block<V: AstVisitor>(visitor: &mut V, block: &Block) {
    for stmt in &block.stmts {
        visitor.visit_stmt(stmt);
    }
    if let Some(expr) = &block.expr {
        visitor.visit_expr(expr);
    }
}

pub fn walk_stmt<V: AstVisitor>(visitor: &mut V, stmt: &Stmt) {
    match stmt {
        Stmt::Let(l) => {
            if let Some(ty) = &l.ty {
                visitor.visit_type(ty);
            }
            if let Some(init) = &l.init {
                visitor.visit_expr(init);
            }
        }
        Stmt::Expr(e) => visitor.visit_expr(e),
        Stmt::Item(i) => visitor.visit_item(i),
        Stmt::Macro(m) => walk_macro(visitor, m),
    }
}

pub fn walk_expr<V: AstVisitor>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Path(p) => walk_path(visitor, p),
        ExprKind::Unary(_, e) => visitor.visit_expr(e),
        ExprKind::Binary(_, l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::Call(callee, args) => {
            visitor.visit_expr(callee);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::MethodCall(receiver, _, type_args, args) => {
            visitor.visit_expr(receiver);
            for ty in type_args {
                visitor.visit_type(ty);
            }
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::Field(e, _) => visitor.visit_expr(e),
        ExprKind::Index(e, idx) => {
            visitor.visit_expr(e);
            visitor.visit_expr(idx);
        }
        ExprKind::Range(start, end, _) => {
            visitor.visit_expr(start);
            visitor.visit_expr(end);
        }
        ExprKind::Struct(path, fields) => {
            walk_path(visitor, path);
            for f in fields {
                visitor.visit_expr(&f.expr);
            }
        }
        ExprKind::Array(elems) => {
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Tuple(elems) => {
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Block(b) => visitor.visit_block(b),
        ExprKind::If(cond, then_branch, else_branch) => {
            visitor.visit_expr(cond);
            visitor.visit_block(then_branch);
            if let Some(eb) = else_branch {
                visitor.visit_expr(eb);
            }
        }
        ExprKind::Match(scrutinee, arms) => {
            visitor.visit_expr(scrutinee);
            for arm in arms {
                walk_match_arm(visitor, arm);
            }
        }
        ExprKind::Loop(body, _) => visitor.visit_block(body),
        ExprKind::While(cond, body, _) => {
            visitor.visit_expr(cond);
            visitor.visit_block(body);
        }
        ExprKind::For(_, iter, body, _) => {
            visitor.visit_expr(iter);
            visitor.visit_block(body);
        }
        ExprKind::Break(_, e) => {
            if let Some(e) = e {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Return(e) => {
            if let Some(e) = e {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Try(e) => visitor.visit_expr(e),
        ExprKind::Closure(c) => {
            for param in &c.params {
                visitor.visit_type(&param.ty);
            }
            if let Some(ret) = &c.return_type {
                visitor.visit_type(ret);
            }
            visitor.visit_block(&c.body);
        }
        ExprKind::Cast(e, ty) => {
            visitor.visit_expr(e);
            visitor.visit_type(ty);
        }
        ExprKind::Assign(l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::AssignOp(_, l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::Ref(e, _) => visitor.visit_expr(e),
        ExprKind::Deref(e) => visitor.visit_expr(e),
        ExprKind::Group(e) => visitor.visit_expr(e),
        ExprKind::Literal(_) => {}
    }
}

pub fn walk_match_arm<V: AstVisitor>(visitor: &mut V, arm: &MatchArm) {
    walk_pattern(visitor, &arm.pattern);
    if let Some(guard) = &arm.guard {
        visitor.visit_expr(guard);
    }
    visitor.visit_expr(&arm.body);
}

pub fn walk_pattern<V: AstVisitor>(visitor: &mut V, pat: &Pattern) {
    match pat {
        Pattern::Tuple(pats) => {
            for p in pats {
                walk_pattern(visitor, p);
            }
        }
        Pattern::Struct(path, fields) => {
            walk_path(visitor, path);
            for f in fields {
                walk_pattern(visitor, &f.pattern);
            }
        }
        Pattern::Or(pats) => {
            for p in pats {
                walk_pattern(visitor, p);
            }
        }
        Pattern::Ref(p, _) => walk_pattern(visitor, p),
        _ => {}
    }
}

pub fn walk_type<V: AstVisitor>(visitor: &mut V, ty: &Type) {
    match ty {
        Type::Path(p) => walk_path(visitor, p),
        Type::Tuple(types) => {
            for t in types {
                visitor.visit_type(t);
            }
        }
        Type::Array(t, _) => visitor.visit_type(t),
        Type::Slice(t) => visitor.visit_type(t),
        Type::Ref(t, _) => visitor.visit_type(t),
        Type::RawPtr(t, _) => visitor.visit_type(t),
        Type::FnPtr(sig) => {
            for p in &sig.params {
                visitor.visit_type(p);
            }
            visitor.visit_type(&sig.return_type);
        }
        _ => {}
    }
}

pub fn walk_path<V: AstVisitor>(visitor: &mut V, path: &Path) {
    for seg in &path.segments {
        for ty in &seg.type_args {
            visitor.visit_type(ty);
        }
    }
}

pub fn walk_attribute<V: AstVisitor>(visitor: &mut V, attr: &Attribute) {
    walk_path(visitor, &attr.path);
    for arg in &attr.args {
        walk_attribute_arg(visitor, arg);
    }
}

pub fn walk_attribute_arg<V: AstVisitor>(visitor: &mut V, arg: &AttributeArg) {
    match arg {
        AttributeArg::Literal(e) => visitor.visit_expr_kind(e),
        AttributeArg::NameValue(_, e) => visitor.visit_expr(e),
        _ => {}
    }
}

pub fn walk_macro<V: AstVisitor>(visitor: &mut V, mac: &MacroCall) {
    walk_path(visitor, &mac.path);
    for arg in &mac.args {
        walk_macro_arg(visitor, arg);
    }
}

pub fn walk_macro_arg<V: AstVisitor>(visitor: &mut V, arg: &MacroArg) {
    match arg {
        MacroArg::Expr(e) => visitor.visit_expr(e),
        MacroArg::Stmt(s) => visitor.visit_stmt(s),
        MacroArg::Pat(p) => walk_pattern(visitor, p),
        MacroArg::Ty(t) => visitor.visit_type(t),
        MacroArg::Block(b) => visitor.visit_block(b),
    }
}

impl AstVisitor for () {}

impl Expr {
    pub fn visit_expr_kind<V: AstVisitor>(&self, visitor: &mut V) {
        match &self.kind {
            ExprKind::Path(p) => walk_path(visitor, p),
            ExprKind::Unary(_, e) => visitor.visit_expr(e),
            ExprKind::Binary(_, l, r) => {
                visitor.visit_expr(l);
                visitor.visit_expr(r);
            }
            ExprKind::Call(callee, args) => {
                visitor.visit_expr(callee);
                for arg in args {
                    visitor.visit_expr(arg);
                }
            }
            ExprKind::MethodCall(receiver, _, type_args, args) => {
                visitor.visit_expr(receiver);
                for ty in type_args {
                    visitor.visit_type(ty);
                }
                for arg in args {
                    visitor.visit_expr(arg);
                }
            }
            ExprKind::Field(e, _) => visitor.visit_expr(e),
            ExprKind::Index(e, idx) => {
                visitor.visit_expr(e);
                visitor.visit_expr(idx);
            }
            ExprKind::Range(start, end, _) => {
                visitor.visit_expr(start);
                visitor.visit_expr(end);
            }
            ExprKind::Struct(path, fields) => {
                walk_path(visitor, path);
                for f in fields {
                    visitor.visit_expr(&f.expr);
                }
            }
            ExprKind::Array(elems) => {
                for e in elems {
                    visitor.visit_expr(e);
                }
            }
            ExprKind::Tuple(elems) => {
                for e in elems {
                    visitor.visit_expr(e);
                }
            }
            ExprKind::Block(b) => visitor.visit_block(b),
            ExprKind::If(cond, then_branch, else_branch) => {
                visitor.visit_expr(cond);
                visitor.visit_block(then_branch);
                if let Some(eb) = else_branch {
                    visitor.visit_expr(eb);
                }
            }
            ExprKind::Match(scrutinee, arms) => {
                visitor.visit_expr(scrutinee);
                for arm in arms {
                    walk_match_arm(visitor, arm);
                }
            }
            ExprKind::Loop(body, _) => visitor.visit_block(body),
            ExprKind::While(cond, body, _) => {
                visitor.visit_expr(cond);
                visitor.visit_block(body);
            }
            ExprKind::For(_, iter, body, _) => {
                visitor.visit_expr(iter);
                visitor.visit_block(body);
            }
            ExprKind::Break(_, e) => {
                if let Some(e) = e {
                    visitor.visit_expr(e);
                }
            }
            ExprKind::Return(e) => {
                if let Some(e) = e {
                    visitor.visit_expr(e);
                }
            }
            ExprKind::Try(e) => visitor.visit_expr(e),
            ExprKind::Closure(c) => {
                for param in &c.params {
                    visitor.visit_type(&param.ty);
                }
                if let Some(ret) = &c.return_type {
                    visitor.visit_type(ret);
                }
                visitor.visit_block(&c.body);
            }
            ExprKind::Cast(e, ty) => {
                visitor.visit_expr(e);
                visitor.visit_type(ty);
            }
            ExprKind::Assign(l, r) => {
                visitor.visit_expr(l);
                visitor.visit_expr(r);
            }
            ExprKind::AssignOp(_, l, r) => {
                visitor.visit_expr(l);
                visitor.visit_expr(r);
            }
            ExprKind::Ref(e, _) => visitor.visit_expr(e),
            ExprKind::Deref(e) => visitor.visit_expr(e),
            ExprKind::Group(e) => visitor.visit_expr(e),
            _ => {}
        }
    }
}