//! Module AST node

use crate::node::NodeId;
use crate::item::Item;
use crate::import::Import;
use diagnostics::Span;

/// Top-level module
#[derive(Debug, Clone)]
pub struct Module {
    pub id: NodeId,
    pub name: String,
    pub span: Span,
    pub items: Vec<Item>,
    pub imports: Vec<Import>,
}