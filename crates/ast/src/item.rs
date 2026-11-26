//! Top-level AST items

use crate::node::NodeId;
use crate::expr::Expr;
use crate::stmt::Block;
use crate::ty::Type;
use crate::path::Path;
use crate::attr::Attribute;
use crate::generics::GenericParam;
use crate::pattern::Pattern;
use diagnostics::Span;

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

/// Module definition
pub use crate::module::Module;