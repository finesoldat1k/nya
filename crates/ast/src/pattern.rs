//! Pattern AST nodes

use crate::node::NodeId;
use crate::expr::Expr;
use crate::path::Path;
use diagnostics::Span;

/// Patterns for matching
#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Literal(Expr),
    Path(Path),
    Tuple(Vec<Pattern>),
    Struct(Path, Vec<FieldPattern>),
    Or(Vec<Pattern>),
    Rest,
    Ref(Box<Pattern>, crate::expr::Mutability),
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Pattern,
}