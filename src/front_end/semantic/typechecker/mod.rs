pub mod builtins;
pub mod checks;
pub mod constants;
pub mod declarations;
pub mod expressions;
pub mod metadata;
pub mod statements;
pub mod symbols;
pub mod validations;

use crate::core::compiler::options::CompilationUnit;
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::typechecker::symbols::TypeCheckerSymbolsTable;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::types::attributes::traits::ThrushAttributesExtensions;

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
    ) -> Self {
        Self {
            ast,
            position: 0,

            bugs: Vec::with_capacity(100),
            errors: Vec::with_capacity(100),
            warnings: Vec::with_capacity(100),

            symbols: TypeCheckerSymbolsTable::new(),
            diagnostician: Diagnostician::new(file),
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
        /* ######################################################################


            TYPE CHECKER DECLARATIONS - START


        ########################################################################*/

        if let Ast::Intrinsic { .. } = node {
            return declarations::functions::validate(self, node);
        }

        if let Ast::AssemblerFunction { .. } = node {
            return declarations::functions::validate(self, node);
        }

        if let Ast::Function { .. } = node {
            return declarations::functions::validate(self, node);
        }

        if let Ast::CustomType { .. } = node {
            return Ok(());
        }

        if let Ast::GlobalAssembler { .. } = node {
            return Ok(());
        }

        if let Ast::Struct { .. } = node {
            return Ok(());
        }

        if let Ast::Enum { .. } = node {
            return declarations::glenum::validate(self, node);
        }

        if let Ast::Static { .. } = node {
            return declarations::glstatic::validate(self, node);
        }

        if let Ast::Const { .. } = node {
            return declarations::glconstant::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER DECLARATIONS - END


        ########################################################################*/

        Ok(())
    }

    pub fn analyze_stmt(&mut self, node: &'type_checker Ast) -> Result<(), CompilationIssue> {
        /* ######################################################################


            TYPE CHECKER STATEMENTS - START


        ########################################################################*/

        if let Ast::CustomType { .. } = node {
            return Ok(());
        }

        if let Ast::Struct { .. } = node {
            return Ok(());
        }

        if let Ast::Enum { .. } = node {
            return statements::lenum::validate(self, node);
        }

        if let Ast::Static { .. } = node {
            return statements::staticvar::validate(self, node);
        }

        if let Ast::Const { .. } = node {
            return statements::constant::validate(self, node);
        }

        if let Ast::Local { .. } = node {
            return statements::local::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER STATEMENTS - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER CODE BLOCK - START


        ########################################################################*/

        if let Ast::Block { stmts, .. } = node {
            self.begin_scope();

            stmts.iter().try_for_each(|stmt| self.analyze_stmt(stmt))?;

            self.end_scope();

            return Ok(());
        }

        /* ######################################################################


            TYPE CHECKER CODE BLOCK - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER CONTROL FLOW - END


        ########################################################################*/

        if let Ast::If { .. } | Ast::Elif { .. } | Ast::Else { .. } = node {
            statements::conditional::validate(self, node)?;

            return Ok(());
        }

        /* ######################################################################


            TYPE CHECKER CONTROL FLOW - START


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER LOOPS - START


        ########################################################################*/

        if let Ast::For { .. } = node {
            return statements::loops::validate(self, node);
        }

        if let Ast::While { .. } = node {
            return statements::loops::validate(self, node);
        }

        if let Ast::Loop { .. } = node {
            return statements::loops::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER LOOPS - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER LOOP CONTROL FLOW - START


        ########################################################################*/

        if let Ast::Continue { .. } | Ast::Break { .. } = node {
            return Ok(());
        }

        /* ######################################################################


            TYPE CHECKER LOOP CONTROL FLOW - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER TERMINATOR - START


        ########################################################################*/

        if let Ast::Return { .. } = node {
            return statements::terminator::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER TERMINATOR - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER MUTATION - START


        ########################################################################*/

        if let Ast::Mut { .. } = node {
            return statements::mutation::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER MUTATION - END


        ########################################################################*/

        self.analyze_expr(node)
    }

    pub fn analyze_expr(&mut self, node: &'type_checker Ast) -> Result<(), CompilationIssue> {
        expressions::validate(self, node)
    }
}

impl TypeChecker<'_> {
    pub fn declare_forward(&mut self) {
        self.ast
            .iter()
            .filter(|stmt| stmt.is_asm_function())
            .for_each(|stmt| {
                if let Ast::AssemblerFunction {
                    name,
                    parameters_types: types,
                    attributes,
                    ..
                } = stmt
                {
                    self.symbols
                        .new_asm_function(name, (types, attributes.has_ignore_attribute()));
                }
            });

        self.ast
            .iter()
            .filter(|stmt| stmt.is_function())
            .for_each(|stmt| {
                if let Ast::Function {
                    name,
                    parameter_types: types,
                    attributes,
                    ..
                } = stmt
                {
                    self.symbols
                        .new_function(name, (types, attributes.has_ignore_attribute()));
                }
            });

        self.ast
            .iter()
            .filter(|stmt| stmt.is_intrinsic())
            .for_each(|stmt| {
                if let Ast::Intrinsic {
                    name,
                    parameters_types: types,
                    attributes,
                    ..
                } = stmt
                {
                    self.symbols
                        .new_intrinsic(name, (types, attributes.has_ignore_attribute()));
                }
            });
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    #[inline]
    pub fn advance(&mut self) {
        if !self.is_eof() {
            self.position += 1;
        }
    }

    #[inline]
    pub fn peek(&self) -> &'type_checker Ast<'type_checker> {
        &self.ast[self.position]
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.position >= self.ast.len()
    }
}

impl TypeChecker<'_> {
    #[inline]
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }

    #[inline]
    pub fn add_bug(&mut self, error: CompilationIssue) {
        self.bugs.push(error);
    }
}

impl<'type_checker> TypeChecker<'type_checker> {
    #[inline]
    pub fn get_symbols(&self) -> &TypeCheckerSymbolsTable<'type_checker> {
        &self.symbols
    }
}

impl TypeChecker<'_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.symbols.end_scope();
    }
}
