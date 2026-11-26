//! Expression AST nodes

use crate::node::NodeId;
use crate::ty::Type;
use crate::path::Path;
use crate::pattern::Pattern;
use crate::stmt::Block;
use crate::attr::Attribute;
use diagnostics::Span;

/// Expression
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
pub struct Closure {
    pub params: Vec<crate::item::Param>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}