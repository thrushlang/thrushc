use crate::core::compiler::options::CompilationUnit;
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::analyzer::context::AnalyzerContext;
use crate::front_end::semantic::analyzer::symbols::AnalyzerSymbolsTable;
use crate::front_end::types::ast::Ast;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

pub mod builtins;
pub mod checks;
pub mod constants;
pub mod context;
pub mod declarations;
pub mod expressions;
pub mod statements;
pub mod symbols;

#[derive(Debug)]
pub struct Analyzer<'analyzer> {
    ast: &'analyzer [Ast<'analyzer>],
    position: usize,

    bugs: Vec<CompilationIssue>,
    errors: Vec<CompilationIssue>,
    warnings: Vec<CompilationIssue>,

    symbols: AnalyzerSymbolsTable<'analyzer>,
    diagnostician: Diagnostician,

    context: AnalyzerContext,
}

impl<'analyzer> Analyzer<'analyzer> {
    #[inline]
    pub fn new(ast: &'analyzer [Ast<'analyzer>], file: &'analyzer CompilationUnit) -> Self {
        Self {
            ast,
            position: 0,

            bugs: Vec::with_capacity(100),
            errors: Vec::with_capacity(100),
            warnings: Vec::with_capacity(100),

            symbols: AnalyzerSymbolsTable::new(),
            diagnostician: Diagnostician::new(file),

            context: AnalyzerContext::new(),
        }
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    pub fn start(&mut self) -> bool {
        self.declare_forward();

        while !self.is_eof() {
            let node: &Ast = self.peek();

            if let Err(error) = self.analyze_decl(node) {
                self.add_error(error);
            }

            self.advance();
        }

        self.check()
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    fn check(&mut self) -> bool {
        self.warnings.iter().for_each(|warn| {
            self.diagnostician
                .dispatch_diagnostic(warn, LoggingType::Warning);
        });

        if !self.errors.is_empty() || !self.bugs.is_empty() {
            self.bugs.iter().for_each(|warn| {
                self.diagnostician
                    .dispatch_diagnostic(warn, LoggingType::Bug);
            });

            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            return true;
        }

        false
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    fn analyze_decl(&mut self, node: &'analyzer Ast) -> Result<(), CompilationIssue> {
        match node {
            Ast::Function { .. } => declarations::functions::validate(self, node),
            Ast::Struct { .. } => Ok(()),
            Ast::GlobalAssembler { .. } => declarations::glasm::validate(self, node),
            Ast::CustomType { .. } => Ok(()),
            Ast::Enum { .. } => declarations::glenum::validate(self, node),
            Ast::Static { .. } => declarations::glstatic::validate(self, node),
            Ast::Const { .. } => declarations::glconstant::validate(self, node),

            _ => Ok(()),
        }
    }

    fn analyze_stmt(&mut self, node: &'analyzer Ast) -> Result<(), CompilationIssue> {
        match node {
            Ast::Enum { .. } => statements::lenum::validate(self, node),
            Ast::Static { .. } => statements::staticvar::validate(self, node),
            Ast::Const { .. } => statements::constant::validate(self, node),
            Ast::Local { .. } => statements::local::validate(self, node),
            Ast::If { .. } | Ast::Elif { .. } | Ast::Else { .. } => {
                statements::conditional::validate(self, node)
            }
            Ast::For { .. } | Ast::While { .. } | Ast::Loop { .. } => {
                statements::loops::validate(self, node)
            }
            Ast::Continue { .. } | Ast::Break { .. } => {
                if !self.get_context().is_inside_loop() {
                    self.add_error(
                        CompilationIssue::Error(
                            "Syntax Error".into(),
                            "Only loop control flow terminators can be inside a loop. The instruction inside a loop was expected.".into(),
                            None,
                            node.get_span(),
                        )
                    );
                }

                Ok(())
            }
            Ast::Mut { .. } => statements::mutation::validate(self, node),
            Ast::Block { nodes, .. } => {
                self.begin_scope();

                checks::check_for_multiple_terminators(self, node);
                checks::check_for_unreachable_code_instructions(self, node);

                nodes.iter().try_for_each(|node| self.analyze_stmt(node))?;

                self.end_scope();

                Ok(())
            }

            Ast::Return { .. } => statements::terminator::validate(self, node),

            node => self.analyze_expr(node),
        }
    }

    fn analyze_expr(&mut self, node: &'analyzer Ast) -> Result<(), CompilationIssue> {
        expressions::validate(self, node)
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    fn declare_forward(&mut self) {
        for stmt in self.ast.iter() {
            match stmt {
                Ast::AssemblerFunction {
                    name,
                    parameters_types: types,
                    attributes,
                    ..
                } => {
                    self.symbols
                        .new_asm_function(name, (types, attributes.has_public_attribute()));
                }

                Ast::Function {
                    name,
                    parameter_types: types,
                    attributes,
                    ..
                } => {
                    self.symbols
                        .new_function(name, (types, attributes.has_ignore_attribute()));
                }

                _ => (),
            }
        }
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    #[inline]
    fn advance(&mut self) {
        if !self.is_eof() {
            self.position += 1;
        }
    }

    #[inline]
    fn peek(&self) -> &'analyzer Ast<'analyzer> {
        &self.ast[self.position]
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.position >= self.ast.len()
    }
}

impl Analyzer<'_> {
    #[inline]
    fn add_warning(&mut self, warning: CompilationIssue) {
        self.warnings.push(warning);
    }

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
    fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    #[inline]
    fn end_scope(&mut self) {
        self.symbols.end_scope();
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
