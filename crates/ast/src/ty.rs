//! Type AST nodes

use crate::path::Path;

/// Type representation
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive types
    Bool,
    Int(IntTy),
    Float(FloatTy),
    Char,
    Str,
    Never,
    
    // Named types
    Path(Path),
    
    // Compound types
    Tuple(Vec<Type>),
    Array(Box<Type>, Option<usize>), // None = slice
    Slice(Box<Type>),
    Ref(Box<Type>, Mutability),
    RawPtr(Box<Type>, Mutability),
    FnPtr(FnSig),
    
    // Generics
    Generic(String),
    
    // Inference placeholder
    Infer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntTy {
    I8, I16, I32, I64, I128, Isize,
    U8, U16, U32, U64, U128, Usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatTy {
    F32, F64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

/// Function signature for function pointers
#[derive(Debug, Clone, PartialEq)]
pub struct FnSig {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
    pub is_variadic: bool,
    pub abi: Option<String>,
}