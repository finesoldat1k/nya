//! Macro call AST nodes

use crate::node::NodeId;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::pattern::Pattern;
use crate::ty::Type;
use crate::stmt::Block;
use crate::path::Path;
use diagnostics::Span;

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