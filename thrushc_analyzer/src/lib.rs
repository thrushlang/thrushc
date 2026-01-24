use thrushc_ast::{
    Ast,
    traits::{
        AstCodeLocation, AstConstantExtensions, AstGetType, AstMemoryExtensions,
        AstStandardExtensions,
    },
};
use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_options::{CompilationUnit, CompilerOptions};

use thrushc_span::Span;
use thrushc_typesystem::{Type, traits::TypeExtensions};

use crate::context::AnalyzerContext;

mod context;
mod expressions;

#[derive(Debug)]
pub struct Analyzer<'analyzer> {
    ast: &'analyzer [Ast<'analyzer>],

    bugs: Vec<CompilationIssue>,
    errors: Vec<CompilationIssue>,
    warnings: Vec<CompilationIssue>,

    diagnostician: Diagnostician,

    context: AnalyzerContext,
}

impl<'analyzer> Analyzer<'analyzer> {
    #[inline]
    pub fn new(
        ast: &'analyzer [Ast<'analyzer>],
        file: &'analyzer CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        Self {
            ast,

            bugs: Vec::with_capacity(100),
            errors: Vec::with_capacity(100),
            warnings: Vec::with_capacity(100),

            diagnostician: Diagnostician::new(file, options),

            context: AnalyzerContext::new(),
        }
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    pub fn start(&mut self) -> bool {
        for node in self.ast.iter() {
            if let Err(error) = self.analyze_decl(node) {
                self.add_error(error);
            }
        }

        self.check()
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    fn check(&mut self) -> bool {
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

            return true;
        }

        false
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    fn analyze_decl(&mut self, node: &'analyzer Ast) -> Result<(), CompilationIssue> {
        match node {
            Ast::AssemblerFunction {
                parameters, span, ..
            } => {
                if parameters.len() > 12 {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0036,
                        "Too many parameters for the assembler function. Package them in structures or use them through pointers.".into(),
                        None,
                        *span,
                    ));
                }

                Ok(())
            }

            Ast::Function {
                parameters,
                body,
                span,
                ..
            } => {
                if parameters.len() > 12 {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0036,
                        "Too many parameters for the function. Package them in structures or use them through pointers.".into(),
                        None,
                        *span,
                    ));
                }

                if let Some(body) = body {
                    self.analyze_stmt(body)?;
                }

                Ok(())
            }
            Ast::Struct { .. } => Ok(()),
            Ast::GlobalAssembler { span, .. } => {
                if self.get_context().has_global_assembler() {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0005,
                        "Global assembler is already defined before. One per file is expected. Remove one.".into(),
                        None,
                        *span,
                    ));
                }

                self.get_mut_context().set_has_global_assembler();

                Ok(())
            }
            Ast::CustomType { .. } => Ok(()),
            Ast::Enum { data, .. } => {
                {
                    for (_, _, expr) in data.iter() {
                        let span: Span = expr.get_span();

                        if !expr.is_constant_value() {
                            self.add_error(CompilationIssue::Error(
                                CompilationIssueCode::E0006,
                                "Expected a valid constant value or reference to a constant value."
                                    .into(),
                                None,
                                span,
                            ));
                        }

                        self.analyze_expr(expr)?;
                    }
                }

                Ok(())
            }
            Ast::Static { value, .. } => {
                if let Some(value) = value {
                    let span: Span = value.get_span();

                    if !value.is_constant_value() {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0006,
                            "Expected a valid constant value or reference to a constant value."
                                .into(),
                            None,
                            span,
                        ));
                    }

                    self.analyze_expr(value)?;
                }

                Ok(())
            }
            Ast::Const { value, .. } => {
                let span: Span = value.get_span();

                if !value.is_constant_value() {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0006,
                        "Expected a valid constant value or reference to a constant value.".into(),
                        None,
                        span,
                    ));
                }

                self.analyze_expr(value)?;

                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn analyze_stmt(&mut self, node: &'analyzer Ast) -> Result<(), CompilationIssue> {
        match node {
            Ast::Enum { data, .. } => {
                {
                    for (_, _, expr) in data.iter() {
                        let span: Span = expr.get_span();

                        if !expr.is_constant_value() {
                            self.add_error(CompilationIssue::Error(
                                CompilationIssueCode::E0006,
                                "Expected a valid constant value or reference to a constant value."
                                    .into(),
                                None,
                                span,
                            ));
                        }

                        self.analyze_expr(expr)?;
                    }
                }

                Ok(())
            }
            Ast::Static { value, .. } => {
                if let Some(value) = value {
                    if !value.is_constant_value() {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0006,
                            "Expected a valid constant value or reference to a constant value."
                                .into(),
                            None,
                            value.get_span(),
                        ));
                    }

                    self.analyze_expr(value)?;
                }

                Ok(())
            }
            Ast::Const { value, .. } => {
                let span: Span = value.get_span();

                if !value.is_constant_value() {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0006,
                        "Expected a valid constant value or reference to a constant value.".into(),
                        None,
                        span,
                    ));
                }

                self.analyze_expr(value)?;

                Ok(())
            }
            Ast::Local {
                value, metadata, ..
            } => {
                if let Some(value) = value {
                    if !metadata.is_undefined() {
                        self.analyze_expr(value)?;
                    }
                }

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
                self.analyze_expr(condition)?;

                self.analyze_expr(actions)?;
                self.analyze_stmt(block)?;

                Ok(())
            }

            Ast::While {
                condition, block, ..
            } => {
                self.analyze_expr(condition)?;
                self.analyze_stmt(block)?;

                Ok(())
            }

            Ast::Loop { block, .. } => {
                self.analyze_stmt(block)?;

                Ok(())
            }
            Ast::Continue { .. }
            | Ast::ContinueAll { .. }
            | Ast::Break { .. }
            | Ast::BreakAll { .. } => Ok(()),
            Ast::Mut { source, value, .. } => {
                let source_type: &Type = source.get_value_type()?;

                if source.is_reference() && !source.is_allocated() {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0007,
                        "An reference with memory address was expected. Try to allocate it.".into(),
                        None,
                        source.get_span(),
                    ));
                }

                if (!source.is_allocated_value()? || !source.is_reference())
                    && source_type.is_value()
                {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0008,
                        format!(
                            "An value with memory address was expected, got '{}'. Try to allocate it.",
                            source_type
                        ),
                        None,
                        source.get_span(),
                    ));
                }

                self.analyze_expr(source)?;
                self.analyze_expr(value)?;

                Ok(())
            }
            Ast::Block { nodes, .. } => {
                nodes.iter().try_for_each(|node| self.analyze_stmt(node))?;

                Ok(())
            }

            Ast::Return { expression, .. } => {
                if let Some(expr) = expression {
                    self.analyze_expr(expr)?;
                }

                Ok(())
            }

            node => self.analyze_expr(node),
        }
    }

    fn analyze_expr(&mut self, node: &'analyzer Ast) -> Result<(), CompilationIssue> {
        expressions::validate(self, node)
    }
}

impl Analyzer<'_> {
    #[inline]
    fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }

    #[inline]
    fn add_bug(&mut self, error: CompilationIssue) {
        self.bugs.push(error);
    }
}

impl Analyzer<'_> {
    #[inline]
    fn get_context(&self) -> &AnalyzerContext {
        &self.context
    }

    #[inline]
    fn get_mut_context(&mut self) -> &mut AnalyzerContext {
        &mut self.context
    }
}
