use thrushc_ast::{
    Ast,
    traits::{AstCodeLocation, AstGetType, AstStandardExtensions},
};

use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;
use thrushc_typesystem::{
    Type,
    traits::{TypeCodeLocation, TypeIsExtensions, VoidTypeExtensions},
};

use crate::{
    context::TypeCheckerTypeContext, metadata::TypeCheckerExpressionMetadata,
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

    context: TypeCheckerTypeContext<'type_checker>,
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

            bugs: Vec::with_capacity(100),
            errors: Vec::with_capacity(100),
            warnings: Vec::with_capacity(100),

            context: TypeCheckerTypeContext::new(),
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

            if let Err(error) = self.analyze_decl(node) {
                self.add_error(error);
            }

            self.advance();
        }

        self.warnings.iter().for_each(|warn| {
            self.diagnostician
                .dispatch_diagnostic(warn, thrushc_logging::LoggingType::Warning);
        });

        if !self.errors.is_empty() || !self.bugs.is_empty() {
            self.bugs.iter().for_each(|warn| {
                self.diagnostician
                    .dispatch_diagnostic(warn, thrushc_logging::LoggingType::Bug);
            });

            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, thrushc_logging::LoggingType::Error);
            });

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

                        let metadata: TypeCheckerExpressionMetadata =
                            TypeCheckerExpressionMetadata::new(expr.is_literal_value());

                        checking::check_types(
                            target_type,
                            from_type,
                            Some(expr),
                            None,
                            metadata,
                            expr.get_span(),
                        )?;

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

                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(value.is_literal_value());

                let value_type: &Type = value.get_value_type()?;

                checking::check_types(
                    static_type,
                    value_type,
                    Some(value),
                    None,
                    metadata,
                    node.get_span(),
                )?;

                self.analyze_expr(value)?;

                Ok(())
            }
            Ast::Const {
                kind: target_type,
                value,
                ..
            } => {
                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(value.is_literal_value());

                let from_type: &Type = value.get_value_type()?;

                checking::check_types(
                    target_type,
                    &Type::Const(from_type.clone().into(), from_type.get_span()),
                    Some(value),
                    None,
                    metadata,
                    node.get_span(),
                )?;

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
            | Ast::Break { .. } => Ok(()),

            Ast::Enum { data, .. } => {
                {
                    for (_, target_type, expr) in data.iter() {
                        let from_type: &Type = expr.get_value_type()?;

                        let metadata: TypeCheckerExpressionMetadata =
                            TypeCheckerExpressionMetadata::new(expr.is_literal_value());

                        checking::check_types(
                            target_type,
                            from_type,
                            Some(expr),
                            None,
                            metadata,
                            expr.get_span(),
                        )?;

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

                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(value.is_literal_value());

                let value_type: &Type = value.get_value_type()?;

                checking::check_types(
                    static_type,
                    value_type,
                    Some(value),
                    None,
                    metadata,
                    node.get_span(),
                )?;

                self.analyze_expr(value)?;

                Ok(())
            }
            Ast::Const {
                kind: target_type,
                value,
                ..
            } => {
                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(value.is_literal_value());

                let from_type: &Type = value.get_value_type()?;

                checking::check_types(
                    target_type,
                    &Type::Const(from_type.clone().into(), from_type.get_span()),
                    Some(value),
                    None,
                    metadata,
                    node.get_span(),
                )?;

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

                let type_metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(local_value.is_literal_value());

                let local_value_type: &Type = local_value.get_value_type()?;
                let is_ref_ptr_like_type: bool =
                    local_value.is_reference() && local_value_type.is_ptr_like_type();

                if is_ref_ptr_like_type {
                    let local_value_type_fixed_ptr: Type = Type::Ptr(
                        Some(local_value_type.clone().into()),
                        local_value_type.get_span(),
                    );

                    checking::check_types(
                        local_type,
                        &local_value_type_fixed_ptr,
                        Some(local_value),
                        None,
                        type_metadata,
                        node.get_span(),
                    )?;
                } else {
                    checking::check_types(
                        local_type,
                        local_value_type,
                        Some(local_value),
                        None,
                        type_metadata,
                        node.get_span(),
                    )?;
                }

                self.analyze_expr(local_value)?;

                Ok(())
            }
            Ast::Block { nodes, .. } => {
                self.begin_scope();

                for node in nodes.iter() {
                    self.analyze_stmt(node)?;
                }

                self.end_scope();

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

                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(condition.is_literal_value());

                let span: Span = condition.get_span();

                checking::check_types(
                    &Type::Bool(span),
                    condition.get_value_type()?,
                    Some(condition),
                    None,
                    metadata,
                    span,
                )?;

                elseif.iter().try_for_each(|elif| self.analyze_stmt(elif))?;

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

                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(condition.is_literal_value());

                let span: Span = condition.get_span();

                checking::check_types(
                    &Type::Bool(condition.get_span()),
                    condition.get_value_type()?,
                    Some(condition),
                    None,
                    metadata,
                    span,
                )?;

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

                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(condition.is_literal_value());

                let span: Span = condition.get_span();

                checking::check_types(
                    &Type::Bool(span),
                    condition.get_value_type()?,
                    Some(condition),
                    None,
                    metadata,
                    span,
                )?;

                self.analyze_expr(condition)?;
                self.analyze_expr(actions)?;
                self.analyze_stmt(block)?;

                Ok(())
            }
            Ast::While {
                condition, block, ..
            } => {
                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(condition.is_literal_value());

                let span: Span = condition.get_span();

                checking::check_types(
                    &Type::Bool(span),
                    condition.get_value_type()?,
                    Some(condition),
                    None,
                    metadata,
                    span,
                )?;

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

                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(expr.is_literal_value());

                let Some((return_type, function_loc)) =
                    self.get_context().get_current_function_type()
                else {
                    return Err(CompilationIssue::Error(
                        CompilationIssueCode::E0020,
                        "Return statement outside of a function.".into(),
                        None,
                        expr.get_span(),
                    ));
                };

                checking::check_types(
                    return_type,
                    expr.get_value_type()?,
                    Some(expr),
                    None,
                    metadata,
                    function_loc,
                )?;

                self.analyze_expr(expr)?;

                Ok(())
            }
            Ast::Mut { source, value, .. } => {
                let metadata: TypeCheckerExpressionMetadata =
                    TypeCheckerExpressionMetadata::new(value.is_literal_value());

                let value_type: &Type = value.get_value_type()?;
                let source_type: &Type = source.get_value_type()?;

                if !source_type.is_ptr_type() {
                    let lhs_type: &Type = source_type;
                    let rhs_type: &Type = value_type;

                    checking::check_types(
                        lhs_type,
                        rhs_type,
                        Some(value),
                        None,
                        metadata,
                        source.get_span(),
                    )?;
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
            self.position += 1;
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
    fn get_context(&self) -> &TypeCheckerTypeContext<'type_checker> {
        &self.context
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    #[inline]
    fn get_mut_table(&mut self) -> &mut TypeCheckerSymbolsTable<'type_checker> {
        &mut self.table
    }

    #[inline]
    fn get_mut_context(&mut self) -> &mut TypeCheckerTypeContext<'type_checker> {
        &mut self.context
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
