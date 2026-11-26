//! Generic parameter AST nodes

use crate::node::NodeId;
use crate::ty::Type;
use diagnostics::Span;

/// Generic type parameter
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub bounds: Vec<Type>,
    pub default: Option<Type>,
}