//! AST node definitions

use diagnostics::Span;
use std::fmt;

/// A unique identifier for AST nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Generate unique node IDs
pub struct NodeIdGen {
    next: usize,
}

impl NodeIdGen {
    pub fn new() -> Self {
        Self { next: 0 }
    }
    
    pub fn next(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next += 1;
        id
    }
}