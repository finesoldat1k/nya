//! Token definitions and lexer implementation

use diagnostics::{Diagnostic, DiagnosticBag, Label, Severity, Span};
use smallvec::SmallVec;

/// Token types for nya
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Identifiers and literals
    Ident(String),
    Integer(u64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    
    // Keywords
    Fn,
    Struct,
    Enum,
    Impl,
    Type,
    Const,
    Var,
    Pub,
    Static,
    Extern,
    Use,
    Mod,
    As,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Loop,
    Break,
    Continue,
    Match,
    Let,
    Mut,
    Ref,
    Move,
    Where,
    Trait,
    TypeAlias,
    
    // Cute keywords
    Meow,        // constructor
    Nya,         // language name / cute keyword
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Ampersand,
    Pipe,
    Tilde,
    Shl,
    Shr,
    Eq,
    EqEq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
    AndAnd,
    OrOr,
    Bang,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    CaretEq,
    AmpEq,
    PipeEq,
    ShlEq,
    ShrEq,
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    DotDot,
    DotDotEq,
    Colon,
    Semi,
    Arrow,
    FatArrow,
    At,
    Hash,
    Dollar,
    Question,
    DoubleColon,
    
    // Special
    Underscore,
    Ellipsis,
    Exclaim,
    
    // End of file
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
    
    pub fn dummy(kind: TokenKind) -> Self {
        Self { kind, span: Span::dummy() }
    }
}

/// Lexer state
pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    byte_offset: usize,
    char_offset: usize,
    line: usize,
    column: usize,
    file_id: usize,
    diagnostics: DiagnosticBag,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, file_id: usize) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            byte_offset: 0,
            char_offset: 0,
            line: 1,
            column: 1,
            file_id,
            diagnostics: DiagnosticBag::new(),
        }
    }
    
    pub fn tokenize(mut self) -> Result<Vec<Token>, DiagnosticBag> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token();
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        
        if self.diagnostics.has_errors() {
            Err(self.diagnostics)
        } else {
            Ok(tokens)
        }
    }
    
    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        
        let start_byte = self.byte_offset;
        let start_line = self.line;
        let start_col = self.column;
        
        let kind = match self.chars.next() {
            Some(c) => self.lex_token(c),
            None => TokenKind::Eof,
        };
        
        let end_byte = self.byte_offset;
        let span = Span::new(
            self.file_id,
            start_byte,
            end_byte,
            start_line,
            start_col,
        );
        
        Token::new(kind, span)
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.chars.peek() {
                Some(&c) if c.is_whitespace() => {
                    self.consume_char();
                }
                Some(&'/') => {
                    let mut peeked = self.chars.clone();
                    peeked.next();
                    match peeked.next() {
                        Some('/') => {
                            self.consume_char();
                            self.consume_char();
                            while let Some(&c) = self.chars.peek() {
                                if c == '\n' { break; }
                                self.consume_char();
                            }
                        }
                        Some('*') => {
                            self.consume_char();
                            self.consume_char();
                            let mut depth = 1;
                            while depth > 0 {
                                match self.chars.next() {
                                    Some('*') => {
                                        if matches!(self.chars.peek(), Some(&'/')) {
                                            self.consume_char();
                                            depth -= 1;
                                        }
                                    }
                                    Some('/') => {
                                        if matches!(self.chars.peek(), Some(&'*')) {
                                            self.consume_char();
                                            depth += 1;
                                        }
                                    }
                                    Some('\n') => {
                                        self.line += 1;
                                        self.column = 1;
                                    }
                                    None => break,
                                    _ => {}
                                }
                            }
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }
    }
    
    fn consume_char(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.byte_offset += c.len_utf8();
        self.char_offset += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }
    
    fn lex_token(&mut self, first: char) -> TokenKind {
        match first {
            'a'..='z' | 'A'..='Z' | '_' => self.lex_ident_or_keyword(first),
            '0'..='9' => self.lex_number(first),
            '"' => self.lex_string(),
            '\'' => self.lex_char(),
            '+' => self.lex_op('=', TokenKind::Plus, TokenKind::PlusEq),
            '-' => self.lex_arrow_or_op(TokenKind::Minus, TokenKind::MinusEq, TokenKind::Arrow),
            '*' => self.lex_op('=', TokenKind::Star, TokenKind::StarEq),
            '/' => self.lex_op('=', TokenKind::Slash, TokenKind::SlashEq),
            '%' => self.lex_op('=', TokenKind::Percent, TokenKind::PercentEq),
            '^' => self.lex_op('=', TokenKind::Caret, TokenKind::CaretEq),
            '&' => self.lex_double('&', TokenKind::Ampersand, TokenKind::AndAnd, TokenKind::AmpEq),
            '|' => self.lex_double('|', TokenKind::Pipe, TokenKind::OrOr, TokenKind::PipeEq),
            '~' => TokenKind::Tilde,
            '=' => self.lex_eq(),
            '!' => self.lex_op('=', TokenKind::Bang, TokenKind::NotEq),
            '<' => self.lex_shift_or_compare(),
            '>' => self.lex_shift_or_compare_gt(),
            '.' => self.lex_dot(),
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semi,
            ':' => self.lex_colon(),
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '@' => TokenKind::At,
            '#' => TokenKind::Hash,
            '$' => TokenKind::Dollar,
            '?' => TokenKind::Question,
            _ => {
                self.diagnostics.push(Diagnostic::error(format!("unexpected character: '{}'", first))
                    .with_label(Label::primary(Span::dummy(), "unexpected character")));
                TokenKind::Eof
            }
        }
    }
    
    fn lex_ident_or_keyword(&mut self, first: char) -> TokenKind {
        let mut ident = String::new();
        ident.push(first);
        
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                ident.push(self.consume_char().unwrap());
            } else {
                break;
            }
        }
        
        match ident.as_str() {
            "fn" => TokenKind::Fn,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "impl" => TokenKind::Impl,
            "type" => TokenKind::Type,
            "const" => TokenKind::Const,
            "var" => TokenKind::Var,
            "pub" => TokenKind::Pub,
            "static" => TokenKind::Static,
            "extern" => TokenKind::Extern,
            "use" => TokenKind::Use,
            "mod" => TokenKind::Mod,
            "as" => TokenKind::As,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "loop" => TokenKind::Loop,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "match" => TokenKind::Match,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "ref" => TokenKind::Ref,
            "move" => TokenKind::Move,
            "where" => TokenKind::Where,
            "trait" => TokenKind::Trait,
            "meow" => TokenKind::Meow,
            "nya" => TokenKind::Nya,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Ident(ident),
        }
    }
    
    fn lex_number(&mut self, first: char) -> TokenKind {
        let mut num = String::new();
        num.push(first);
        let mut is_float = false;
        
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' { num.push(c); }
                self.consume_char();
            } else if c == '.' && !is_float {
                let mut peeked = self.chars.clone();
                peeked.next();
                if matches!(peeked.peek(), Some(&('0'..='9'))) {
                    is_float = true;
                    num.push(self.consume_char().unwrap());
                } else {
                    break;
                }
            } else if (c == 'e' || c == 'E') && !is_float {
                is_float = true;
                num.push(self.consume_char().unwrap());
                if matches!(self.chars.peek(), Some(&'+') | Some(&'-')) {
                    num.push(self.consume_char().unwrap());
                }
            } else {
                break;
            }
        }
        
        if is_float {
            TokenKind::Float(num.parse().unwrap_or(0.0))
        } else {
            TokenKind::Integer(num.parse().unwrap_or(0))
        }
    }
    
    fn lex_string(&mut self) -> TokenKind {
        let mut result = String::new();
        let mut escaped = false;
        
        while let Some(c) = self.consume_char() {
            if escaped {
                match c {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    '0' => result.push('\0'),
                    'x' => {
                        let h1 = self.consume_char().unwrap_or('0');
                        let h2 = self.consume_char().unwrap_or('0');
                        let val = u8::from_str_radix(&format!("{}{}", h1, h2), 16).unwrap_or(0);
                        result.push(val as char);
                    }
                    'u' => {
                        if matches!(self.chars.peek(), Some(&'{')) {
                            self.consume_char();
                            let mut hex = String::new();
                            while let Some(&c) = self.chars.peek() {
                                if c == '}' { break; }
                                if c.is_ascii_hexdigit() {
                                    hex.push(self.consume_char().unwrap());
                                } else { break; }
                            }
                            self.consume_char();
                            if let Ok(val) = u32::from_str_radix(&hex, 16) {
                                if let Some(ch) = char::from_u32(val) {
                                    result.push(ch);
                                }
                            }
                        }
                    }
                    _ => result.push(c),
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                break;
            } else {
                result.push(c);
            }
        }
        
        TokenKind::String(result)
    }
    
    fn lex_char(&mut self) -> TokenKind {
        let mut result = '\0';
        let mut escaped = false;
        
        while let Some(c) = self.consume_char() {
            if escaped {
                match c {
                    'n' => result = '\n',
                    't' => result = '\t',
                    'r' => result = '\r',
                    '\\' => result = '\\',
                    '"' => result = '"',
                    '\'' => result = '\'',
                    '0' => result = '\0',
                    _ => result = c,
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '\'' {
                break;
            } else {
                result = c;
            }
        }
        
        TokenKind::Char(result)
    }
    
    fn lex_op(&mut self, eq_char: char, single: TokenKind, double: TokenKind) -> TokenKind {
        if matches!(self.chars.peek(), Some(&c) if c == eq_char) {
            self.consume_char();
            double
        } else {
            single
        }
    }
    
    fn lex_arrow_or_op(&mut self, single: TokenKind, eq: TokenKind, arrow: TokenKind) -> TokenKind {
        if matches!(self.chars.peek(), Some(&'>')) {
            self.consume_char();
            arrow
        } else if matches!(self.chars.peek(), Some(&'=')) {
            self.consume_char();
            eq
        } else {
            single
        }
    }
    
    fn lex_double(&mut self, second: char, single: TokenKind, double: TokenKind, eq: TokenKind) -> TokenKind {
        if matches!(self.chars.peek(), Some(&c) if c == second) {
            self.consume_char();
            double
        } else if matches!(self.chars.peek(), Some(&'=')) {
            self.consume_char();
            eq
        } else {
            single
        }
    }
    
    fn lex_eq(&mut self) -> TokenKind {
        if matches!(self.chars.peek(), Some(&'=')) {
            self.consume_char();
            TokenKind::EqEq
        } else if matches!(self.chars.peek(), Some(&'>')) {
            self.consume_char();
            TokenKind::FatArrow
        } else {
            TokenKind::Eq
        }
    }
    
    fn lex_shift_or_compare(&mut self) -> TokenKind {
        if matches!(self.chars.peek(), Some(&'<')) {
            self.consume_char();
            if matches!(self.chars.peek(), Some(&'=')) {
                self.consume_char();
                TokenKind::ShlEq
            } else {
                TokenKind::Shl
            }
        } else if matches!(self.chars.peek(), Some(&'=')) {
            self.consume_char();
            TokenKind::Le
        } else {
            TokenKind::Lt
        }
    }
    
    fn lex_shift_or_compare_gt(&mut self) -> TokenKind {
        if matches!(self.chars.peek(), Some(&'>')) {
            self.consume_char();
            if matches!(self.chars.peek(), Some(&'=')) {
                self.consume_char();
                TokenKind::ShrEq
            } else {
                TokenKind::Shr
            }
        } else if matches!(self.chars.peek(), Some(&'=')) {
            self.consume_char();
            TokenKind::Ge
        } else {
            TokenKind::Gt
        }
    }
    
    fn lex_dot(&mut self) -> TokenKind {
        if matches!(self.chars.peek(), Some(&'.')) {
            self.consume_char();
            if matches!(self.chars.peek(), Some(&'=')) {
                self.consume_char();
                TokenKind::DotDotEq
            } else if matches!(self.chars.peek(), Some(&'.')) {
                self.consume_char();
                TokenKind::Ellipsis
            } else {
                TokenKind::DotDot
            }
        } else {
            TokenKind::Dot
        }
    }
    
    fn lex_colon(&mut self) -> TokenKind {
        if matches!(self.chars.peek(), Some(&':')) {
            self.consume_char();
            TokenKind::DoubleColon
        } else {
            TokenKind::Colon
        }
    }
}

/// Public tokenizer function
pub fn tokenize(source: &str, filename: &str) -> Result<Vec<Token>, DiagnosticBag> {
    let lexer = Lexer::new(source, 0);
    lexer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_tokens() {
        let source = "fn main() { var x = 42; }";
        let tokens = tokenize(source, "test.nya").unwrap();
        assert!(tokens.len() > 0);
    }
    
    #[test]
    fn test_keywords() {
        let source = "fn struct enum impl const var pub static extern use mod as return if else while for in loop break continue match let mut ref move where trait";
        let tokens = tokenize(source, "test.nya").unwrap();
        let keywords: Vec<_> = tokens.iter().filter_map(|t| match &t.kind {
            TokenKind::Ident(_) => None,
            _ => Some(t.kind.clone()),
        }).collect();
        assert_eq!(keywords.len(), 27);
    }
    
    #[test]
    fn test_cute_keywords() {
        let source = "meow nya";
        let tokens = tokenize(source, "test.nya").unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Meow));
        assert!(matches!(tokens[1].kind, TokenKind::Nya));
    }
}