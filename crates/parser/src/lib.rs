//! Parser for the nya programming language

use crate::ast::*;
use lexer::{Token, TokenKind};
use diagnostics::{Diagnostic, DiagnosticBag, Label, Severity, Span};
use smallvec::SmallVec;

/// Parser state
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    file_id: usize,
    diagnostics: DiagnosticBag,
    next_node_id: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file_id: usize) -> Self {
        Self {
            tokens,
            current: 0,
            file_id,
            diagnostics: DiagnosticBag::new(),
            next_node_id: 0,
        }
    }
    
    pub fn parse(mut self) -> Result<Module, DiagnosticBag> {
        let module = self.parse_module();
        
        if self.diagnostics.has_errors() {
            Err(self.diagnostics)
        } else {
            Ok(module)
        }
    }
    
    fn next_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }
    
    fn parse_module(&mut self) -> Module {
        let start_span = self.current_span();
        let mut items = Vec::new();
        let mut imports = Vec::new();
        
        while !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::Use) {
                imports.push(self.parse_import());
            } else {
                items.push(self.parse_item());
            }
        }
        
        Module {
            id: self.next_node_id(),
            name: "main".to_string(),
            span: start_span,
            items,
            imports,
        }
    }
    
    fn parse_import(&mut self) -> Import {
        let start = self.current_span();
        self.expect(TokenKind::Use);
        
        let mut path = Vec::new();
        let mut alias = None;
        let mut items = Vec::new();
        
        while self.check(&TokenKind::Ident("".to_string())) || self.check(&TokenKind::DoubleColon) {
            if self.check(&TokenKind::DoubleColon) {
                self.advance();
            }
            if let TokenKind::Ident(name) = &self.current_token().kind {
                path.push(name.clone());
                self.advance();
            } else {
                break;
            }
        }
        
        if self.check(&TokenKind::LBrace) {
            self.advance();
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                if let TokenKind::Ident(name) = &self.current_token().kind {
                    let item_name = name.clone();
                    self.advance();
                    
                    let item_alias = if self.check(&TokenKind::As) {
                        self.advance();
                        if let TokenKind::Ident(alias) = &self.current_token().kind {
                            let a = alias.clone();
                            self.advance();
                            Some(a)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    items.push(ImportItem {
                        id: self.next_node_id(),
                        span: self.current_span(),
                        name: item_name,
                        alias: item_alias,
                    });
                    
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RBrace);
        }
        
        self.expect(TokenKind::Semi);
        
        Import {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            path,
            alias,
            items,
        }
    }
    
    fn parse_item(&mut self) -> Item {
        let attrs = self.parse_attributes();
        let is_pub = self.parse_visibility();
        
        if self.check(&TokenKind::Extern) {
            return self.parse_extern_block(attrs, is_pub);
        }
        
        if self.check(&TokenKind::Mod) {
            return self.parse_module_item(attrs, is_pub);
        }
        
        if self.check(&TokenKind::Fn) {
            return Item::Fn(self.parse_function(attrs, is_pub, false));
        }
        
        if self.check(&TokenKind::Struct) {
            return Item::Struct(self.parse_struct(attrs, is_pub));
        }
        
        if self.check(&TokenKind::Enum) {
            return Item::Enum(self.parse_enum(attrs, is_pub));
        }
        
        if self.check(&TokenKind::Impl) {
            return Item::Impl(self.parse_impl(attrs));
        }
        
        if self.check(&TokenKind::Type) {
            return Item::TypeAlias(self.parse_type_alias(attrs, is_pub));
        }
        
        if self.check(&TokenKind::Const) {
            return Item::Const(self.parse_const(attrs, is_pub));
        }
        
        if self.check(&TokenKind::Static) {
            return Item::Static(self.parse_static(attrs, is_pub));
        }
        
        self.error("expected item");
        self.synchronize();
        Item::Fn(Function {
            id: self.next_node_id(),
            span: self.current_span(),
            name: "<error>".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: None,
            body: None,
            is_pub: false,
            is_static: false,
            is_extern: false,
            abi: None,
            attributes: Vec::new(),
        })
    }
    
    fn parse_extern_block(&mut self, attrs: Vec<Attribute>, is_pub: bool) -> Item {
        self.expect(TokenKind::Extern);
        
        let abi = if self.check(&TokenKind::String("".to_string())) {
            if let TokenKind::String(s) = &self.current_token().kind {
                let abi = s.clone();
                self.advance();
                abi
            } else {
                "C".to_string()
            }
        } else {
            "C".to_string()
        };
        
        self.expect(TokenKind::LBrace);
        let mut items = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let attrs = self.parse_attributes();
            let is_pub = self.parse_visibility();
            
            if self.check(&TokenKind::Fn) {
                items.push(ExternItem::Fn(self.parse_function(attrs, is_pub, true)));
            } else if self.check(&TokenKind::Static) {
                items.push(ExternItem::Static(self.parse_static(attrs, is_pub)));
            } else {
                self.error("expected function or static in extern block");
                self.synchronize();
            }
        }
        
        self.expect(TokenKind::RBrace);
        
        Item::ExternBlock(ExternBlock {
            id: self.next_node_id(),
            span: self.current_span(),
            abi,
            items,
        })
    }
    
    fn parse_module_item(&mut self, attrs: Vec<Attribute>, is_pub: bool) -> Item {
        self.expect(TokenKind::Mod);
        let name = self.expect_ident();
        self.expect(TokenKind::LBrace);
        
        let mut items = Vec::new();
        let mut imports = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::Use) {
                imports.push(self.parse_import());
            } else {
                items.push(self.parse_item());
            }
        }
        
        self.expect(TokenKind::RBrace);
        
        Item::Module(Module {
            id: self.next_node_id(),
            name,
            span: self.current_span(),
            items,
            imports,
        })
    }
    
    fn parse_function(&mut self, attrs: Vec<Attribute>, is_pub: bool, is_extern: bool) -> Function {
        let start = self.current_span();
        self.expect(TokenKind::Fn);
        
        let name = if self.check(&TokenKind::Meow) {
            self.advance();
            "meow".to_string()
        } else {
            self.expect_ident()
        };
        
        let generics = self.parse_generics();
        self.expect(TokenKind::LParen);
        let params = self.parse_params();
        self.expect(TokenKind::RParen);
        
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };
        
        let body = if self.check(&TokenKind::LBrace) {
            Some(self.parse_block())
        } else if is_extern {
            self.expect(TokenKind::Semi);
            None
        } else {
            self.expect(TokenKind::LBrace);
            Some(self.parse_block())
        };
        
        Function {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            name,
            generics,
            params,
            return_type,
            body,
            is_pub,
            is_static: false,
            is_extern,
            abi: None,
            attributes: attrs,
        }
    }
    
    fn parse_struct(&mut self, attrs: Vec<Attribute>, is_pub: bool) -> Struct {
        let start = self.current_span();
        self.expect(TokenKind::Struct);
        let name = self.expect_ident();
        let generics = self.parse_generics();
        
        self.expect(TokenKind::LBrace);
        let mut fields = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let field_attrs = self.parse_attributes();
            let is_field_pub = self.parse_visibility();
            
            let name = self.expect_ident();
            self.expect(TokenKind::Colon);
            let ty = self.parse_type();
            
            fields.push(StructField {
                id: self.next_node_id(),
                span: self.current_span(),
                name,
                ty,
                is_pub: is_field_pub,
                attributes: field_attrs,
            });
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.expect(TokenKind::RBrace);
        
        Struct {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            name,
            generics,
            fields,
            is_pub,
            attributes: attrs,
        }
    }
    
    fn parse_enum(&mut self, attrs: Vec<Attribute>, is_pub: bool) -> Enum {
        let start = self.current_span();
        self.expect(TokenKind::Enum);
        let name = self.expect_ident();
        let generics = self.parse_generics();
        
        self.expect(TokenKind::LBrace);
        let mut variants = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let variant_attrs = self.parse_attributes();
            let variant_name = self.expect_ident();
            
            let data = if self.check(&TokenKind::LParen) {
                self.advance();
                let mut types = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                    types.push(self.parse_type());
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.expect(TokenKind::RParen);
                EnumVariantData::Tuple(types)
            } else if self.check(&TokenKind::LBrace) {
                self.advance();
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                    let field_attrs = self.parse_attributes();
                    let is_field_pub = self.parse_visibility();
                    let name = self.expect_ident();
                    self.expect(TokenKind::Colon);
                    let ty = self.parse_type();
                    fields.push(StructField {
                        id: self.next_node_id(),
                        span: self.current_span(),
                        name,
                        ty,
                        is_pub: is_field_pub,
                        attributes: field_attrs,
                    });
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.expect(TokenKind::RBrace);
                EnumVariantData::Struct(fields)
            } else {
                EnumVariantData::Unit
            };
            
            let discriminant = if self.check(&TokenKind::Eq) {
                self.advance();
                Some(self.parse_expr())
            } else {
                None
            };
            
            variants.push(EnumVariant {
                id: self.next_node_id(),
                span: self.current_span(),
                name: variant_name,
                data,
                discriminant,
                attributes: variant_attrs,
            });
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.expect(TokenKind::RBrace);
        
        Enum {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            name,
            generics,
            variants,
            is_pub,
            attributes: attrs,
        }
    }
    
    fn parse_impl(&mut self, attrs: Vec<Attribute>) -> Impl {
        let start = self.current_span();
        self.expect(TokenKind::Impl);
        let generics = self.parse_generics();
        
        let trait_ref = if self.check(&TokenKind::Ident("for".to_string())) {
            None
        } else if self.check(&TokenKind::Ident("".to_string())) {
            let path = self.parse_path();
            if self.check(&TokenKind::Ident("for".to_string())) {
                self.advance();
                let self_ty = self.parse_type();
                return Impl {
                    id: self.next_node_id(),
                    span: start.merge(self.previous_span()),
                    generics,
                    trait_ref: Some(path),
                    self_ty,
                    items: Vec::new(),
                    attributes: attrs,
                };
            }
            let self_ty = Type::Path(path);
            None
        } else {
            None
        };
        
        let self_ty = if trait_ref.is_some() {
            self.parse_type()
        } else if let Some(ty) = self.parse_type_opt() {
            ty
        } else {
            self.error("expected type in impl block");
            Type::Infer
        };
        
        self.expect(TokenKind::LBrace);
        let mut items = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let attrs = self.parse_attributes();
            let is_pub = self.parse_visibility();
            
            if self.check(&TokenKind::Fn) {
                items.push(ImplItem::Fn(self.parse_function(attrs, is_pub, false)));
            } else if self.check(&TokenKind::Const) {
                items.push(ImplItem::Const(self.parse_const(attrs, is_pub)));
            } else if self.check(&TokenKind::Type) {
                items.push(ImplItem::TypeAlias(self.parse_type_alias(attrs, is_pub)));
            } else {
                self.error("expected function, const, or type in impl block");
                self.synchronize();
            }
        }
        
        self.expect(TokenKind::RBrace);
        
        Impl {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            generics,
            trait_ref,
            self_ty,
            items,
            attributes: attrs,
        }
    }
    
    fn parse_type_alias(&mut self, attrs: Vec<Attribute>, is_pub: bool) -> TypeAlias {
        let start = self.current_span();
        self.expect(TokenKind::Type);
        let name = self.expect_ident();
        let generics = self.parse_generics();
        self.expect(TokenKind::Eq);
        let ty = self.parse_type();
        self.expect(TokenKind::Semi);
        
        TypeAlias {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            name,
            generics,
            ty,
            is_pub,
            attributes: attrs,
        }
    }
    
    fn parse_const(&mut self, attrs: Vec<Attribute>, is_pub: bool) -> Const {
        let start = self.current_span();
        self.expect(TokenKind::Const);
        let name = self.expect_ident();
        self.expect(TokenKind::Colon);
        let ty = self.parse_type();
        self.expect(TokenKind::Eq);
        let value = self.parse_expr();
        self.expect(TokenKind::Semi);
        
        Const {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            name,
            ty,
            value,
            is_pub,
            attributes: attrs,
        }
    }
    
    fn parse_static(&mut self, attrs: Vec<Attribute>, is_pub: bool) -> Static {
        let start = self.current_span();
        self.expect(TokenKind::Static);
        let is_mut = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };
        let name = self.expect_ident();
        self.expect(TokenKind::Colon);
        let ty = self.parse_type();
        self.expect(TokenKind::Eq);
        let value = self.parse_expr();
        self.expect(TokenKind::Semi);
        
        Static {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            name,
            ty,
            value,
            is_pub,
            is_mut,
            attributes: attrs,
        }
    }
    
    fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        
        while self.check(&TokenKind::Hash) {
            self.advance();
            self.expect(TokenKind::LBracket);
            let path = self.parse_path();
            let mut args = Vec::new();
            
            if self.check(&TokenKind::LParen) {
                self.advance();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                    if self.check(&TokenKind::Ident("".to_string())) {
                        let name = self.expect_ident();
                        if self.check(&TokenKind::Eq) {
                            self.advance();
                            let value = self.parse_expr();
                            args.push(AttributeArg::NameValue(name, value));
                        } else {
                            args.push(AttributeArg::Ident(name));
                        }
                    } else {
                        args.push(AttributeArg::Literal(self.parse_expr_kind()));
                    }
                    
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.expect(TokenKind::RParen);
            }
            
            self.expect(TokenKind::RBracket);
            
            attrs.push(Attribute {
                id: self.next_node_id(),
                span: self.current_span(),
                path,
                args,
            });
        }
        
        attrs
    }
    
    fn parse_visibility(&mut self) -> bool {
        if self.check(&TokenKind::Pub) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn parse_generics(&mut self) -> Vec<GenericParam> {
        if !self.check(&TokenKind::Lt) {
            return Vec::new();
        }
        
        self.advance();
        let mut generics = Vec::new();
        
        while !self.check(&TokenKind::Gt) && !self.check(&TokenKind::Eof) {
            let start = self.current_span();
            let name = self.expect_ident();
            
            let bounds = if self.check(&TokenKind::Colon) {
                self.advance();
                let mut bounds = Vec::new();
                bounds.push(self.parse_type());
                while self.check(&TokenKind::Plus) {
                    self.advance();
                    bounds.push(self.parse_type());
                }
                bounds
            } else {
                Vec::new()
            };
            
            let default = if self.check(&TokenKind::Eq) {
                self.advance();
                Some(self.parse_type())
            } else {
                None
            };
            
            generics.push(GenericParam {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                name,
                bounds,
                default,
            });
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.expect(TokenKind::Gt);
        generics
    }
    
    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        
        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
            let start = self.current_span();
            let is_mut = if self.check(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };
            
            let is_ref = if self.check(&TokenKind::Ampersand) {
                self.advance();
                let _mut = if self.check(&TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                true
            } else {
                false
            };
            
            let name = self.expect_ident();
            self.expect(TokenKind::Colon);
            let ty = self.parse_type();
            
            params.push(Param {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                name,
                ty,
                is_mut,
                is_ref,
            });
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        params
    }
    
    fn parse_block(&mut self) -> Block {
        let start = self.current_span();
        self.expect(TokenKind::LBrace);
        let mut stmts = Vec::new();
        let mut expr = None;
        
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::Semi) {
                self.advance();
                continue;
            }
            
            let is_last = !self.check(&TokenKind::RBrace) && 
                          (self.check(&TokenKind::RBrace) || 
                           (self.peek_is_terminator() && !self.check(&TokenKind::Semi)));
            
            let stmt = self.parse_stmt();
            
            if is_last && matches!(stmt, Stmt::Expr(_)) {
                if let Stmt::Expr(e) = stmt {
                    expr = Some(Box::new(e));
                    break;
                }
            }
            
            stmts.push(stmt);
        }
        
        self.expect(TokenKind::RBrace);
        
        Block {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            stmts,
            expr,
        }
    }
    
    fn parse_stmt(&mut self) -> Stmt {
        let attrs = self.parse_attributes();
        
        if self.check(&TokenKind::Let) {
            return self.parse_let_stmt(attrs);
        }
        
        if self.check(&TokenKind::Fn) || self.check(&TokenKind::Struct) || self.check(&TokenKind::Enum) 
            || self.check(&TokenKind::Impl) || self.check(&TokenKind::Type) 
            || self.check(&TokenKind::Const) || self.check(&TokenKind::Static)
            || self.check(&TokenKind::Extern) || self.check(&TokenKind::Mod) {
            return Stmt::Item(self.parse_item());
        }
        
        if self.check(&TokenKind::Ident("".to_string())) && self.peek_is_macro() {
            return Stmt::Macro(self.parse_macro());
        }
        
        let expr = self.parse_expr();
        
        if self.check(&TokenKind::Semi) {
            self.advance();
            Stmt::Expr(expr)
        } else {
            Stmt::Expr(expr)
        }
    }
    
    fn parse_let_stmt(&mut self, attrs: Vec<Attribute>) -> Stmt {
        let start = self.current_span();
        self.expect(TokenKind::Let);
        
        let is_mut = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };
        
        let is_ref = if self.check(&TokenKind::Ampersand) {
            self.advance();
            let _mut = if self.check(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };
            true
        } else {
            false
        };
        
        let name = self.expect_ident();
        
        let ty = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };
        
        let init = if self.check(&TokenKind::Eq) {
            self.advance();
            Some(self.parse_expr())
        } else {
            None
        };
        
        self.expect(TokenKind::Semi);
        
        Stmt::Let(LetStmt {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            name,
            ty,
            init,
            is_mut,
            is_ref,
        })
    }
    
    fn parse_expr(&mut self) -> Expr {
        self.parse_expr_prec(Precedence::Lowest)
    }
    
    fn parse_expr_kind(&mut self) -> ExprKind {
        self.parse_expr().kind
    }
    
    // Precedence climbing parser
    fn parse_expr_prec(&mut self, prec: Precedence) -> Expr {
        let mut lhs = self.parse_prefix();
        
        while prec < self.current_precedence() {
            lhs = self.parse_infix(lhs);
        }
        
        lhs
    }
    
    fn parse_prefix(&mut self) -> Expr {
        let start = self.current_span();
        
        if let TokenKind::Integer(val) = self.current_token().kind.clone() {
            self.advance();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Integer(val),
                ty: None,
            };
        }
        
        if let TokenKind::Float(val) = self.current_token().kind.clone() {
            self.advance();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Float(val),
                ty: None,
            };
        }
        
        if let TokenKind::String(val) = self.current_token().kind.clone() {
            self.advance();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::String(val),
                ty: None,
            };
        }
        
        if let TokenKind::Char(val) = self.current_token().kind.clone() {
            self.advance();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Char(val),
                ty: None,
            };
        }
        
        if let TokenKind::Bool(val) = self.current_token().kind.clone() {
            self.advance();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Bool(val),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::Ident("".to_string())) || self.check(&TokenKind::DoubleColon) {
            let path = self.parse_path();
            
            if self.check(&TokenKind::LBrace) {
                return self.parse_struct_expr(path, start);
            }
            
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Path(path),
                ty: None,
            };
        }
        
        if let Some(op) = self.parse_unary_op() {
            self.advance();
            let expr = self.parse_expr_prec(Precedence::Prefix);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Unary(op, Box::new(expr)),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::LParen) {
            self.advance();
            let expr = self.parse_expr();
            self.expect(TokenKind::RParen);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Group(Box::new(expr)),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::LBrace) {
            let block = self.parse_block();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Block(block),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::If) {
            return self.parse_if_expr(start);
        }
        
        if self.check(&TokenKind::Match) {
            return self.parse_match_expr(start);
        }
        
        if self.check(&TokenKind::Loop) {
            return self.parse_loop_expr(start);
        }
        
        if self.check(&TokenKind::While) {
            return self.parse_while_expr(start);
        }
        
        if self.check(&TokenKind::For) {
            return self.parse_for_expr(start);
        }
        
        if self.check(&TokenKind::Return) {
            self.advance();
            let value = if self.check(&TokenKind::Semi) || self.check(&TokenKind::RBrace) {
                None
            } else {
                Some(Box::new(self.parse_expr()))
            };
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Return(value),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::Break) {
            self.advance();
            let label = if self.check(&TokenKind::Ident("".to_string())) {
                Some(self.expect_ident())
            } else {
                None
            };
            let value = if self.check(&TokenKind::Semi) || self.check(&TokenKind::RBrace) {
                None
            } else {
                Some(Box::new(self.parse_expr()))
            };
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Break(label, value),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::Continue) {
            self.advance();
            let label = if self.check(&TokenKind::Ident("".to_string())) {
                Some(self.expect_ident())
            } else {
                None
            };
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Continue(label),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::LBracket) {
            self.advance();
            let mut elems = Vec::new();
            while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
                elems.push(self.parse_expr());
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect(TokenKind::RBracket);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Array(elems),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::Pipe) {
            return self.parse_closure(start);
        }
        
        self.error("expected expression");
        self.synchronize();
        Expr {
            id: self.next_node_id(),
            span: start,
            kind: ExprKind::Integer(0),
            ty: None,
        }
    }
    
    fn parse_struct_expr(&mut self, path: Path, start: Span) -> Expr {
        self.expect(TokenKind::LBrace);
        let mut fields = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let name = self.expect_ident();
            self.expect(TokenKind::Colon);
            let expr = self.parse_expr();
            fields.push(FieldExpr { name, expr });
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.expect(TokenKind::RBrace);
        
        Expr {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            kind: ExprKind::Struct(path, fields),
            ty: None,
        }
    }
    
    fn parse_if_expr(&mut self, start: Span) -> Expr {
        self.expect(TokenKind::If);
        let cond = self.parse_expr();
        let then_branch = self.parse_block();
        
        let else_branch = if self.check(&TokenKind::Else) {
            self.advance();
            if self.check(&TokenKind::If) {
                Some(Box::new(self.parse_if_expr(self.current_span())))
            } else {
                Some(Box::new(Expr {
                    id: self.next_node_id(),
                    span: self.current_span(),
                    kind: ExprKind::Block(self.parse_block()),
                    ty: None,
                }))
            }
        } else {
            None
        };
        
        Expr {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            kind: ExprKind::If(Box::new(cond), then_branch, else_branch),
            ty: None,
        }
    }
    
    fn parse_match_expr(&mut self, start: Span) -> Expr {
        self.expect(TokenKind::Match);
        let scrutinee = self.parse_expr();
        self.expect(TokenKind::LBrace);
        
        let mut arms = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let arm_start = self.current_span();
            let pattern = self.parse_pattern();
            
            let guard = if self.check(&TokenKind::If) {
                self.advance();
                Some(self.parse_expr())
            } else {
                None
            };
            
            self.expect(TokenKind::FatArrow);
            let body = self.parse_expr();
            
            arms.push(MatchArm {
                id: self.next_node_id(),
                span: arm_start.merge(self.previous_span()),
                pattern,
                guard,
                body,
            });
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.expect(TokenKind::RBrace);
        
        Expr {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            kind: ExprKind::Match(Box::new(scrutinee), arms),
            ty: None,
        }
    }
    
    fn parse_loop_expr(&mut self, start: Span) -> Expr {
        self.expect(TokenKind::Loop);
        
        let label = if self.check(&TokenKind::Ident("".to_string())) && self.peek_is(&TokenKind::Colon) {
            let label = self.expect_ident();
            self.expect(TokenKind::Colon);
            Some(label)
        } else {
            None
        };
        
        let body = self.parse_block();
        
        Expr {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            kind: ExprKind::Loop(body, label),
            ty: None,
        }
    }
    
    fn parse_while_expr(&mut self, start: Span) -> Expr {
        self.expect(TokenKind::While);
        let cond = self.parse_expr();
        
        let label = if self.check(&TokenKind::Ident("".to_string())) && self.peek_is(&TokenKind::Colon) {
            let label = self.expect_ident();
            self.expect(TokenKind::Colon);
            Some(label)
        } else {
            None
        };
        
        let body = self.parse_block();
        
        Expr {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            kind: ExprKind::While(Box::new(cond), body, label),
            ty: None,
        }
    }
    
    fn parse_for_expr(&mut self, start: Span) -> Expr {
        self.expect(TokenKind::For);
        let var = self.expect_ident();
        self.expect(TokenKind::In);
        let iter = self.parse_expr();
        
        let label = if self.check(&TokenKind::Ident("".to_string())) && self.peek_is(&TokenKind::Colon) {
            let label = self.expect_ident();
            self.expect(TokenKind::Colon);
            Some(label)
        } else {
            None
        };
        
        let body = self.parse_block();
        
        Expr {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            kind: ExprKind::For(var, Box::new(iter), body, label),
            ty: None,
        }
    }
    
    fn parse_closure(&mut self, start: Span) -> Expr {
        self.expect(TokenKind::Pipe);
        let mut params = Vec::new();
        
        while !self.check(&TokenKind::Pipe) && !self.check(&TokenKind::Eof) {
            let start = self.current_span();
            let is_mut = if self.check(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };
            
            let name = self.expect_ident();
            let ty = if self.check(&TokenKind::Colon) {
                self.advance();
                Some(self.parse_type())
            } else {
                None
            };
            
            params.push(Param {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                name,
                ty: ty.unwrap_or(Type::Infer),
                is_mut,
                is_ref: false,
            });
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.expect(TokenKind::Pipe);
        
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };
        
        let body = if self.check(&TokenKind::LBrace) {
            self.parse_block()
        } else {
            let expr = self.parse_expr();
            Block {
                id: self.next_node_id(),
                span: expr.span,
                stmts: Vec::new(),
                expr: Some(Box::new(expr)),
            }
        };
        
        Expr {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            kind: ExprKind::Closure(Closure {
                params,
                return_type,
                body,
                is_move: false,
            }),
            ty: None,
        }
    }
    
    fn parse_infix(&mut self, lhs: Expr) -> Expr {
        let start = lhs.span;
        
        if self.check(&TokenKind::Eq) && !self.peek_is(&TokenKind::Eq) && !self.peek_is(&TokenKind::Gt) {
            self.advance();
            let rhs = self.parse_expr_prec(Precedence::Assignment);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Assign(Box::new(lhs), Box::new(rhs)),
                ty: None,
            };
        }
        
        if let Some(op) = self.parse_compound_assignment_op() {
            self.advance();
            let rhs = self.parse_expr_prec(Precedence::Assignment);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::AssignOp(op, Box::new(lhs), Box::new(rhs)),
                ty: None,
            };
        }
        
        if let Some((op, prec)) = self.parse_binary_op() {
            self.advance();
            let rhs = self.parse_expr_prec(prec);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::LParen) {
            self.advance();
            let mut args = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                args.push(self.parse_expr());
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect(TokenKind::RParen);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Call(Box::new(lhs), args),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::Dot) {
            self.advance();
            let method = self.expect_ident();
            
            let mut type_args = Vec::new();
            if self.check(&TokenKind::DoubleColon) && self.peek_is(&TokenKind::Lt) {
                self.advance();
                self.expect(TokenKind::Lt);
                while !self.check(&TokenKind::Gt) && !self.check(&TokenKind::Eof) {
                    type_args.push(self.parse_type());
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.expect(TokenKind::Gt);
            }
            
            self.expect(TokenKind::LParen);
            let mut args = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                args.push(self.parse_expr());
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect(TokenKind::RParen);
            
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::MethodCall(Box::new(lhs), method, type_args, args),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::Dot) {
            self.advance();
            let field = self.expect_ident();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Field(Box::new(lhs), field),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::LBracket) {
            self.advance();
            let index = self.parse_expr();
            self.expect(TokenKind::RBracket);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Index(Box::new(lhs), Box::new(index)),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::Question) {
            self.advance();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Try(Box::new(lhs)),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::As) {
            self.advance();
            let ty = self.parse_type();
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Cast(Box::new(lhs), ty),
                ty: None,
            };
        }
        
        if self.check(&TokenKind::DotDot) || self.check(&TokenKind::DotDotEq) {
            let end_kind = if self.check(&TokenKind::DotDotEq) {
                RangeEnd::Included
            } else {
                RangeEnd::Excluded
            };
            self.advance();
            let end = self.parse_expr_prec(Precedence::Range);
            return Expr {
                id: self.next_node_id(),
                span: start.merge(self.previous_span()),
                kind: ExprKind::Range(Box::new(lhs), Box::new(end), end_kind),
                ty: None,
            };
        }
        
        lhs
    }
    
    fn parse_pattern(&mut self) -> Pattern {
        let mut patterns = Vec::new();
        patterns.push(self.parse_pattern_single());
        
        while self.check(&TokenKind::Pipe) {
            self.advance();
            patterns.push(self.parse_pattern_single());
        }
        
        if patterns.len() > 1 {
            Pattern::Or(patterns)
        } else {
            patterns.into_iter().next().unwrap()
        }
    }
    
    fn parse_pattern_single(&mut self) -> Pattern {
        if self.check(&TokenKind::Underscore) {
            self.advance();
            return Pattern::Wildcard;
        }
        
        if matches!(self.current_token().kind, 
            TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::String(_) | 
            TokenKind::Char(_) | TokenKind::Bool(_)) {
            let expr_kind = self.parse_expr_kind();
            return Pattern::Literal(expr_kind);
        }
        
        if self.check(&TokenKind::Ident("".to_string())) || self.check(&TokenKind::DoubleColon) {
            let path = self.parse_path();
            
            if self.check(&TokenKind::LBrace) {
                self.advance();
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                    let name = self.expect_ident();
                    let pattern = if self.check(&TokenKind::Colon) {
                        self.advance();
                        self.parse_pattern()
                    } else {
                        Pattern::Ident(name.clone())
                    };
                    fields.push(FieldPattern { name, pattern });
                    
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.expect(TokenKind::RBrace);
                return Pattern::Struct(path, fields);
            }
            
            if self.check(&TokenKind::LParen) {
                self.advance();
                let mut patterns = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                    patterns.push(self.parse_pattern());
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.expect(TokenKind::RParen);
                return Pattern::Struct(path, patterns.into_iter().enumerate().map(|(i, p)| FieldPattern {
                    name: i.to_string(),
                    pattern: p,
                }).collect());
            }
            
            return Pattern::Path(path);
        }
        
        if self.check(&TokenKind::Ampersand) {
            self.advance();
            let mutability = if self.check(&TokenKind::Mut) {
                self.advance();
                Mutability::Mutable
            } else {
                Mutability::Immutable
            };
            let pattern = self.parse_pattern();
            return Pattern::Ref(Box::new(pattern), mutability);
        }
        
        if self.check(&TokenKind::LParen) {
            self.advance();
            let mut patterns = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                patterns.push(self.parse_pattern());
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect(TokenKind::RParen);
            return Pattern::Tuple(patterns);
        }
        
        if self.check(&TokenKind::Ellipsis) {
            self.advance();
            return Pattern::Rest;
        }
        
        if self.check(&TokenKind::Ident("".to_string())) {
            let name = self.expect_ident();
            return Pattern::Ident(name);
        }
        
        self.error("expected pattern");
        self.synchronize();
        Pattern::Wildcard
    }
    
    fn parse_macro(&mut self) -> MacroCall {
        let start = self.current_span();
        let path = self.parse_path();
        self.expect(TokenKind::Exclaim);
        
        let mut args = Vec::new();
        
        if self.check(&TokenKind::LParen) {
            self.advance();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                args.push(self.parse_macro_arg());
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect(TokenKind::RParen);
        } else if self.check(&TokenKind::LBrace) {
            self.advance();
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                args.push(self.parse_macro_arg());
                if !self.check(&TokenKind::Comma) && !self.check(&TokenKind::Semi) {
                    break;
                }
                if self.check(&TokenKind::Comma) || self.check(&TokenKind::Semi) {
                    self.advance();
                }
            }
            self.expect(TokenKind::RBrace);
        } else if self.check(&TokenKind::LBracket) {
            self.advance();
            while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
                args.push(self.parse_macro_arg());
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect(TokenKind::RBracket);
        }
        
        MacroCall {
            id: self.next_node_id(),
            span: start.merge(self.previous_span()),
            path,
            args,
        }
    }
    
    fn parse_macro_arg(&mut self) -> MacroArg {
        MacroArg::Expr(self.parse_expr())
    }
    
    // Type parsing
    fn parse_type(&mut self) -> Type {
        self.parse_type_prec(TypePrecedence::Lowest)
    }
    
    fn parse_type_opt(&mut self) -> Option<Type> {
        if self.check(&TokenKind::Ident("".to_string())) || self.check(&TokenKind::DoubleColon) {
            Some(self.parse_type())
        } else {
            None
        }
    }
    
    fn parse_type_prec(&mut self, prec: TypePrecedence) -> Type {
        let mut lhs = self.parse_type_prefix();
        
        while prec < self.current_type_precedence() {
            lhs = self.parse_type_infix(lhs);
        }
        
        lhs
    }
    
    fn parse_type_prefix(&mut self) -> Type {
        if let TokenKind::Ident(name) = &self.current_token().kind {
            match name.as_str() {
                "bool" => { self.advance(); return Type::Bool; }
                "i8" => { self.advance(); return Type::Int(IntTy::I8); }
                "i16" => { self.advance(); return Type::Int(IntTy::I16); }
                "i32" => { self.advance(); return Type::Int(IntTy::I32); }
                "i64" => { self.advance(); return Type::Int(IntTy::I64); }
                "i128" => { self.advance(); return Type::Int(IntTy::I128); }
                "isize" => { self.advance(); return Type::Int(IntTy::Isize); }
                "u8" => { self.advance(); return Type::Int(IntTy::U8); }
                "u16" => { self.advance(); return Type::Int(IntTy::U16); }
                "u32" => { self.advance(); return Type::Int(IntTy::U32); }
                "u64" => { self.advance(); return Type::Int(IntTy::U64); }
                "u128" => { self.advance(); return Type::Int(IntTy::U128); }
                "usize" => { self.advance(); return Type::Int(IntTy::Usize); }
                "f32" => { self.advance(); return Type::Float(FloatTy::F32); }
                "f64" => { self.advance(); return Type::Float(FloatTy::F64); }
                "char" => { self.advance(); return Type::Char; }
                "str" => { self.advance(); return Type::Str; }
                "never" | "!" => { self.advance(); return Type::Never; }
                "_" => { self.advance(); return Type::Infer; }
                _ => {}
            }
        }
        
        if self.check(&TokenKind::Ident("".to_string())) || self.check(&TokenKind::DoubleColon) {
            let path = self.parse_path();
            return Type::Path(path);
        }
        
        if self.check(&TokenKind::Ampersand) {
            self.advance();
            let mutability = if self.check(&TokenKind::Mut) {
                self.advance();
                Mutability::Mutable
            } else {
                Mutability::Immutable
            };
            let inner = self.parse_type();
            return Type::Ref(Box::new(inner), mutability);
        }
        
        if self.check(&TokenKind::Star) {
            self.advance();
            let mutability = if self.check(&TokenKind::Mut) {
                self.advance();
                Mutability::Mutable
            } else if self.check(&TokenKind::Const) {
                self.advance();
                Mutability::Immutable
            } else {
                Mutability::Immutable
            };
            let inner = self.parse_type();
            return Type::RawPtr(Box::new(inner), mutability);
        }
        
        if self.check(&TokenKind::Fn) {
            self.advance();
            self.expect(TokenKind::LParen);
            let mut params = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                params.push(self.parse_type());
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect(TokenKind::RParen);
            let return_type = if self.check(&TokenKind::Arrow) {
                self.advance();
                self.parse_type()
            } else {
                Type::Tuple(Vec::new())
            };
            return Type::FnPtr(FnSig {
                params,
                return_type: Box::new(return_type),
                is_variadic: false,
                abi: None,
            });
        }
        
        if self.check(&TokenKind::LParen) {
            self.advance();
            let mut types = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                types.push(self.parse_type());
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect(TokenKind::RParen);
            if types.len() == 1 {
                return types.into_iter().next().unwrap();
            }
            return Type::Tuple(types);
        }
        
        if self.check(&TokenKind::LBracket) {
            self.advance();
            let elem = self.parse_type();
            
            if self.check(&TokenKind::Semi) {
                self.advance();
                let size = if let TokenKind::Integer(n) = self.current_token().kind {
                    self.advance();
                    Some(n as usize)
                } else {
                    None
                };
                self.expect(TokenKind::RBracket);
                return Type::Array(Box::new(elem), size);
            } else {
                self.expect(TokenKind::RBracket);
                return Type::Slice(Box::new(elem));
            }
        }
        
        if self.check(&TokenKind::Bang) {
            self.advance();
            return Type::Never;
        }
        
        self.error("expected type");
        self.synchronize();
        Type::Infer
    }
    
    fn parse_type_infix(&mut self, lhs: Type) -> Type {
        lhs
    }
    
    fn parse_path(&mut self) -> Path {
        let mut segments = Vec::new();
        let is_absolute = self.check(&TokenKind::DoubleColon);
        
        if is_absolute {
            self.advance();
        }
        
        loop {
            let name = self.expect_ident();
            let mut type_args = Vec::new();
            
            if self.check(&TokenKind::DoubleColon) && self.peek_is(&TokenKind::Lt) {
                self.advance();
                self.expect(TokenKind::Lt);
                while !self.check(&TokenKind::Gt) && !self.check(&TokenKind::Eof) {
                    type_args.push(self.parse_type());
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.expect(TokenKind::Gt);
            }
            
            segments.push(PathSegment { name, type_args });
            
            if !self.check(&TokenKind::DoubleColon) {
                break;
            }
            self.advance();
        }
        
        Path { segments, is_absolute }
    }
    
    // Operator parsing helpers
    fn parse_unary_op(&mut self) -> Option<UnOp> {
        match self.current_token().kind {
            TokenKind::Minus => Some(UnOp::Neg),
            TokenKind::Bang => Some(UnOp::Not),
            TokenKind::Tilde => Some(UnOp::BitNot),
            TokenKind::Star => Some(UnOp::Deref),
            TokenKind::Ampersand => {
                let mut peeked = self.current + 1;
                if peeked < self.tokens.len() {
                    if matches!(self.tokens[peeked].kind, TokenKind::Mut) {
                        return Some(UnOp::RefMut);
                    }
                }
                Some(UnOp::Ref)
            }
            _ => None,
        }
    }
    
    fn parse_binary_op(&mut self) -> Option<(BinOp, Precedence)> {
        let (op, prec) = match &self.current_token().kind {
            TokenKind::OrOr => (BinOp::Or, Precedence::LogicalOr),
            TokenKind::AndAnd => (BinOp::And, Precedence::LogicalAnd),
            TokenKind::EqEq => (BinOp::Eq, Precedence::Comparison),
            TokenKind::NotEq => (BinOp::Ne, Precedence::Comparison),
            TokenKind::Lt => (BinOp::Lt, Precedence::Comparison),
            TokenKind::Gt => (BinOp::Gt, Precedence::Comparison),
            TokenKind::Le => (BinOp::Le, Precedence::Comparison),
            TokenKind::Ge => (BinOp::Ge, Precedence::Comparison),
            TokenKind::Pipe => (BinOp::BitOr, Precedence::BitwiseOr),
            TokenKind::Caret => (BinOp::BitXor, Precedence::BitwiseXor),
            TokenKind::Ampersand => (BinOp::BitAnd, Precedence::BitwiseAnd),
            TokenKind::Shl => (BinOp::Shl, Precedence::Shift),
            TokenKind::Shr => (BinOp::Shr, Precedence::Shift),
            TokenKind::Plus => (BinOp::Add, Precedence::Additive),
            TokenKind::Minus => (BinOp::Sub, Precedence::Additive),
            TokenKind::Star => (BinOp::Mul, Precedence::Multiplicative),
            TokenKind::Slash => (BinOp::Div, Precedence::Multiplicative),
            TokenKind::Percent => (BinOp::Rem, Precedence::Multiplicative),
            _ => return None,
        };
        Some((op, prec))
    }
    
    fn parse_compound_assignment_op(&mut self) -> Option<BinOp> {
        match &self.current_token().kind {
            TokenKind::PlusEq => Some(BinOp::AddAssign),
            TokenKind::MinusEq => Some(BinOp::SubAssign),
            TokenKind::StarEq => Some(BinOp::MulAssign),
            TokenKind::SlashEq => Some(BinOp::DivAssign),
            TokenKind::PercentEq => Some(BinOp::RemAssign),
            TokenKind::CaretEq => Some(BinOp::BitXorAssign),
            TokenKind::AmpEq => Some(BinOp::BitAndAssign),
            TokenKind::PipeEq => Some(BinOp::BitOrAssign),
            TokenKind::ShlEq => Some(BinOp::ShlAssign),
            TokenKind::ShrEq => Some(BinOp::ShrAssign),
            _ => None,
        }
    }
    
    // Token stream helpers
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current_token().kind) == std::mem::discriminant(kind)
    }
    
    fn peek_is(&self, kind: &TokenKind) -> bool {
        if self.current + 1 < self.tokens.len() {
            std::mem::discriminant(&self.tokens[self.current + 1].kind) == std::mem::discriminant(kind)
        } else {
            false
        }
    }
    
    fn peek_is_terminator(&self) -> bool {
        if self.current + 1 < self.tokens.len() {
            matches!(self.tokens[self.current + 1].kind, 
                TokenKind::RBrace | TokenKind::Semi | TokenKind::Eof)
        } else {
            true
        }
    }
    
    fn peek_is_macro(&self) -> bool {
        if self.current + 1 < self.tokens.len() {
            matches!(self.tokens[self.current + 1].kind, TokenKind::Exclaim)
        } else {
            false
        }
    }
    
    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() - 1 {
            self.current += 1;
        }
        &self.tokens[self.current]
    }
    
    fn expect(&mut self, kind: TokenKind) -> &Token {
        if self.check(&kind) {
            self.advance();
            &self.tokens[self.current - 1]
        } else {
            self.error(&format!("expected {:?}, found {:?}", kind, self.current_token().kind));
            self.current_token()
        }
    }
    
    fn expect_ident(&mut self) -> String {
        if let TokenKind::Ident(name) = &self.current_token().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            self.error("expected identifier");
            self.advance();
            "<error>".to_string()
        }
    }
    
    fn current_span(&self) -> Span {
        self.current_token().span
    }
    
    fn previous_span(&self) -> Span {
        if self.current > 0 {
            self.tokens[self.current - 1].span
        } else {
            Span::dummy()
        }
    }
    
    fn error(&mut self, message: &str) {
        self.diagnostics.push(Diagnostic::error(message)
            .with_label(Label::primary(self.current_span(), "here")));
    }
    
    fn synchronize(&mut self) {
        while !self.check(&TokenKind::Eof) {
            if self.previous_token_was_semi() {
                return;
            }
            
            match self.current_token().kind {
                TokenKind::Fn | TokenKind::Struct | TokenKind::Enum | 
                TokenKind::Impl | TokenKind::Type | TokenKind::Const |
                TokenKind::Static | TokenKind::Extern | TokenKind::Mod |
                TokenKind::Let | TokenKind::Pub => return,
                _ => {}
            }
            
            self.advance();
        }
    }
    
    fn previous_token_was_semi(&self) -> bool {
        if self.current > 0 {
            matches!(self.tokens[self.current - 1].kind, TokenKind::Semi)
        } else {
            false
        }
    }
    
    fn current_precedence(&self) -> Precedence {
        match self.current_token().kind {
            TokenKind::OrOr => Precedence::LogicalOr,
            TokenKind::AndAnd => Precedence::LogicalAnd,
            TokenKind::EqEq | TokenKind::NotEq => Precedence::Comparison,
            TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => Precedence::Comparison,
            TokenKind::Pipe => Precedence::BitwiseOr,
            TokenKind::Caret => Precedence::BitwiseXor,
            TokenKind::Ampersand => Precedence::BitwiseAnd,
            TokenKind::Shl | TokenKind::Shr => Precedence::Shift,
            TokenKind::Plus | TokenKind::Minus => Precedence::Additive,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Multiplicative,
            TokenKind::DotDot | TokenKind::DotDotEq => Precedence::Range,
            _ => Precedence::Lowest,
        }
    }
    
    fn current_type_precedence(&self) -> TypePrecedence {
        TypePrecedence::Lowest
    }
}

/// Precedence levels for expression parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest,
    Assignment,
    Range,
    LogicalOr,
    LogicalAnd,
    Comparison,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Shift,
    Additive,
    Multiplicative,
    Prefix,
    Postfix,
    Call,
    Primary,
}

/// Precedence levels for type parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TypePrecedence {
    Lowest,
    Primary,
}

/// Public parse function
pub fn parse(tokens: Vec<Token>, filename: &str) -> Result<Module, DiagnosticBag> {
    let parser = Parser::new(tokens, 0);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::tokenize;
    
    #[test]
    fn test_parse_simple_fn() {
        let source = "fn main() { var x = 42; }";
        let tokens = tokenize(source, "test.nya").unwrap();
        let module = parse(tokens, "test.nya").unwrap();
        assert_eq!(module.items.len(), 1);
    }
    
    #[test]
    fn test_parse_struct() {
        let source = "struct Cat { name: String, age: u8 }";
        let tokens = tokenize(source, "test.nya").unwrap();
        let module = parse(tokens, "test.nya").unwrap();
        assert_eq!(module.items.len(), 1);
    }
    
    #[test]
    fn test_parse_enum() {
        let source = "enum Option<T> { Some(T), None }";
        let tokens = tokenize(source, "test.nya").unwrap();
        let module = parse(tokens, "test.nya").unwrap();
        assert_eq!(module.items.len(), 1);
    }
}