//! Visitor pattern for AST traversal

use crate::module::Module;
use crate::item::*;
use crate::expr::*;
use crate::stmt::*;
use crate::pattern::*;
use crate::ty::*;
use crate::path::*;
use crate::attr::*;
use crate::import::*;
use crate::generics::*;
use crate::macro_call::*;

/// Visitor trait for AST traversal
pub trait AstVisitor {
    fn visit_module(&mut self, module: &Module) {
        walk_module(self, module);
    }
    
    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item);
    }
    
    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }
    
    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }
    
    fn visit_type(&mut self, ty: &Type) {
        walk_type(self, ty);
    }
    
    fn visit_pattern(&mut self, pat: &Pattern) {
        walk_pattern(self, pat);
    }
    
    fn visit_block(&mut self, block: &Block) {
        walk_block(self, block);
    }
}

/// Walk functions for default visitor behavior
pub fn walk_module<V: AstVisitor + ?Sized>(visitor: &mut V, module: &Module) {
    for import in &module.imports {
        walk_import(visitor, import);
    }
    for item in &module.items {
        walk_item(visitor, item);
    }
}

pub fn walk_import<V: AstVisitor + ?Sized>(visitor: &mut V, import: &Import) {
    for item in &import.items {
        // Import items don't have sub-nodes to walk
    }
}

pub fn walk_item<V: AstVisitor + ?Sized>(visitor: &mut V, item: &Item) {
    match item {
        Item::Fn(f) => walk_function(visitor, f),
        Item::Struct(s) => walk_struct(visitor, s),
        Item::Enum(e) => walk_enum(visitor, e),
        Item::Impl(i) => walk_impl(visitor, i),
        Item::TypeAlias(t) => walk_type_alias(visitor, t),
        Item::Const(c) => walk_const(visitor, c),
        Item::Static(s) => walk_static(visitor, s),
        Item::ExternBlock(e) => walk_extern_block(visitor, e),
        Item::Module(m) => walk_module(visitor, m),
    }
}

pub fn walk_function<V: AstVisitor + ?Sized>(visitor: &mut V, func: &Function) {
    for param in &func.params {
        visitor.visit_type(&param.ty);
    }
    if let Some(ret) = &func.return_type {
        visitor.visit_type(ret);
    }
    if let Some(body) = &func.body {
        walk_block(visitor, body);
    }
    for attr in &func.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_struct<V: AstVisitor + ?Sized>(visitor: &mut V, s: &Struct) {
    for field in &s.fields {
        visitor.visit_type(&field.ty);
        for attr in &field.attributes {
            walk_attribute(visitor, attr);
        }
    }
    for attr in &s.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_enum<V: AstVisitor + ?Sized>(visitor: &mut V, e: &Enum) {
    for variant in &e.variants {
        walk_enum_variant(visitor, variant);
    }
    for attr in &e.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_enum_variant<V: AstVisitor + ?Sized>(visitor: &mut V, v: &EnumVariant) {
    match &v.data {
        EnumVariantData::Unit => {}
        EnumVariantData::Tuple(types) => {
            for ty in types {
                visitor.visit_type(ty);
            }
        }
        EnumVariantData::Struct(fields) => {
            for field in fields {
                visitor.visit_type(&field.ty);
                for attr in &field.attributes {
                    walk_attribute(visitor, attr);
                }
            }
        }
    }
    if let Some(disc) = &v.discriminant {
        visitor.visit_expr(disc);
    }
    for attr in &v.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_impl<V: AstVisitor + ?Sized>(visitor: &mut V, i: &Impl) {
    visitor.visit_type(&i.self_ty);
    if let Some(trait_ref) = &i.trait_ref {
        walk_path(visitor, trait_ref);
    }
    for item in &i.items {
        match item {
            ImplItem::Fn(f) => walk_function(visitor, f),
            ImplItem::Const(c) => walk_const(visitor, c),
            ImplItem::TypeAlias(t) => walk_type_alias(visitor, t),
        }
    }
    for attr in &i.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_type_alias<V: AstVisitor + ?Sized>(visitor: &mut V, t: &TypeAlias) {
    visitor.visit_type(&t.ty);
    for attr in &t.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_const<V: AstVisitor + ?Sized>(visitor: &mut V, c: &Const) {
    visitor.visit_type(&c.ty);
    visitor.visit_expr(&c.value);
    for attr in &c.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_static<V: AstVisitor + ?Sized>(visitor: &mut V, s: &Static) {
    visitor.visit_type(&s.ty);
    visitor.visit_expr(&s.value);
    for attr in &s.attributes {
        walk_attribute(visitor, attr);
    }
}

pub fn walk_extern_block<V: AstVisitor + ?Sized>(visitor: &mut V, e: &ExternBlock) {
    for item in &e.items {
        match item {
            ExternItem::Fn(f) => walk_function(visitor, f),
            ExternItem::Static(s) => walk_static(visitor, s),
        }
    }
}

pub fn walk_block<V: AstVisitor + ?Sized>(visitor: &mut V, block: &Block) {
    for stmt in &block.stmts {
        visitor.visit_stmt(stmt);
    }
    if let Some(expr) = &block.expr {
        visitor.visit_expr(expr);
    }
}

pub fn walk_stmt<V: AstVisitor + ?Sized>(visitor: &mut V, stmt: &Stmt) {
    match stmt {
        Stmt::Let(l) => {
            if let Some(ty) = &l.ty {
                visitor.visit_type(ty);
            }
            if let Some(init) = &l.init {
                visitor.visit_expr(init);
            }
        }
        Stmt::Expr(e) => visitor.visit_expr(e),
        Stmt::Item(i) => visitor.visit_item(i),
        Stmt::Macro(m) => walk_macro(visitor, m),
    }
}

pub fn walk_expr<V: AstVisitor + ?Sized>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Path(p) => walk_path(visitor, p),
        ExprKind::Unary(_, e) => visitor.visit_expr(e),
        ExprKind::Binary(_, l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::Call(callee, args) => {
            visitor.visit_expr(callee);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::MethodCall(receiver, _, type_args, args) => {
            visitor.visit_expr(receiver);
            for ty in type_args {
                visitor.visit_type(ty);
            }
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::Field(e, _) => visitor.visit_expr(e),
        ExprKind::Index(e, idx) => {
            visitor.visit_expr(e);
            visitor.visit_expr(idx);
        }
        ExprKind::Range(start, end, _) => {
            visitor.visit_expr(start);
            visitor.visit_expr(end);
        }
        ExprKind::Struct(path, fields) => {
            walk_path(visitor, path);
            for f in fields {
                visitor.visit_expr(&f.expr);
            }
        }
        ExprKind::Array(elems) => {
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Tuple(elems) => {
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Block(b) => visitor.visit_block(b),
        ExprKind::If(cond, then_branch, else_branch) => {
            visitor.visit_expr(cond);
            walk_block(visitor, then_branch);
            if let Some(eb) = else_branch {
                visitor.visit_expr(eb);
            }
        }
        ExprKind::Match(scrutinee, arms) => {
            visitor.visit_expr(scrutinee);
            for arm in arms {
                walk_match_arm(visitor, arm);
            }
        }
        ExprKind::Loop(body, _) => walk_block(visitor, body),
        ExprKind::While(cond, body, _) => {
            visitor.visit_expr(cond);
            walk_block(visitor, body);
        }
        ExprKind::For(_, iter, body, _) => {
            visitor.visit_expr(iter);
            walk_block(visitor, body);
        }
        ExprKind::Break(_, e) => {
            if let Some(e) = e {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Return(e) => {
            if let Some(e) = e {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Try(e) => visitor.visit_expr(e),
        ExprKind::Closure(c) => {
            for param in &c.params {
                visitor.visit_type(&param.ty);
            }
            if let Some(ret) = &c.return_type {
                visitor.visit_type(ret);
            }
            walk_block(visitor, &c.body);
        }
        ExprKind::Cast(e, ty) => {
            visitor.visit_expr(e);
            visitor.visit_type(ty);
        }
        ExprKind::Assign(l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::AssignOp(_, l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::Ref(e, _) => visitor.visit_expr(e),
        ExprKind::Deref(e) => visitor.visit_expr(e),
        ExprKind::Group(e) => visitor.visit_expr(e),
        _ => {}
    }
}

pub fn walk_match_arm<V: AstVisitor + ?Sized>(visitor: &mut V, arm: &MatchArm) {
    walk_pattern(visitor, &arm.pattern);
    if let Some(guard) = &arm.guard {
        visitor.visit_expr(guard);
    }
    visitor.visit_expr(&arm.body);
}

pub fn walk_pattern<V: AstVisitor + ?Sized>(visitor: &mut V, pat: &Pattern) {
    match pat {
        Pattern::Tuple(pats) => {
            for p in pats {
                walk_pattern(visitor, p);
            }
        }
        Pattern::Struct(path, fields) => {
            walk_path(visitor, path);
            for f in fields {
                walk_pattern(visitor, &f.pattern);
            }
        }
        Pattern::Or(pats) => {
            for p in pats {
                walk_pattern(visitor, p);
            }
        }
        Pattern::Ref(p, _) => walk_pattern(visitor, p),
        _ => {}
    }
}

pub fn walk_type<V: AstVisitor + ?Sized>(visitor: &mut V, ty: &Type) {
    match ty {
        Type::Path(p) => walk_path(visitor, p),
        Type::Tuple(types) => {
            for t in types {
                visitor.visit_type(t);
            }
        }
        Type::Array(t, _) => visitor.visit_type(t),
        Type::Slice(t) => visitor.visit_type(t),
        Type::Ref(t, _) => visitor.visit_type(t),
        Type::RawPtr(t, _) => visitor.visit_type(t),
        Type::FnPtr(sig) => {
            for p in &sig.params {
                visitor.visit_type(p);
            }
            visitor.visit_type(&sig.return_type);
        }
        _ => {}
    }
}

pub fn walk_path<V: AstVisitor + ?Sized>(visitor: &mut V, path: &Path) {
    for seg in &path.segments {
        for ty in &seg.type_args {
            visitor.visit_type(ty);
        }
    }
}

pub fn walk_attribute<V: AstVisitor + ?Sized>(visitor: &mut V, attr: &Attribute) {
    walk_path(visitor, &attr.path);
    for arg in &attr.args {
        walk_attribute_arg(visitor, arg);
    }
}

pub fn walk_attribute_arg<V: AstVisitor + ?Sized>(visitor: &mut V, arg: &AttributeArg) {
    match arg {
        AttributeArg::Literal(e) => visitor.visit_expr(e),
        AttributeArg::NameValue(_, e) => visitor.visit_expr(e),
        _ => {}
    }
}

pub fn walk_macro<V: AstVisitor + ?Sized>(visitor: &mut V, mac: &crate::macro_call::MacroCall) {
    walk_path(visitor, &mac.path);
    for arg in &mac.args {
        walk_macro_arg(visitor, arg);
    }
}

pub fn walk_macro_arg<V: AstVisitor + ?Sized>(visitor: &mut V, arg: &MacroArg) {
    match arg {
        MacroArg::Expr(e) => visitor.visit_expr(e),
        MacroArg::Stmt(s) => visitor.visit_stmt(s),
        MacroArg::Pat(p) => walk_pattern(visitor, p),
        MacroArg::Ty(t) => visitor.visit_type(t),
        MacroArg::Block(b) => walk_block(visitor, b),
    }
}

impl AstVisitor for () {}