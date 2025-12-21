use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::typechecker::symbols::TypeCheckerSymbolsTable;
use crate::front_end::types::ast::Ast;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

pub mod builtins;
pub mod checks;
pub mod constants;
pub mod declarations;
pub mod expressions;
pub mod metadata;
pub mod statements;
pub mod symbols;
pub mod validations;

#[derive(Debug)]
pub struct TypeChecker<'type_checker> {
    ast: &'type_checker [Ast<'type_checker>],
    position: usize,

    bugs: Vec<CompilationIssue>,
    errors: Vec<CompilationIssue>,
    warnings: Vec<CompilationIssue>,

    symbols: TypeCheckerSymbolsTable<'type_checker>,
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

            symbols: TypeCheckerSymbolsTable::new(),
            diagnostician: Diagnostician::new(file, options),
        }
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
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

impl<'type_checker> TypeChecker<'type_checker> {
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

impl<'type_checker> TypeChecker<'type_checker> {
    pub fn analyze_decl(&mut self, node: &'type_checker Ast) -> Result<(), CompilationIssue> {
        match node {
            Ast::Intrinsic { .. } | Ast::AssemblerFunction { .. } | Ast::Function { .. } => {
                declarations::functions::validate(self, node)
            }
            Ast::CustomType { .. } | Ast::GlobalAssembler { .. } | Ast::Struct { .. } => Ok(()),
            Ast::Enum { .. } => declarations::glenum::validate(self, node),
            Ast::Static { .. } => declarations::glstatic::validate(self, node),
            Ast::Const { .. } => declarations::glconstant::validate(self, node),

            _ => Ok(()),
        }
    }

    fn analyze_stmt(&mut self, node: &'type_checker Ast) -> Result<(), CompilationIssue> {
        match node {
            Ast::CustomType { .. }
            | Ast::Struct { .. }
            | Ast::Continue { .. }
            | Ast::Break { .. } => Ok(()),
            Ast::Enum { .. } => statements::lenum::validate(self, node),
            Ast::Static { .. } => statements::staticvar::validate(self, node),
            Ast::Const { .. } => statements::constant::validate(self, node),
            Ast::Local { .. } => statements::local::validate(self, node),
            Ast::Block { nodes, .. } => {
                self.begin_scope();
                nodes.iter().try_for_each(|node| self.analyze_stmt(node))?;
                self.end_scope();
                Ok(())
            }
            Ast::If { .. } | Ast::Elif { .. } | Ast::Else { .. } => {
                statements::conditional::validate(self, node)?;
                Ok(())
            }
            Ast::For { .. } | Ast::While { .. } | Ast::Loop { .. } => {
                statements::loops::validate(self, node)
            }
            Ast::Return { .. } => statements::terminator::validate(self, node),
            Ast::Mut { .. } => statements::mutation::validate(self, node),
            _ => self.analyze_expr(node),
        }
    }

    fn analyze_expr(&mut self, node: &'type_checker Ast) -> Result<(), CompilationIssue> {
        expressions::validate(self, node)
    }
}

impl TypeChecker<'_> {
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
                        .new_asm_function(name, (types, attributes.has_ignore_attribute()));
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
                Ast::Intrinsic {
                    name,
                    parameters_types: types,
                    attributes,
                    ..
                } => {
                    self.symbols
                        .new_intrinsic(name, (types, attributes.has_ignore_attribute()));
                }

                _ => (),
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
    fn get_symbols(&self) -> &TypeCheckerSymbolsTable<'type_checker> {
        &self.symbols
    }
}

impl TypeChecker<'_> {
    #[inline]
    fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    #[inline]
    fn end_scope(&mut self) {
        self.symbols.end_scope();
    }
}
