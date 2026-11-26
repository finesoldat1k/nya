//! Import AST nodes

use crate::node::NodeId;
use diagnostics::Span;

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