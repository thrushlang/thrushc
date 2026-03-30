/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

use thrustc_ast::{
    Ast,
    traits::{AstCodeLocation, AstGetType, AstStandardExtensions},
};

use thrustc_diagnostician::Diagnostician;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_options::{CompilationUnit, CompilerOptions};
use thrustc_span::Span;
use thrustc_typesystem::{
    Type,
    traits::{TypeCodeLocation, TypeIsExtensions, VoidTypeExtensions},
};

use crate::{
    context::{TypeCheckerControlContext, TypeCheckerTypeContext},
    metadata::TypeCheckerNodeMetadata,
    table::TypeCheckerSymbolsTable,
};

mod checking;
mod context;
mod expressions;
mod globals;
mod metadata;
mod operations;
mod table;

#[derive(Debug)]
pub struct TypeChecker<'type_checker> {
    ast: &'type_checker [Ast<'type_checker>],
    position: usize,

    bugs: Vec<CompilationIssue>,
    errors: Vec<CompilationIssue>,
    warnings: Vec<CompilationIssue>,

    control_context: TypeCheckerControlContext,
    type_context: TypeCheckerTypeContext<'type_checker>,
    table: TypeCheckerSymbolsTable<'type_checker>,
    diagnostician: Diagnostician,
}

impl<'type_checker> TypeChecker<'type_checker> {
    pub fn new(
        ast: &'type_checker [Ast<'type_checker>],
        file: &'type_checker CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        Self {
            ast,
            position: 0,

            bugs: Vec::with_capacity(u8::MAX as usize),
            errors: Vec::with_capacity(u8::MAX as usize),
            warnings: Vec::with_capacity(u8::MAX as usize),

            control_context: TypeCheckerControlContext::new(),
            type_context: TypeCheckerTypeContext::new(),
            table: TypeCheckerSymbolsTable::new(),
            diagnostician: Diagnostician::new(file, options),
        }
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    pub fn start(&mut self) -> bool {
        self.parse_top();

        while !self.is_eof() {
            let node: &Ast = self.peek();

            match self.analyze_decl(node) {
                Ok(()) => (),
                Err(error) => {
                    self.add_error(error);

                    {
                        let context: &mut TypeCheckerControlContext =
                            self.get_mut_control_context();

                        context.reset_checking_depth();
                        context.reset_type_cast_depth();

                        self.get_mut_type_context().unset_current_function_type();
                    }
                }
            }

            self.advance();
        }

        for warning in self.warnings.iter() {
            self.diagnostician
                .dispatch_diagnostic(warning, thrustc_logging::LoggingType::Warning);
        }

        if !self.errors.is_empty() || !self.bugs.is_empty() {
            {
                for bug in self.bugs.iter() {
                    self.diagnostician
                        .dispatch_diagnostic(bug, thrustc_logging::LoggingType::Bug);
                }

                for error in self.errors.iter() {
                    self.diagnostician
                        .dispatch_diagnostic(error, thrustc_logging::LoggingType::Error);
                }
            }

            true
        } else {
            false
        }
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    pub fn analyze_decl(&mut self, node: &'type_checker Ast) -> Result<(), CompilationIssue> {
        match node {
            Ast::Intrinsic { .. } | Ast::AssemblerFunction { .. } | Ast::Function { .. } => {
                globals::functions::validate(self, node)
            }
            Ast::CustomType { .. } | Ast::GlobalAssembler { .. } | Ast::Struct { .. } => Ok(()),
            Ast::Enum { data, .. } => {
                {
                    for (_, target_type, expr) in data.iter() {
                        let from_type: &Type = expr.get_value_type()?;

                        let metadata: TypeCheckerNodeMetadata =
                            TypeCheckerNodeMetadata::new(expr.is_literal_value());

                        {
                            let control_context: &mut TypeCheckerControlContext =
                                self.get_mut_control_context();

                            checking::check_types(
                                target_type,
                                from_type,
                                Some(expr),
                                None,
                                metadata,
                                expr.get_span(),
                                control_context,
                            )?;

                            control_context.reset_checking_depth();
                        }

                        self.analyze_expr(expr)?;
                    }
                }

                Ok(())
            }
            Ast::Static {
                kind: static_type,
                value,
                ..
            } => {
                let Some(value) = value else {
                    return Ok(());
                };

                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(value.is_literal_value());

                let value_type: &Type = value.get_value_type()?;

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        static_type,
                        value_type,
                        Some(value),
                        None,
                        metadata,
                        node.get_span(),
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                self.analyze_expr(value)?;

                Ok(())
            }
            Ast::Const {
                kind: target_type,
                value,
                ..
            } => {
                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(value.is_literal_value());

                let from_type: &Type = value.get_value_type()?;

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        target_type,
                        &Type::Const(from_type.clone().into(), from_type.get_span()),
                        Some(value),
                        None,
                        metadata,
                        node.get_span(),
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                self.analyze_expr(value)?;

                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn analyze_stmt(&mut self, node: &'type_checker Ast) -> Result<(), CompilationIssue> {
        match node {
            Ast::CustomType { .. }
            | Ast::Struct { .. }
            | Ast::Continue { .. }
            | Ast::ContinueAll { .. }
            | Ast::Break { .. }
            | Ast::BreakAll { .. } => Ok(()),

            Ast::Enum { data, .. } => {
                {
                    for (_, target_type, expr) in data.iter() {
                        let from_type: &Type = expr.get_value_type()?;

                        let metadata: TypeCheckerNodeMetadata =
                            TypeCheckerNodeMetadata::new(expr.is_literal_value());

                        {
                            let control_context: &mut TypeCheckerControlContext =
                                self.get_mut_control_context();

                            checking::check_types(
                                target_type,
                                from_type,
                                Some(expr),
                                None,
                                metadata,
                                expr.get_span(),
                                control_context,
                            )?;

                            control_context.reset_checking_depth();
                        }

                        self.analyze_expr(expr)?;
                    }
                }

                Ok(())
            }
            Ast::Static {
                kind: static_type,
                value,
                ..
            } => {
                let Some(value) = value else {
                    return Ok(());
                };

                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(value.is_literal_value());

                let value_type: &Type = value.get_value_type()?;

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        static_type,
                        value_type,
                        Some(value),
                        None,
                        metadata,
                        node.get_span(),
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                self.analyze_expr(value)?;

                Ok(())
            }
            Ast::Const {
                kind: target_type,
                value,
                ..
            } => {
                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(value.is_literal_value());

                let from_type: &Type = value.get_value_type()?;

                let fixed_from_type: &Type = if !from_type.is_const_type() {
                    &Type::Const(from_type.clone().into(), from_type.get_span())
                } else {
                    from_type
                };

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        target_type,
                        fixed_from_type,
                        Some(value),
                        None,
                        metadata,
                        node.get_span(),
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                self.analyze_expr(value)?;

                Ok(())
            }
            Ast::Local {
                name,
                kind: local_type,
                value,
                span,
                ..
            } => {
                self.get_mut_table().new_local(name, (local_type, *span));

                if local_type.contains_void_type() || local_type.is_void_type() {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0019,
                        "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
                        None,
                        *span,
                    ));
                }

                let Some(local_value) = value else {
                    return Ok(());
                };

                let type_metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(local_value.is_literal_value());

                let local_value_type: &Type = local_value.get_value_type()?;
                let is_ptr_type: bool = local_value_type.is_ptr_like_type();

                if is_ptr_type {
                    let fixed_type: Type = Type::Ptr(
                        Some(local_value_type.clone().into()),
                        local_value_type.get_span(),
                    );

                    {
                        let control_context: &mut TypeCheckerControlContext =
                            self.get_mut_control_context();

                        checking::check_types(
                            local_type,
                            &fixed_type,
                            Some(local_value),
                            None,
                            type_metadata,
                            node.get_span(),
                            control_context,
                        )?;

                        control_context.reset_checking_depth();
                    }
                } else {
                    {
                        let control_context: &mut TypeCheckerControlContext =
                            self.get_mut_control_context();

                        checking::check_types(
                            local_type,
                            local_value_type,
                            Some(local_value),
                            None,
                            type_metadata,
                            node.get_span(),
                            control_context,
                        )?;

                        control_context.reset_checking_depth();
                    }
                }

                self.analyze_expr(local_value)?;

                Ok(())
            }
            Ast::Block { nodes, post, .. } => {
                self.begin_scope();

                {
                    for node in nodes.iter() {
                        self.analyze_stmt(node)?;
                    }

                    for postnode in post.iter() {
                        self.analyze_stmt(postnode)?;
                    }
                }

                self.end_scope();

                Ok(())
            }
            Ast::Defer { node, .. } => {
                self.analyze_stmt(node)?;

                Ok(())
            }

            Ast::If {
                condition,
                block,
                elseif,
                anyway,
                ..
            } => {
                self.analyze_expr(condition)?;

                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(condition.is_literal_value());

                let span: Span = condition.get_span();

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        &Type::Bool(span),
                        condition.get_value_type()?,
                        Some(condition),
                        None,
                        metadata,
                        span,
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                for node in elseif.iter() {
                    self.analyze_stmt(node)?;
                }

                if let Some(otherwise) = anyway {
                    self.analyze_stmt(otherwise)?;
                }

                self.analyze_stmt(block)?;

                Ok(())
            }
            Ast::Elif {
                condition, block, ..
            } => {
                self.analyze_expr(condition)?;

                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(condition.is_literal_value());

                let span: Span = condition.get_span();

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        &Type::Bool(condition.get_span()),
                        condition.get_value_type()?,
                        Some(condition),
                        None,
                        metadata,
                        span,
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                self.analyze_stmt(block)?;

                Ok(())
            }
            Ast::Else { block, .. } => {
                self.analyze_stmt(block)?;

                Ok(())
            }

            Ast::For {
                local,
                condition,
                actions,
                block,
                ..
            } => {
                self.analyze_stmt(local)?;

                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(condition.is_literal_value());

                let span: Span = condition.get_span();

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        &Type::Bool(span),
                        condition.get_value_type()?,
                        Some(condition),
                        None,
                        metadata,
                        span,
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                self.analyze_expr(condition)?;
                self.analyze_expr(actions)?;
                self.analyze_stmt(block)?;

                Ok(())
            }
            Ast::While {
                variable,
                condition,
                block,
                ..
            } => {
                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(condition.is_literal_value());

                let span: Span = condition.get_span();

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        &Type::Bool(span),
                        condition.get_value_type()?,
                        Some(condition),
                        None,
                        metadata,
                        span,
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                if let Some(variable) = variable {
                    self.analyze_stmt(variable)?;
                }

                self.analyze_expr(condition)?;
                self.analyze_stmt(block)?;

                Ok(())
            }
            Ast::Loop { block, .. } => {
                self.analyze_stmt(block)?;

                Ok(())
            }

            Ast::Return { expression, .. } => {
                let Some(expr) = expression else {
                    return Ok(());
                };

                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(expr.is_literal_value());

                let Some((return_type, function_loc)) =
                    self.get_type_context().get_current_function_type()
                else {
                    return Err(CompilationIssue::Error(
                        CompilationIssueCode::E0020,
                        "Return statement outside of a function.".into(),
                        None,
                        expr.get_span(),
                    ));
                };

                {
                    let control_context: &mut TypeCheckerControlContext =
                        self.get_mut_control_context();

                    checking::check_types(
                        return_type,
                        expr.get_value_type()?,
                        Some(expr),
                        None,
                        metadata,
                        function_loc,
                        control_context,
                    )?;

                    control_context.reset_checking_depth();
                }

                self.analyze_expr(expr)?;

                Ok(())
            }
            Ast::Mut { source, value, .. } => {
                let metadata: TypeCheckerNodeMetadata =
                    TypeCheckerNodeMetadata::new(value.is_literal_value());

                let value_type: &Type = value.get_value_type()?;
                let source_type: &Type = source.get_value_type()?;

                if !source_type.is_ptr_type() {
                    let lhs_type: &Type = source_type;
                    let rhs_type: &Type = value_type;

                    {
                        let control_context: &mut TypeCheckerControlContext =
                            self.get_mut_control_context();

                        checking::check_types(
                            lhs_type,
                            rhs_type,
                            Some(value),
                            None,
                            metadata,
                            source.get_span(),
                            control_context,
                        )?;

                        control_context.reset_checking_depth();
                    }
                }

                self.analyze_expr(source)?;
                self.analyze_expr(value)?;

                Ok(())
            }
            _ => self.analyze_expr(node),
        }
    }

    fn analyze_expr(&mut self, node: &'type_checker Ast) -> Result<(), CompilationIssue> {
        expressions::validate(self, node)
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    fn post_expression_evaluation(&mut self) {
        self.control_context.reset_checking_depth();
    }
}

impl TypeChecker<'_> {
    fn parse_top(&mut self) {
        {
            for node in self.ast.iter().rev() {
                match node {
                    Ast::AssemblerFunction {
                        name,
                        parameters_types: types,
                        attributes,
                        return_type,
                        ..
                    } => {
                        self.get_mut_table()
                            .new_asm_function(name, (return_type, types, attributes));
                    }
                    Ast::Function {
                        name,
                        parameter_types: types,
                        attributes,
                        return_type,
                        ..
                    } => {
                        self.get_mut_table()
                            .new_function(name, (return_type, types, attributes));
                    }
                    Ast::Intrinsic {
                        name,
                        parameters_types: types,
                        attributes,
                        return_type,
                        ..
                    } => {
                        self.get_mut_table()
                            .new_intrinsic(name, (return_type, types, attributes));
                    }

                    _ => (),
                }
            }
        }
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    #[inline]
    fn advance(&mut self) {
        if !self.is_eof() {
            self.position = self.position.saturating_add(1);
        }
    }

    #[inline]
    fn peek(&self) -> &'type_checker Ast<'type_checker> {
        &self.ast[self.position]
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.position >= self.ast.len()
    }
}

impl TypeChecker<'_> {
    #[inline]
    fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }

    #[inline]
    fn add_bug(&mut self, error: CompilationIssue) {
        self.bugs.push(error);
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    #[inline]
    fn get_table(&self) -> &TypeCheckerSymbolsTable<'type_checker> {
        &self.table
    }

    #[inline]
    fn get_type_context(&self) -> &TypeCheckerTypeContext<'type_checker> {
        &self.type_context
    }

    #[inline]
    fn get_control_context(&self) -> &TypeCheckerControlContext {
        &self.control_context
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    #[inline]
    fn get_mut_table(&mut self) -> &mut TypeCheckerSymbolsTable<'type_checker> {
        &mut self.table
    }

    #[inline]
    fn get_mut_type_context(&mut self) -> &mut TypeCheckerTypeContext<'type_checker> {
        &mut self.type_context
    }

    #[inline]
    fn get_mut_control_context(&mut self) -> &mut TypeCheckerControlContext {
        &mut self.control_context
    }
}

impl TypeChecker<'_> {
    #[inline]
    fn begin_scope(&mut self) {
        self.table.begin_scope();
    }

    #[inline]
    fn end_scope(&mut self) {
        self.table.end_scope();
    }
}
