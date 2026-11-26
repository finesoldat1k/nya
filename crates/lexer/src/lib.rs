//! Lexer for the nya programming language

pub mod token;

use diagnostics::{Diagnostic, DiagnosticBag, Label, Severity, Span};
use smallvec::SmallVec;

pub use crate::token::{Token, TokenKind, tokenize};