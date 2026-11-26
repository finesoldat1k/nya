//! Type checker for nya

use crate::hir::{HirModule, HirItem, HirItemKind, HirExpr, HirExprKind, HirType, HirPath, HirPat, HirStmt, HirArm};
use crate::resolver::{ResolvedModule, ResolvedItem, ResolvedFunction, ResolvedStruct, ResolvedEnum, ResolvedImpl, ResolvedTypeAlias, ResolvedConst, ResolvedStatic, ResolvedExternBlock, ResolvedExternItem, ResolvedModule as ResolvedModuleData, ResolvedImport, ResolvedImportItem, ResolvedGenericParam, ResolvedParam, ResolvedStructField, ResolvedEnumVariant, ResolvedEnumVariantData, ResolvedBlock, ResolvedStmt, ResolvedLetStmt, ResolvedExpr, ResolvedExprKind, ResolvedFieldExpr, ResolvedArm, ResolvedPattern, ResolvedFieldPattern, ResolvedClosure, ResolvedMacroCall, ResolvedMacroArg, ResolvedPath, ResolvedPathSegment, ResolvedType, IntTy, FloatTy, Mutability, ResolvedFnSig, UnOp, BinOp, RangeEnd};
use nya_frontend::diagnostics::{Diagnostic, DiagnosticBag, Label, Severity, Span};
use indexmap::IndexMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::rc::Rc;

/// Type checking context
pub struct TypeChecker {
    module: ResolvedModule,
    diagnostics: DiagnosticBag,
    type_vars: IndexMap<String, ResolvedType>,
    next_type_var: usize,
}

impl TypeChecker {
    pub fn new(module: ResolvedModule) -> Self {
        Self {
            module,
            diagnostics: DiagnosticBag::new(),
            type_vars: IndexMap::new(),
            next_type_var: 0,
        }
    }
    
    pub fn check(mut self) -> Result<TypedModule, DiagnosticBag> {
        let module = self.check_module();
        
        if self.diagnostics.has_errors() {
            Err(self.diagnostics)
        } else {
            Ok(module)
        }
    }
    
    fn fresh_type_var(&mut self) -> ResolvedType {
        let name = format!("$T{}", self.next_type_var);
        self.next_type_var += 1;
        ResolvedType::Generic(name)
    }
    
    fn unify(&mut self, expected: &ResolvedType, actual: &ResolvedType, span: Span) -> bool {
        // Simplified unification - in reality this would be much more complex
        match (expected, actual) {
            (ResolvedType::Infer, _) | (_, ResolvedType::Infer) => true,
            (ResolvedType::Generic(a), ResolvedType::Generic(b)) if a == b => true,
            (ResolvedType::Int(a), ResolvedType::Int(b)) if a == b => true,
            (ResolvedType::Float(a), ResolvedType::Float(b)) if a == b => true,
            (ResolvedType::Bool, ResolvedType::Bool) => true,
            (ResolvedType::Char, ResolvedType::Char) => true,
            (ResolvedType::Str, ResolvedType::Str) => true,
            (ResolvedType::Never, ResolvedType::Never) => true,
            (ResolvedType::Path(a), ResolvedType::Path(b)) if a == b => true,
            (ResolvedType::Tuple(a), ResolvedType::Tuple(b)) if a.len() == b.len() => {
                a.iter().zip(b.iter()).all(|(ea, eb)| self.unify(ea, eb, span))
            }
            (ResolvedType::Array(a, sa), ResolvedType::Array(b, sb)) if sa == sb => {
                self.unify(a, b, span)
            }
            (ResolvedType::Slice(a), ResolvedType::Slice(b)) => self.unify(a, b, span),
            (ResolvedType::Ref(a, ma), ResolvedType::Ref(b, mb)) if ma == mb => self.unify(a, b, span),
            (ResolvedType::RawPtr(a, ma), ResolvedType::RawPtr(b, mb)) if ma == mb => self.unify(a, b, span),
            _ => {
                self.diagnostics.push(Diagnostic::error(format!("type mismatch: expected {:?}, found {:?}", expected, actual))
                    .with_label(Label::primary(span, "expected this type"))
                    .with_label(Label::secondary(span, "but found this type")));
                false
            }
        }
    }
    
    fn check_module(&mut self) -> TypedModule {
        let mut items = Vec::new();
        
        for item in &self.module.items {
            items.push(self.check_item(item));
        }
        
        TypedModule {
            name: self.module.name.clone(),
            span: self.module.span,
            items,
            imports: self.module.imports.clone(),
        }
    }
    
    fn check_item(&mut self, item: &ResolvedItem) -> TypedItem {
        match item {
            ResolvedItem::Function(f) => TypedItem::Function(self.check_function(f)),
            ResolvedItem::Struct(s) => TypedItem::Struct(self.check_struct(s)),
            ResolvedItem::Enum(e) => TypedItem::Enum(self.check_enum(e)),
            ResolvedItem::Impl(i) => TypedItem::Impl(self.check_impl(i)),
            ResolvedItem::TypeAlias(t) => TypedItem::TypeAlias(self.check_type_alias(t)),
            ResolvedItem::Const(c) => TypedItem::Const(self.check_const(c)),
            ResolvedItem::Static(s) => TypedItem::Static(self.check_static(s)),
            ResolvedItem::ExternBlock(e) => TypedItem::ExternBlock(self.check_extern_block(e)),
            ResolvedItem::Module(m) => TypedItem::Module(self.check_module_data(m)),
        }
    }
    
    fn check_function(&mut self, func: &ResolvedFunction) -> TypedFunction {
        // Check parameter types
        for param in &func.params {
            self.check_type(&param.ty);
        }
        
        // Check return type
        if let Some(ret) = &func.return_type {
            self.check_type(ret);
        }
        
        // Check body
        let body = func.body.as_ref().map(|b| self.check_block(b, func.return_type.as_ref()));
        
        TypedFunction {
            id: func.id,
            span: func.span,
            name: func.name.clone(),
            generics: func.generics.clone(),
            params: func.params.clone(),
            return_type: func.return_type.clone(),
            body,
            is_pub: func.is_pub,
            is_static: func.is_static,
            is_extern: func.is_extern,
            abi: func.abi.clone(),
            attributes: func.attributes.clone(),
        }
    }
    
    fn check_struct(&mut self, s: &ResolvedStruct) -> TypedStruct {
        for field in &s.fields {
            self.check_type(&field.ty);
        }
        
        TypedStruct {
            id: s.id,
            span: s.span,
            name: s.name.clone(),
            generics: s.generics.clone(),
            fields: s.fields.clone(),
            is_pub: s.is_pub,
            attributes: s.attributes.clone(),
        }
    }
    
    fn check_enum(&mut self, e: &ResolvedEnum) -> TypedEnum {
        for variant in &e.variants {
            self.check_enum_variant(variant);
        }
        
        TypedEnum {
            id: e.id,
            span: e.span,
            name: e.name.clone(),
            generics: e.generics.clone(),
            variants: e.variants.clone(),
            is_pub: e.is_pub,
            attributes: e.attributes.clone(),
        }
    }
    
    fn check_enum_variant(&mut self, v: &ResolvedEnumVariant) {
        match &v.data {
            ResolvedEnumVariantData::Unit => {}
            ResolvedEnumVariantData::Tuple(types) => {
                for t in types {
                    self.check_type(t);
                }
            }
            ResolvedEnumVariantData::Struct(fields) => {
                for f in fields {
                    self.check_type(&f.ty);
                }
            }
        }
    }
    
    fn check_impl(&mut self, i: &ResolvedImpl) -> TypedImpl {
        self.check_type(&i.self_ty);
        
        if let Some(trait_ref) = &i.trait_ref {
            self.check_path(trait_ref);
        }
        
        let mut items = Vec::new();
        for item in &i.items {
            items.push(match item {
                ResolvedImplItem::Function(f) => TypedImplItem::Function(self.check_function(f)),
                ResolvedImplItem::Const(c) => TypedImplItem::Const(self.check_const(c)),
                ResolvedImplItem::TypeAlias(t) => TypedImplItem::TypeAlias(self.check_type_alias(t)),
            });
        }
        
        TypedImpl {
            id: i.id,
            span: i.span,
            generics: i.generics.clone(),
            trait_ref: i.trait_ref.clone(),
            self_ty: i.self_ty.clone(),
            items,
            attributes: i.attributes.clone(),
        }
    }
    
    fn check_type_alias(&mut self, t: &ResolvedTypeAlias) -> TypedTypeAlias {
        self.check_type(&t.ty);
        
        TypedTypeAlias {
            id: t.id,
            span: t.span,
            name: t.name.clone(),
            generics: t.generics.clone(),
            ty: t.ty.clone(),
            is_pub: t.is_pub,
            attributes: t.attributes.clone(),
        }
    }
    
    fn check_const(&mut self, c: &ResolvedConst) -> TypedConst {
        self.check_type(&c.ty);
        let value = self.check_expr(&c.value);
        // Check that value type matches declared type
        if let Some(value_ty) = &value.ty {
            self.unify(&c.ty, value_ty, value.span);
        }
        
        TypedConst {
            id: c.id,
            span: c.span,
            name: c.name.clone(),
            ty: c.ty.clone(),
            value,
            is_pub: c.is_pub,
            attributes: c.attributes.clone(),
        }
    }
    
    fn check_static(&mut self, s: &ResolvedStatic) -> TypedStatic {
        self.check_type(&s.ty);
        let value = self.check_expr(&s.value);
        if let Some(value_ty) = &value.ty {
            self.unify(&s.ty, value_ty, value.span);
        }
        
        TypedStatic {
            id: s.id,
            span: s.span,
            name: s.name.clone(),
            ty: s.ty.clone(),
            value,
            is_pub: s.is_pub,
            is_mut: s.is_mut,
            attributes: s.attributes.clone(),
        }
    }
    
    fn check_extern_block(&mut self, e: &ResolvedExternBlock) -> TypedExternBlock {
        let mut items = Vec::new();
        for item in &e.items {
            items.push(match item {
                ResolvedExternItem::Function(f) => TypedExternItem::Function(self.check_function(f)),
                ResolvedExternItem::Static(s) => TypedExternItem::Static(self.check_static(s)),
            });
        }
        
        TypedExternBlock {
            id: e.id,
            span: e.span,
            abi: e.abi.clone(),
            items,
        }
    }
    
    fn check_module_data(&mut self, m: &ResolvedModuleData) -> TypedModule {
        let mut items = Vec::new();
        for item in &m.items {
            items.push(self.check_item(item));
        }
        
        TypedModule {
            name: m.name.clone(),
            span: m.span,
            items,
            imports: m.imports.clone(),
        }
    }
    
    fn check_block(&mut self, block: &ResolvedBlock, expected_ret: Option<&ResolvedType>) -> TypedBlock {
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.push(self.check_stmt(stmt));
        }
        
        let expr = block.expr.as_ref().map(|e| {
            let checked = self.check_expr(e);
            if let Some(expected) = expected_ret {
                if let Some(actual) = &checked.ty {
                    self.unify(expected, actual, checked.span);
                }
            }
            Box::new(checked)
        });
        
        TypedBlock {
            id: block.id,
            span: block.span,
            stmts,
            expr,
        }
    }
    
    fn check_stmt(&mut self, stmt: &ResolvedStmt) -> TypedStmt {
        match stmt {
            ResolvedStmt::Let(l) => TypedStmt::Let(self.check_let_stmt(l)),
            ResolvedStmt::Expr(e) => TypedStmt::Expr(self.check_expr(e)),
            ResolvedStmt::Item(i) => TypedStmt::Item(self.check_item(i)),
            ResolvedStmt::Macro(m) => TypedStmt::Macro(self.check_macro(m)),
        }
    }
    
    fn check_let_stmt(&mut self, l: &ResolvedLetStmt) -> TypedLetStmt {
        let ty = l.ty.as_ref().map(|t| self.check_type(t)).flatten();
        let init = l.init.as_ref().map(|e| {
            let checked = self.check_expr(e);
            if let Some(expected) = &ty {
                if let Some(actual) = &checked.ty {
                    self.unify(expected, actual, checked.span);
                }
            }
            checked
        });
        
        TypedLetStmt {
            id: l.id,
            span: l.span,
            name: l.name.clone(),
            ty,
            init,
            is_mut: l.is_mut,
            is_ref: l.is_ref,
        }
    }
    
    fn check_expr(&mut self, expr: &ResolvedExpr) -> TypedExpr {
        let (kind, ty) = match &expr.kind {
            ResolvedExprKind::Bool(_) => (expr.kind.clone(), Some(ResolvedType::Bool)),
            ResolvedExprKind::Integer(_) => (expr.kind.clone(), Some(ResolvedType::Int(IntTy::I32))),
            ResolvedExprKind::Float(_) => (expr.kind.clone(), Some(ResolvedType::Float(FloatTy::F64))),
            ResolvedExprKind::String(_) => (expr.kind.clone(), Some(ResolvedType::Str)),
            ResolvedExprKind::Char(_) => (expr.kind.clone(), Some(ResolvedType::Char)),
            ResolvedExprKind::Path(p) => {
                let checked = self.check_path(p);
                (ResolvedExprKind::Path(checked), Some(ResolvedType::Infer)) // Would be resolved
            }
            ResolvedExprKind::Unary(op, e) => {
                let checked = self.check_expr(e);
                let ty = self.unary_op_type(op, &checked);
                (ResolvedExprKind::Unary(*op, Box::new(checked)), ty)
            }
            ResolvedExprKind::Binary(op, l, r) => {
                let left = self.check_expr(l);
                let right = self.check_expr(r);
                let ty = self.binary_op_type(op, &left, &right);
                (ResolvedExprKind::Binary(*op, Box::new(left), Box::new(right)), ty)
            }
            ResolvedExprKind::Call(callee, args) => {
                let checked_callee = self.check_expr(callee);
                let checked_args: Vec<_> = args.iter().map(|a| self.check_expr(a)).collect();
                (ResolvedExprKind::Call(Box::new(checked_callee), checked_args), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::MethodCall(receiver, method, type_args, args) => {
                let checked_receiver = self.check_expr(receiver);
                let checked_args: Vec<_> = args.iter().map(|a| self.check_expr(a)).collect();
                let checked_type_args: Vec<_> = type_args.iter().map(|t| self.check_type(t)).collect();
                (ResolvedExprKind::MethodCall(Box::new(checked_receiver), method.clone(), checked_type_args, checked_args), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Field(e, field) => {
                let checked = self.check_expr(e);
                (ResolvedExprKind::Field(Box::new(checked), field.clone()), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Index(e, idx) => {
                let checked_e = self.check_expr(e);
                let checked_idx = self.check_expr(idx);
                (ResolvedExprKind::Index(Box::new(checked_e), Box::new(checked_idx)), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Range(start, end, end_kind) => {
                let checked_start = self.check_expr(start);
                let checked_end = self.check_expr(end);
                (ResolvedExprKind::Range(Box::new(checked_start), Box::new(checked_end), *end_kind), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Struct(path, fields) => {
                let checked_path = self.check_path(path);
                let checked_fields: Vec<_> = fields.iter().map(|f| TypedFieldExpr { name: f.name.clone(), expr: self.check_expr(&f.expr) }).collect();
                (ResolvedExprKind::Struct(checked_path, checked_fields), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Array(elems) => {
                let checked: Vec<_> = elems.iter().map(|e| self.check_expr(e)).collect();
                (ResolvedExprKind::Array(checked), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Tuple(elems) => {
                let checked: Vec<_> = elems.iter().map(|e| self.check_expr(e)).collect();
                (ResolvedExprKind::Tuple(checked), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Block(b) => {
                let checked = self.check_block(b, None);
                (ResolvedExprKind::Block(checked), None)
            }
            ResolvedExprKind::If(cond, then_branch, else_branch) => {
                let checked_cond = self.check_expr(cond);
                let checked_then = self.check_block(then_branch, None);
                let checked_else = else_branch.as_ref().map(|e| Box::new(self.check_expr(e)));
                (ResolvedExprKind::If(Box::new(checked_cond), checked_then, checked_else), None)
            }
            ResolvedExprKind::Match(scrutinee, arms) => {
                let checked_scrutinee = self.check_expr(scrutinee);
                let checked_arms: Vec<_> = arms.iter().map(|a| self.check_arm(a)).collect();
                (ResolvedExprKind::Match(Box::new(checked_scrutinee), checked_arms), None)
            }
            ResolvedExprKind::Loop(body, label) => {
                let checked = self.check_block(body, None);
                (ResolvedExprKind::Loop(checked, label.clone()), Some(ResolvedType::Never))
            }
            ResolvedExprKind::While(cond, body, label) => {
                let checked_cond = self.check_expr(cond);
                let checked_body = self.check_block(body, None);
                (ResolvedExprKind::While(Box::new(checked_cond), checked_body, label.clone()), Some(ResolvedType::Tuple(Vec::new())))
            }
            ResolvedExprKind::For(var, iter, body, label) => {
                let checked_iter = self.check_expr(iter);
                let checked_body = self.check_block(body, None);
                (ResolvedExprKind::For(var.clone(), Box::new(checked_iter), checked_body, label.clone()), Some(ResolvedType::Tuple(Vec::new())))
            }
            ResolvedExprKind::Break(label, value) => {
                let checked_value = value.as_ref().map(|v| Box::new(self.check_expr(v)));
                (ResolvedExprKind::Break(label.clone(), checked_value), Some(ResolvedType::Never))
            }
            ResolvedExprKind::Continue(label) => {
                (ResolvedExprKind::Continue(label.clone()), Some(ResolvedType::Never))
            }
            ResolvedExprKind::Return(value) => {
                let checked_value = value.as_ref().map(|v| Box::new(self.check_expr(v)));
                (ResolvedExprKind::Return(checked_value), Some(ResolvedType::Never))
            }
            ResolvedExprKind::Try(e) => {
                let checked = self.check_expr(e);
                (ResolvedExprKind::Try(Box::new(checked)), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Closure(c) => {
                let checked = self.check_closure(c);
                (ResolvedExprKind::Closure(checked), Some(ResolvedType::Infer))
            }
            ResolvedExprKind::Cast(e, ty) => {
                let checked = self.check_expr(e);
                let checked_ty = self.check_type(ty);
                (ResolvedExprKind::Cast(Box::new(checked), checked_ty), checked_ty)
            }
            ResolvedExprKind::Assign(l, r) => {
                let checked_l = self.check_expr(l);
                let checked_r = self.check_expr(r);
                if let Some(ty_l) = &checked_l.ty {
                    if let Some(ty_r) = &checked_r.ty {
                        self.unify(ty_l, ty_r, checked_r.span);
                    }
                }
                (ResolvedExprKind::Assign(Box::new(checked_l), Box::new(checked_r)), Some(ResolvedType::Tuple(Vec::new())))
            }
            ResolvedExprKind::AssignOp(op, l, r) => {
                let checked_l = self.check_expr(l);
                let checked_r = self.check_expr(r);
                if let Some(ty_l) = &checked_l.ty {
                    if let Some(ty_r) = &checked_r.ty {
                        self.unify(ty_l, ty_r, checked_r.span);
                    }
                }
                (ResolvedExprKind::AssignOp(*op, Box::new(checked_l), Box::new(checked_r)), Some(ResolvedType::Tuple(Vec::new())))
            }
            ResolvedExprKind::Ref(e, m) => {
                let checked = self.check_expr(e);
                let ty = checked.ty.clone().map(|t| ResolvedType::Ref(Box::new(t), *m));
                (ResolvedExprKind::Ref(Box::new(checked), *m), ty)
            }
            ResolvedExprKind::Deref(e) => {
                let checked = self.check_expr(e);
                let ty = checked.ty.clone().and_then(|t| match t {
                    ResolvedType::Ref(inner, _) | ResolvedType::RawPtr(inner, _) => Some(*inner),
                    _ => None,
                });
                (ResolvedExprKind::Deref(Box::new(checked)), ty)
            }
            ResolvedExprKind::Group(e) => {
                let checked = self.check_expr(e);
                (ResolvedExprKind::Group(Box::new(checked)), checked.ty)
            }
        };
        
        TypedExpr {
            id: expr.id,
            span: expr.span,
            kind,
            ty,
        }
    }
    
    fn check_arm(&mut self, arm: &ResolvedArm) -> TypedArm {
        let pattern = self.check_pattern(&arm.pattern);
        let guard = arm.guard.as_ref().map(|g| self.check_expr(g));
        let body = self.check_expr(&arm.body);
        
        TypedArm {
            id: arm.id,
            span: arm.span,
            pattern,
            guard,
            body,
        }
    }
    
    fn check_pattern(&mut self, pat: &ResolvedPattern) -> TypedPattern {
        match pat {
            ResolvedPattern::Wildcard => TypedPattern::Wildcard,
            ResolvedPattern::Ident(name) => TypedPattern::Ident(name.clone()),
            ResolvedPattern::Literal(e) => TypedPattern::Literal(self.check_expr_kind(e)),
            ResolvedPattern::Path(p) => TypedPattern::Path(self.check_path(p)),
            ResolvedPattern::Tuple(pats) => TypedPattern::Tuple(pats.iter().map(|p| self.check_pattern(p)).collect()),
            ResolvedPattern::Struct(path, fields) => TypedPattern::Struct(
                self.check_path(path),
                fields.iter().map(|f| TypedFieldPattern { name: f.name.clone(), pattern: self.check_pattern(&f.pattern) }).collect(),
            ),
            ResolvedPattern::Or(pats) => TypedPattern::Or(pats.iter().map(|p| self.check_pattern(p)).collect()),
            ResolvedPattern::Rest => TypedPattern::Rest,
            ResolvedPattern::Ref(p, m) => TypedPattern::Ref(Box::new(self.check_pattern(p)), *m),
        }
    }
    
    fn check_expr_kind(&mut self, kind: &ResolvedExprKind) -> TypedExprKind {
        match kind {
            ResolvedExprKind::Bool(v) => TypedExprKind::Bool(*v),
            ResolvedExprKind::Integer(v) => TypedExprKind::Integer(*v),
            ResolvedExprKind::Float(v) => TypedExprKind::Float(*v),
            ResolvedExprKind::String(v) => TypedExprKind::String(v.clone()),
            ResolvedExprKind::Char(v) => TypedExprKind::Char(*v),
            ResolvedExprKind::Path(p) => TypedExprKind::Path(self.check_path(p)),
            _ => TypedExprKind::Integer(0),
        }
    }
    
    fn check_closure(&mut self, c: &ResolvedClosure) -> TypedClosure {
        let params = c.params.iter().map(|p| TypedParam {
            id: p.id,
            span: p.span,
            name: p.name.clone(),
            ty: self.check_type(&p.ty).unwrap_or(ResolvedType::Infer),
            is_mut: p.is_mut,
            is_ref: p.is_ref,
        }).collect();
        
        let return_type = c.return_type.as_ref().map(|t| self.check_type(t)).flatten();
        let body = self.check_block(&c.body, return_type.as_ref());
        
        TypedClosure {
            params,
            return_type,
            body,
            is_move: c.is_move,
        }
    }
    
    fn check_macro(&mut self, m: &ResolvedMacroCall) -> TypedMacroCall {
        TypedMacroCall {
            id: m.id,
            span: m.span,
            path: self.check_path(&m.path),
            args: m.args.iter().map(|a| self.check_macro_arg(a)).collect(),
        }
    }
    
    fn check_macro_arg(&mut self, arg: &ResolvedMacroArg) -> TypedMacroArg {
        match arg {
            ResolvedMacroArg::Expr(e) => TypedMacroArg::Expr(self.check_expr(e)),
            ResolvedMacroArg::Stmt(s) => TypedMacroArg::Stmt(self.check_stmt(s)),
            ResolvedMacroArg::Pat(p) => TypedMacroArg::Pat(self.check_pattern(p)),
            ResolvedMacroArg::Ty(t) => TypedMacroArg::Ty(self.check_type(t).unwrap_or(ResolvedType::Infer)),
            ResolvedMacroArg::Block(b) => TypedMacroArg::Block(self.check_block(b, None)),
        }
    }
    
    fn check_path(&mut self, path: &ResolvedPath) -> TypedPath {
        TypedPath {
            segments: path.segments.iter().map(|seg| TypedPathSegment {
                name: seg.name.clone(),
                type_args: seg.type_args.iter().map(|t| self.check_type(t).unwrap_or(ResolvedType::Infer)).collect(),
            }).collect(),
            is_absolute: path.is_absolute,
        }
    }
    
    fn check_type(&mut self, ty: &ResolvedType) -> Option<ResolvedType> {
        match ty {
            ResolvedType::Infer => None,
            ResolvedType::Generic(_) => Some(ty.clone()),
            ResolvedType::Path(p) => Some(ResolvedType::Path(self.check_path(p))),
            ResolvedType::Tuple(types) => Some(ResolvedType::Tuple(types.iter().map(|t| self.check_type(t).unwrap_or(ResolvedType::Infer)).collect())),
            ResolvedType::Array(t, size) => Some(ResolvedType::Array(Box::new(self.check_type(t).unwrap_or(ResolvedType::Infer)), *size)),
            ResolvedType::Slice(t) => Some(ResolvedType::Slice(Box::new(self.check_type(t).unwrap_or(ResolvedType::Infer)))),
            ResolvedType::Ref(t, m) => Some(ResolvedType::Ref(Box::new(self.check_type(t).unwrap_or(ResolvedType::Infer)), *m)),
            ResolvedType::RawPtr(t, m) => Some(ResolvedType::RawPtr(Box::new(self.check_type(t).unwrap_or(ResolvedType::Infer)), *m)),
            ResolvedType::FnPtr(sig) => Some(ResolvedType::FnPtr(ResolvedFnSig {
                params: sig.params.iter().map(|p| self.check_type(p).unwrap_or(ResolvedType::Infer)).collect(),
                return_type: Box::new(self.check_type(&sig.return_type).unwrap_or(ResolvedType::Infer)),
                is_variadic: sig.is_variadic,
                abi: sig.abi.clone(),
            })),
            _ => Some(ty.clone()),
        }
    }
    
    fn unary_op_type(&mut self, op: &UnOp, expr: &TypedExpr) -> Option<ResolvedType> {
        match op {
            UnOp::Neg => expr.ty.clone(),
            UnOp::Not => Some(ResolvedType::Bool),
            UnOp::BitNot => expr.ty.clone(),
            UnOp::Deref => expr.ty.clone().and_then(|t| match t {
                ResolvedType::Ref(inner, _) | ResolvedType::RawPtr(inner, _) => Some(*inner),
                _ => None,
            }),
            UnOp::Ref => expr.ty.clone().map(|t| ResolvedType::Ref(Box::new(t), Mutability::Immutable)),
            UnOp::RefMut => expr.ty.clone().map(|t| ResolvedType::Ref(Box::new(t), Mutability::Mutable)),
        }
    }
    
    fn binary_op_type(&mut self, op: &BinOp, left: &TypedExpr, right: &TypedExpr) -> Option<ResolvedType> {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                // Numeric operations - return left type if both are same
                if left.ty == right.ty { left.ty.clone() } else { None }
            }
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                if left.ty == right.ty { left.ty.clone() } else { None }
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                Some(ResolvedType::Bool)
            }
            BinOp::And | BinOp::Or => {
                Some(ResolvedType::Bool)
            }
            _ => None,
        }
    }
}

/// Public check function
pub fn check(module: ResolvedModule) -> Result<TypedModule, DiagnosticBag> {
    let checker = TypeChecker::new(module);
    checker.check()
}

/// Typed AST types (simplified - in practice would be more detailed)
pub type TypedModule = ResolvedModuleData;
pub type TypedItem = ResolvedItem;
pub type TypedFunction = ResolvedFunction;
pub type TypedStruct = ResolvedStruct;
pub type TypedEnum = ResolvedEnum;
pub type TypedImpl = ResolvedImpl;
pub type TypedImplItem = ResolvedImplItem;
pub type TypedTypeAlias = ResolvedTypeAlias;
pub type TypedConst = ResolvedConst;
pub type TypedStatic = ResolvedStatic;
pub type TypedExternBlock = ResolvedExternBlock;
pub type TypedExternItem = ResolvedExternItem;
pub type TypedBlock = ResolvedBlock;
pub type TypedStmt = ResolvedStmt;
pub type TypedLetStmt = ResolvedLetStmt;
pub type TypedExpr = ResolvedExpr;
pub type TypedExprKind = ResolvedExprKind;
pub type TypedFieldExpr = ResolvedFieldExpr;
pub type TypedArm = ResolvedArm;
pub type TypedPattern = ResolvedPattern;
pub type TypedFieldPattern = ResolvedFieldPattern;
pub type TypedClosure = ResolvedClosure;
pub type TypedMacroCall = ResolvedMacroCall;
pub type TypedMacroArg = ResolvedMacroArg;
pub type TypedPath = ResolvedPath;
pub type TypedPathSegment = ResolvedPathSegment;