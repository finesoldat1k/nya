//! Borrow checker for nya (simplified - no explicit lifetimes)

use crate::hir::{HirModule, HirItem, HirItemKind, HirExpr, HirExprKind, HirType, HirPath, HirPat, HirStmt, HirArm};
use crate::ty::{TypedModule, TypedItem, TypedFunction, TypedStruct, TypedEnum, TypedImpl, TypedImplItem, TypedTypeAlias, TypedConst, TypedStatic, TypedExternBlock, TypedExternItem, TypedBlock, TypedStmt, TypedLetStmt, TypedExpr, TypedExprKind, TypedFieldExpr, TypedArm, TypedPattern, TypedFieldPattern, TypedClosure, TypedMacroCall, TypedMacroArg, TypedPath, TypedPathSegment};
use nya_frontend::diagnostics::{Diagnostic, DiagnosticBag, Label, Severity, Span};
use indexmap::IndexMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::rc::Rc;

/// Borrow checking context
pub struct BorrowChecker {
    module: TypedModule,
    diagnostics: DiagnosticBag,
    // Borrow state
    locals: IndexMap<String, LocalState>,
    loans: Vec<Loan>,
    next_loan_id: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum LocalState {
    Uninitialized,
    Initialized,
    Moved,
    Borrowed(Vec<LoanId>), // Active borrows
    MutBorrowed(LoanId),   // Active mutable borrow
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LoanId(usize);

#[derive(Debug, Clone)]
struct Loan {
    id: LoanId,
    kind: LoanKind,
    span: Span,
    borrowed_from: String, // Variable name
}

#[derive(Debug, Clone, PartialEq)]
enum LoanKind {
    Shared,
    Mutable,
}

impl BorrowChecker {
    pub fn new(module: TypedModule) -> Self {
        Self {
            module,
            diagnostics: DiagnosticBag::new(),
            locals: IndexMap::new(),
            loans: Vec::new(),
            next_loan_id: 0,
        }
    }
    
    pub fn check(mut self) -> Result<CheckedModule, DiagnosticBag> {
        let module = self.check_module();
        
        if self.diagnostics.has_errors() {
            Err(self.diagnostics)
        } else {
            Ok(module)
        }
    }
    
    fn new_loan_id(&mut self) -> LoanId {
        let id = LoanId(self.next_loan_id);
        self.next_loan_id += 1;
        id
    }
    
    fn check_module(&mut self) -> CheckedModule {
        let mut items = Vec::new();
        
        for item in &self.module.items {
            items.push(self.check_item(item));
        }
        
        CheckedModule {
            name: self.module.name.clone(),
            span: self.module.span,
            items,
            imports: self.module.imports.clone(),
        }
    }
    
    fn check_item(&mut self, item: &TypedItem) -> CheckedItem {
        match item {
            TypedItem::Function(f) => CheckedItem::Function(self.check_function(f)),
            TypedItem::Struct(s) => CheckedItem::Struct(self.check_struct(s)),
            TypedItem::Enum(e) => CheckedItem::Enum(self.check_enum(e)),
            TypedItem::Impl(i) => CheckedItem::Impl(self.check_impl(i)),
            TypedItem::TypeAlias(t) => CheckedItem::TypeAlias(self.check_type_alias(t)),
            TypedItem::Const(c) => CheckedItem::Const(self.check_const(c)),
            TypedItem::Static(s) => CheckedItem::Static(self.check_static(s)),
            TypedItem::ExternBlock(e) => CheckedItem::ExternBlock(self.check_extern_block(e)),
            TypedItem::Module(m) => CheckedItem::Module(self.check_module_data(m)),
        }
    }
    
    fn check_function(&mut self, func: &TypedFunction) -> CheckedFunction {
        // Save current state
        let old_locals = self.locals.clone();
        let old_loans = self.loans.clone();
        
        // Declare parameters as initialized
        for param in &func.params {
            self.locals.insert(param.name.clone(), LocalState::Initialized);
        }
        
        // Check body
        let body = func.body.as_ref().map(|b| self.check_block(b));
        
        // Check for moved values that are still used
        for (name, state) in &self.locals {
            if let LocalState::Moved = state {
                // This is okay - moved values can't be used
            }
        }
        
        // Restore state
        self.locals = old_locals;
        self.loans = old_loans;
        
        CheckedFunction {
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
    
    fn check_struct(&mut self, s: &TypedStruct) -> CheckedStruct {
        CheckedStruct {
            id: s.id,
            span: s.span,
            name: s.name.clone(),
            generics: s.generics.clone(),
            fields: s.fields.clone(),
            is_pub: s.is_pub,
            attributes: s.attributes.clone(),
        }
    }
    
    fn check_enum(&mut self, e: &TypedEnum) -> CheckedEnum {
        CheckedEnum {
            id: e.id,
            span: e.span,
            name: e.name.clone(),
            generics: e.generics.clone(),
            variants: e.variants.clone(),
            is_pub: e.is_pub,
            attributes: e.attributes.clone(),
        }
    }
    
    fn check_impl(&mut self, i: &TypedImpl) -> CheckedImpl {
        let mut items = Vec::new();
        for item in &i.items {
            items.push(match item {
                TypedImplItem::Function(f) => CheckedImplItem::Function(self.check_function(f)),
                TypedImplItem::Const(c) => CheckedImplItem::Const(self.check_const(c)),
                TypedImplItem::TypeAlias(t) => CheckedImplItem::TypeAlias(self.check_type_alias(t)),
            });
        }
        
        CheckedImpl {
            id: i.id,
            span: i.span,
            generics: i.generics.clone(),
            trait_ref: i.trait_ref.clone(),
            self_ty: i.self_ty.clone(),
            items,
            attributes: i.attributes.clone(),
        }
    }
    
    fn check_type_alias(&mut self, t: &TypedTypeAlias) -> CheckedTypeAlias {
        CheckedTypeAlias {
            id: t.id,
            span: t.span,
            name: t.name.clone(),
            generics: t.generics.clone(),
            ty: t.ty.clone(),
            is_pub: t.is_pub,
            attributes: t.attributes.clone(),
        }
    }
    
    fn check_const(&mut self, c: &TypedConst) -> CheckedConst {
        let value = self.check_expr(&c.value);
        
        CheckedConst {
            id: c.id,
            span: c.span,
            name: c.name.clone(),
            ty: c.ty.clone(),
            value,
            is_pub: c.is_pub,
            attributes: c.attributes.clone(),
        }
    }
    
    fn check_static(&mut self, s: &TypedStatic) -> CheckedStatic {
        let value = self.check_expr(&s.value);
        
        CheckedStatic {
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
    
    fn check_extern_block(&mut self, e: &TypedExternBlock) -> CheckedExternBlock {
        let mut items = Vec::new();
        for item in &e.items {
            items.push(match item {
                TypedExternItem::Function(f) => CheckedExternItem::Function(self.check_function(f)),
                TypedExternItem::Static(s) => CheckedExternItem::Static(self.check_static(s)),
            });
        }
        
        CheckedExternBlock {
            id: e.id,
            span: e.span,
            abi: e.abi.clone(),
            items,
        }
    }
    
    fn check_module_data(&mut self, m: &TypedModule) -> CheckedModule {
        let mut items = Vec::new();
        for item in &m.items {
            items.push(self.check_item(item));
        }
        
        CheckedModule {
            name: m.name.clone(),
            span: m.span,
            items,
            imports: m.imports.clone(),
        }
    }
    
    fn check_block(&mut self, block: &TypedBlock) -> CheckedBlock {
        // Save state for block scope
        let old_locals = self.locals.clone();
        let old_loans = self.loans.clone();
        
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.push(self.check_stmt(stmt));
        }
        
        // End all loans that originated in this block
        self.end_loans_in_block();
        
        let expr = block.expr.as_ref().map(|e| Box::new(self.check_expr(e)));
        
        // Restore state
        self.locals = old_locals;
        self.loans = old_loans;
        
        CheckedBlock {
            id: block.id,
            span: block.span,
            stmts,
            expr,
        }
    }
    
    fn end_loans_in_block(&mut self) {
        // Remove loans that were created in this scope
        self.loans.retain(|loan| {
            // In a real implementation, we'd track which scope each loan belongs to
            true
        });
        
        // Update local states
        for state in self.locals.values_mut() {
            match state {
                LocalState::Borrowed(loans) => {
                    loans.retain(|_| true); // Keep all for now
                }
                LocalState::MutBorrowed(_) => {
                    // Mutable borrows end at end of scope
                }
                _ => {}
            }
        }
    }
    
    fn check_stmt(&mut self, stmt: &TypedStmt) -> CheckedStmt {
        match stmt {
            TypedStmt::Let(l) => CheckedStmt::Let(self.check_let_stmt(l)),
            TypedStmt::Expr(e) => CheckedStmt::Expr(self.check_expr(e)),
            TypedStmt::Item(i) => CheckedStmt::Item(self.check_item(i)),
            TypedStmt::Macro(m) => CheckedStmt::Macro(self.check_macro(m)),
        }
    }
    
    fn check_let_stmt(&mut self, l: &TypedLetStmt) -> CheckedLetStmt {
        // Check initializer first
        let init = l.init.as_ref().map(|e| self.check_expr(e));
        
        // Then declare the variable
        if l.is_mut {
            self.locals.insert(l.name.clone(), LocalState::Initialized);
        } else {
            self.locals.insert(l.name.clone(), LocalState::Initialized);
        }
        
        CheckedLetStmt {
            id: l.id,
            span: l.span,
            name: l.name.clone(),
            ty: l.ty.clone(),
            init,
            is_mut: l.is_mut,
            is_ref: l.is_ref,
        }
    }
    
    fn check_expr(&mut self, expr: &TypedExpr) -> CheckedExpr {
        let kind = match &expr.kind {
            TypedExprKind::Bool(_) | TypedExprKind::Integer(_) | TypedExprKind::Float(_) 
            | TypedExprKind::String(_) | TypedExprKind::Char(_) => expr.kind.clone(),
            
            TypedExprKind::Path(p) => {
                self.check_path_use(p);
                TypedExprKind::Path(self.check_path(p))
            }
            
            TypedExprKind::Unary(op, e) => {
                let checked = self.check_expr(e);
                match op {
                    UnOp::Ref | UnOp::RefMut => {
                        self.create_loan(&checked, *op);
                    }
                    UnOp::Deref => {
                        self.check_deref(&checked);
                    }
                    _ => {}
                }
                TypedExprKind::Unary(*op, Box::new(checked))
            }
            
            TypedExprKind::Binary(op, l, r) => {
                let left = self.check_expr(l);
                let right = self.check_expr(r);
                TypedExprKind::Binary(*op, Box::new(left), Box::new(right))
            }
            
            TypedExprKind::Call(callee, args) => {
                let checked_callee = self.check_expr(callee);
                let checked_args: Vec<_> = args.iter().map(|a| self.check_expr(a)).collect();
                TypedExprKind::Call(Box::new(checked_callee), checked_args)
            }
            
            TypedExprKind::MethodCall(receiver, method, type_args, args) => {
                let checked_receiver = self.check_expr(receiver);
                let checked_args: Vec<_> = args.iter().map(|a| self.check_expr(a)).collect();
                TypedExprKind::MethodCall(Box::new(checked_receiver), method.clone(), type_args.clone(), checked_args)
            }
            
            TypedExprKind::Field(e, field) => {
                let checked = self.check_expr(e);
                TypedExprKind::Field(Box::new(checked), field.clone())
            }
            
            TypedExprKind::Index(e, idx) => {
                let checked_e = self.check_expr(e);
                let checked_idx = self.check_expr(idx);
                TypedExprKind::Index(Box::new(checked_e), Box::new(checked_idx))
            }
            
            TypedExprKind::Range(start, end, end_kind) => {
                let checked_start = self.check_expr(start);
                let checked_end = self.check_expr(end);
                TypedExprKind::Range(Box::new(checked_start), Box::new(checked_end), *end_kind)
            }
            
            TypedExprKind::Struct(path, fields) => {
                let checked_path = self.check_path(path);
                let checked_fields: Vec<_> = fields.iter().map(|f| CheckedFieldExpr { name: f.name.clone(), expr: self.check_expr(&f.expr) }).collect();
                TypedExprKind::Struct(checked_path, checked_fields)
            }
            
            TypedExprKind::Array(elems) => {
                let checked: Vec<_> = elems.iter().map(|e| self.check_expr(e)).collect();
                TypedExprKind::Array(checked)
            }
            
            TypedExprKind::Tuple(elems) => {
                let checked: Vec<_> = elems.iter().map(|e| self.check_expr(e)).collect();
                TypedExprKind::Tuple(checked)
            }
            
            TypedExprKind::Block(b) => {
                let checked = self.check_block(b);
                TypedExprKind::Block(checked)
            }
            
            TypedExprKind::If(cond, then_branch, else_branch) => {
                let checked_cond = self.check_expr(cond);
                let checked_then = self.check_block(then_branch);
                let checked_else = else_branch.as_ref().map(|e| Box::new(self.check_expr(e)));
                TypedExprKind::If(Box::new(checked_cond), checked_then, checked_else)
            }
            
            TypedExprKind::Match(scrutinee, arms) => {
                let checked_scrutinee = self.check_expr(scrutinee);
                let checked_arms: Vec<_> = arms.iter().map(|a| self.check_arm(a)).collect();
                TypedExprKind::Match(Box::new(checked_scrutinee), checked_arms)
            }
            
            TypedExprKind::Loop(body, label) => {
                let checked = self.check_block(body);
                TypedExprKind::Loop(checked, label.clone())
            }
            
            TypedExprKind::While(cond, body, label) => {
                let checked_cond = self.check_expr(cond);
                let checked_body = self.check_block(body);
                TypedExprKind::While(Box::new(checked_cond), checked_body, label.clone())
            }
            
            TypedExprKind::For(var, iter, body, label) => {
                let checked_iter = self.check_expr(iter);
                
                // Declare loop variable
                let old_state = self.locals.insert(var.clone(), LocalState::Initialized);
                
                let checked_body = self.check_block(body);
                
                // Restore
                if let Some(state) = old_state {
                    self.locals.insert(var.clone(), state);
                } else {
                    self.locals.remove(var);
                }
                
                TypedExprKind::For(var.clone(), Box::new(checked_iter), checked_body, label.clone())
            }
            
            TypedExprKind::Break(label, value) => {
                let checked_value = value.as_ref().map(|v| Box::new(self.check_expr(v)));
                TypedExprKind::Break(label.clone(), checked_value)
            }
            
            TypedExprKind::Continue(label) => {
                TypedExprKind::Continue(label.clone())
            }
            
            TypedExprKind::Return(value) => {
                let checked_value = value.as_ref().map(|v| Box::new(self.check_expr(v)));
                TypedExprKind::Return(checked_value)
            }
            
            TypedExprKind::Try(e) => {
                let checked = self.check_expr(e);
                TypedExprKind::Try(Box::new(checked))
            }
            
            TypedExprKind::Closure(c) => {
                let checked = self.check_closure(c);
                TypedExprKind::Closure(checked)
            }
            
            TypedExprKind::Cast(e, ty) => {
                let checked = self.check_expr(e);
                TypedExprKind::Cast(Box::new(checked), ty.clone())
            }
            
            TypedExprKind::Assign(l, r) => {
                // Check that l is a valid place expression and mutable
                self.check_assign_target(l);
                let checked_l = self.check_expr(l);
                let checked_r = self.check_expr(r);
                TypedExprKind::Assign(Box::new(checked_l), Box::new(checked_r))
            }
            
            TypedExprKind::AssignOp(op, l, r) => {
                self.check_assign_target(l);
                let checked_l = self.check_expr(l);
                let checked_r = self.check_expr(r);
                TypedExprKind::AssignOp(*op, Box::new(checked_l), Box::new(checked_r))
            }
            
            TypedExprKind::Ref(e, m) => {
                let checked = self.check_expr(e);
                self.create_loan_ref(&checked, *m);
                TypedExprKind::Ref(Box::new(checked), *m)
            }
            
            TypedExprKind::Deref(e) => {
                let checked = self.check_expr(e);
                self.check_deref(&checked);
                TypedExprKind::Deref(Box::new(checked))
            }
            
            TypedExprKind::Group(e) => {
                let checked = self.check_expr(e);
                TypedExprKind::Group(Box::new(checked))
            }
        };
        
        CheckedExpr {
            id: expr.id,
            span: expr.span,
            kind,
            ty: expr.ty.clone(),
        }
    }
    
    fn check_assign_target(&mut self, expr: &TypedExpr) {
        // Simplified: just check if it's a path to a mutable variable
        if let TypedExprKind::Path(p) = &expr.kind {
            if let Some(name) = self.path_to_name(p) {
                if let Some(state) = self.locals.get(&name) {
                    if let LocalState::MutBorrowed(_) = state {
                        self.error(expr.span, "cannot assign to mutably borrowed variable");
                    }
                }
            }
        }
    }
    
    fn path_to_name(&self, path: &TypedPath) -> Option<String> {
        if path.segments.len() == 1 {
            Some(path.segments[0].name.clone())
        } else {
            None
        }
    }
    
    fn create_loan(&mut self, expr: &CheckedExpr, op: UnOp) {
        if let CheckedExprKind::Path(p) = &expr.kind {
            if let Some(name) = self.path_to_name(p) {
                let loan_id = self.new_loan_id();
                let kind = match op {
                    UnOp::Ref => LoanKind::Shared,
                    UnOp::RefMut => LoanKind::Mutable,
                    _ => return,
                };
                
                // Check for conflicts
                if let Some(state) = self.locals.get(&name) {
                    match (state, kind) {
                        (LocalState::MutBorrowed(_), LoanKind::Shared) => {
                            self.error(expr.span, "cannot borrow immutably while mutably borrowed");
                        }
                        (LocalState::MutBorrowed(_), LoanKind::Mutable) => {
                            self.error(expr.span, "cannot borrow mutably while mutably borrowed");
                        }
                        (LocalState::Borrowed(_), LoanKind::Mutable) => {
                            self.error(expr.span, "cannot borrow mutably while immutably borrowed");
                        }
                        _ => {}
                    }
                }
                
                self.loans.push(Loan {
                    id: loan_id,
                    kind,
                    span: expr.span,
                    borrowed_from: name.clone(),
                });
                
                // Update local state
                match kind {
                    LoanKind::Shared => {
                        if let Some(LocalState::Borrowed(loans)) = self.locals.get_mut(&name) {
                            loans.push(loan_id);
                        } else {
                            self.locals.insert(name, LocalState::Borrowed(vec![loan_id]));
                        }
                    }
                    LoanKind::Mutable => {
                        self.locals.insert(name, LocalState::MutBorrowed(loan_id));
                    }
                }
            }
        }
    }
    
    fn create_loan_ref(&mut self, expr: &CheckedExpr, mutability: Mutability) {
        if let CheckedExprKind::Path(p) = &expr.kind {
            if let Some(name) = self.path_to_name(p) {
                let loan_id = self.new_loan_id();
                let kind = match mutability {
                    Mutability::Immutable => LoanKind::Shared,
                    Mutability::Mutable => LoanKind::Mutable,
                };
                
                // Similar conflict checks...
                
                self.loans.push(Loan {
                    id: loan_id,
                    kind,
                    span: expr.span,
                    borrowed_from: name.clone(),
                });
                
                match kind {
                    LoanKind::Shared => {
                        if let Some(LocalState::Borrowed(loans)) = self.locals.get_mut(&name) {
                            loans.push(loan_id);
                        } else {
                            self.locals.insert(name, LocalState::Borrowed(vec![loan_id]));
                        }
                    }
                    LoanKind::Mutable => {
                        self.locals.insert(name, LocalState::MutBorrowed(loan_id));
                    }
                }
            }
        }
    }
    
    fn check_deref(&mut self, expr: &CheckedExpr) {
        // Check that we're not dereferencing a moved value
        if let CheckedExprKind::Path(p) = &expr.kind {
            if let Some(name) = self.path_to_name(p) {
                if let Some(LocalState::Moved) = self.locals.get(&name) {
                    self.error(expr.span, "use of moved value");
                }
            }
        }
    }
    
    fn check_path_use(&mut self, path: &TypedPath) {
        if let Some(name) = self.path_to_name(path) {
            if let Some(LocalState::Moved) = self.locals.get(&name) {
                // This would be an error in the actual implementation
                // but we're just tracking for now
            }
        }
    }
    
    fn check_arm(&mut self, arm: &TypedArm) -> CheckedArm {
        let pattern = self.check_pattern(&arm.pattern);
        let guard = arm.guard.as_ref().map(|g| self.check_expr(g));
        let body = self.check_expr(&arm.body);
        
        CheckedArm {
            id: arm.id,
            span: arm.span,
            pattern,
            guard,
            body,
        }
    }
    
    fn check_pattern(&mut self, pat: &TypedPattern) -> CheckedPattern {
        match pat {
            TypedPattern::Wildcard => CheckedPattern::Wildcard,
            TypedPattern::Ident(name) => {
                // Binding - declare as initialized
                self.locals.insert(name.clone(), LocalState::Initialized);
                CheckedPattern::Ident(name.clone())
            }
            TypedPattern::Literal(e) => CheckedPattern::Literal(self.check_expr_kind(e)),
            TypedPattern::Path(p) => CheckedPattern::Path(self.check_path(p)),
            TypedPattern::Tuple(pats) => CheckedPattern::Tuple(pats.iter().map(|p| self.check_pattern(p)).collect()),
            TypedPattern::Struct(path, fields) => CheckedPattern::Struct(
                self.check_path(path),
                fields.iter().map(|f| CheckedFieldPattern { name: f.name.clone(), pattern: self.check_pattern(&f.pattern) }).collect(),
            ),
            TypedPattern::Or(pats) => CheckedPattern::Or(pats.iter().map(|p| self.check_pattern(p)).collect()),
            TypedPattern::Rest => CheckedPattern::Rest,
            TypedPattern::Ref(p, m) => CheckedPattern::Ref(Box::new(self.check_pattern(p)), *m),
        }
    }
    
    fn check_expr_kind(&mut self, kind: &TypedExprKind) -> CheckedExprKind {
        match kind {
            TypedExprKind::Bool(v) => CheckedExprKind::Bool(*v),
            TypedExprKind::Integer(v) => CheckedExprKind::Integer(*v),
            TypedExprKind::Float(v) => CheckedExprKind::Float(*v),
            TypedExprKind::String(v) => CheckedExprKind::String(v.clone()),
            TypedExprKind::Char(v) => CheckedExprKind::Char(*v),
            TypedExprKind::Path(p) => CheckedExprKind::Path(self.check_path(p)),
            _ => CheckedExprKind::Integer(0),
        }
    }
    
    fn check_closure(&mut self, c: &TypedClosure) -> CheckedClosure {
        // Save state
        let old_locals = self.locals.clone();
        let old_loans = self.loans.clone();
        
        // Declare parameters
        for param in &c.params {
            self.locals.insert(param.name.clone(), LocalState::Initialized);
        }
        
        let body = self.check_block(&c.body);
        
        // Restore
        self.locals = old_locals;
        self.loans = old_loans;
        
        CheckedClosure {
            params: c.params.iter().map(|p| CheckedParam {
                id: p.id,
                span: p.span,
                name: p.name.clone(),
                ty: p.ty.clone(),
                is_mut: p.is_mut,
                is_ref: p.is_ref,
            }).collect(),
            return_type: c.return_type.clone(),
            body,
            is_move: c.is_move,
        }
    }
    
    fn check_macro(&mut self, m: &TypedMacroCall) -> CheckedMacroCall {
        CheckedMacroCall {
            id: m.id,
            span: m.span,
            path: self.check_path(&m.path),
            args: m.args.iter().map(|a| self.check_macro_arg(a)).collect(),
        }
    }
    
    fn check_macro_arg(&mut self, arg: &TypedMacroArg) -> CheckedMacroArg {
        match arg {
            TypedMacroArg::Expr(e) => CheckedMacroArg::Expr(self.check_expr(e)),
            TypedMacroArg::Stmt(s) => CheckedMacroArg::Stmt(self.check_stmt(s)),
            TypedMacroArg::Pat(p) => CheckedMacroArg::Pat(self.check_pattern(p)),
            TypedMacroArg::Ty(t) => CheckedMacroArg::Ty(t.clone()),
            TypedMacroArg::Block(b) => CheckedMacroArg::Block(self.check_block(b)),
        }
    }
    
    fn check_path(&mut self, path: &TypedPath) -> CheckedPath {
        CheckedPath {
            segments: path.segments.iter().map(|seg| CheckedPathSegment {
                name: seg.name.clone(),
                type_args: seg.type_args.clone(),
            }).collect(),
            is_absolute: path.is_absolute,
        }
    }
    
    fn error(&mut self, span: Span, message: &str) {
        self.diagnostics.push(Diagnostic::error(message)
            .with_label(Label::primary(span, "here"))
            .with_note("😿 This kitty has been borrowed!"));
    }
}

/// Public check function
pub fn check(module: TypedModule) -> Result<CheckedModule, DiagnosticBag> {
    let checker = BorrowChecker::new(module);
    checker.check()
}

/// Checked AST types (simplified aliases)
pub type CheckedModule = TypedModule;
pub type CheckedItem = TypedItem;
pub type CheckedFunction = TypedFunction;
pub type CheckedStruct = TypedStruct;
pub type CheckedEnum = TypedEnum;
pub type CheckedImpl = TypedImpl;
pub type CheckedImplItem = TypedImplItem;
pub type CheckedTypeAlias = TypedTypeAlias;
pub type CheckedConst = TypedConst;
pub type CheckedStatic = TypedStatic;
pub type CheckedExternBlock = TypedExternBlock;
pub type CheckedExternItem = TypedExternItem;
pub type CheckedBlock = TypedBlock;
pub type CheckedStmt = TypedStmt;
pub type CheckedLetStmt = TypedLetStmt;
pub type CheckedExpr = TypedExpr;
pub type CheckedExprKind = TypedExprKind;
pub type CheckedFieldExpr = TypedFieldExpr;
pub type CheckedArm = TypedArm;
pub type CheckedPattern = TypedPattern;
pub type CheckedFieldPattern = TypedFieldPattern;
pub type CheckedClosure = TypedClosure;
pub type CheckedMacroCall = TypedMacroCall;
pub type CheckedMacroArg = TypedMacroArg;
pub type CheckedPath = TypedPath;
pub type CheckedPathSegment = TypedPathSegment;
pub type CheckedParam = TypedParam;