//! Path AST nodes

/// Path (for type names, function calls, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub is_absolute: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    pub name: String,
    pub type_args: Vec<crate::ty::Type>,
}