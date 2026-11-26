//! Attribute and macro AST nodes

use crate::node::NodeId;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::pattern::Pattern;
use crate::ty::Type;
use crate::path::Path;
use crate::stmt::Block;
use diagnostics::Span;

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
    Literal(Expr),
    NameValue(String, Expr),
}