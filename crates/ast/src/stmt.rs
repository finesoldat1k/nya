//! Statement AST nodes

use crate::node::NodeId;
use crate::expr::Expr;
use crate::ty::Type;
use crate::pattern::Pattern;
use crate::macro_call::MacroCall;
use diagnostics::Span;

/// Statements
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    Item(crate::item::Item),
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

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub id: NodeId,
    pub span: Span,
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
}