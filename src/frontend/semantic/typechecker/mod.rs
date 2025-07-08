use symbols::TypeCheckerSymbolsTable;

use crate::{
    core::{
        compiler::options::CompilerFile, console::logging::LoggingType,
        diagnostic::diagnostician::Diagnostician, errors::standard::ThrushCompilerIssue,
    },
    frontend::types::{ast::Ast, parser::stmts::traits::ThrushAttributesExtensions},
};

mod bounds;
mod builtins;
mod call;
mod casts;
mod conditionals;
mod constant;
mod deref;
mod expressions;
mod functions;
mod lli;
mod local;
mod loops;
mod position;
mod staticvar;
mod symbols;
mod terminator;
mod validations;

#[derive(Debug)]
pub struct TypeChecker<'type_checker> {
    ast: &'type_checker [Ast<'type_checker>],
    position: usize,
    bugs: Vec<ThrushCompilerIssue>,
    errors: Vec<ThrushCompilerIssue>,
    warnings: Vec<ThrushCompilerIssue>,
    symbols: TypeCheckerSymbolsTable<'type_checker>,
    diagnostician: Diagnostician,
}

impl<'type_checker> TypeChecker<'type_checker> {
    pub fn new(
        ast: &'type_checker [Ast<'type_checker>],
        file: &'type_checker CompilerFile,
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

    pub fn check(&mut self) -> bool {
        self.init();

        while !self.is_eof() {
            let current_stmt: &Ast = self.peek();

            if let Err(error) = self.analyze_ast(current_stmt) {
                self.add_error(error);
            }

            self.advance();
        }

        self.warnings.iter().for_each(|warn| {
            self.diagnostician
                .build_diagnostic(warn, LoggingType::Warning);
        });

        if !self.errors.is_empty() || !self.bugs.is_empty() {
            self.bugs.iter().for_each(|warn| {
                self.diagnostician.build_diagnostic(warn, LoggingType::Bug);
            });

            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .build_diagnostic(error, LoggingType::Error);
            });

            return true;
        }

        false
    }

    pub fn analyze_ast(&mut self, stmt: &'type_checker Ast) -> Result<(), ThrushCompilerIssue> {
        /* ######################################################################


            TYPE CHECKER FUNCTIONS - START


        ########################################################################*/

        if let Ast::EntryPoint { .. } = stmt {
            return functions::validate_function(self, stmt);
        }

        if let Ast::AssemblerFunction { .. } = stmt {
            return functions::validate_function(self, stmt);
        }

        if let Ast::Function { .. } = stmt {
            return functions::validate_function(self, stmt);
        }

        /* ######################################################################


            TYPE CHECKER FUNCTIONS - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER DECLARATION - START


        ########################################################################*/

        if let Ast::GlobalAssembler { .. } = stmt {
            return Ok(());
        }

        if let Ast::Struct { .. } = stmt {
            return Ok(());
        }

        if let Ast::Enum { .. } = stmt {
            return Ok(());
        }

        if let Ast::Static { .. } = stmt {
            return staticvar::validate_static(self, stmt);
        }

        if let Ast::Const { .. } = stmt {
            return constant::validate_constant(self, stmt);
        }

        if let Ast::Local { .. } = stmt {
            return local::validate_local(self, stmt);
        }

        if let Ast::LLI { .. } = stmt {
            return lli::validate_lli(self, stmt);
        }

        if let Ast::Block { stmts, .. } = stmt {
            self.begin_scope();

            stmts.iter().try_for_each(|stmt| self.analyze_ast(stmt))?;

            self.end_scope();

            return Ok(());
        }

        /* ######################################################################


            TYPE CHECKER DECLARATION - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER CONTROL FLOW - END


        ########################################################################*/

        if let Ast::If { .. } | Ast::Elif { .. } | Ast::Else { .. } = stmt {
            conditionals::validate_conditional(self, stmt)?;

            return Ok(());
        }

        /* ######################################################################


            TYPE CHECKER CONTROL FLOW - START


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER LOOPS - START


        ########################################################################*/

        if let Ast::For { .. } = stmt {
            return loops::validate_loop(self, stmt);
        }

        if let Ast::While { .. } = stmt {
            return loops::validate_loop(self, stmt);
        }

        if let Ast::Loop { .. } = stmt {
            return loops::validate_loop(self, stmt);
        }

        /* ######################################################################


            TYPE CHECKER LOOPS - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER LOOP CONTROL FLOW - START


        ########################################################################*/

        if let Ast::Continue { .. } | Ast::Break { .. } = stmt {
            return Ok(());
        }

        /* ######################################################################


            TYPE CHECKER LOOP CONTROL FLOW - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER TERMINATOR - START


        ########################################################################*/

        if let Ast::Return { .. } = stmt {
            return terminator::validate_terminator(self, stmt);
        }

        /* ######################################################################


            TYPE CHECKER TERMINATOR - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER DEREFERENCE - START


        ########################################################################*/

        if let Ast::Deref { .. } = stmt {
            return deref::validate_dereference(self, stmt);
        }

        /* ######################################################################


            TYPE CHECKER DEREFERENCE - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER CASTS - START


        ########################################################################*/

        if let Ast::As { .. } = stmt {
            return casts::validate_cast_as(self, stmt);
        }

        /* ######################################################################


            TYPE CHECKER CASTS - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER LLI - START


        ########################################################################*/

        if let Ast::Write { .. } = stmt {
            return lli::validate_lli(self, stmt);
        }

        if let Ast::Address { .. } = stmt {
            return lli::validate_lli(self, stmt);
        }

        if let Ast::Load { .. } = stmt {
            return lli::validate_lli(self, stmt);
        }

        /* ######################################################################


            TYPE CHECKER LLI - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER BUILTINS - START


        ########################################################################*/
        if let Ast::Builtin { builtin, span, .. } = stmt {
            return builtins::validate_builtin(self, builtin, *span);
        }

        if let Ast::SizeOf { sizeof, span, .. } = stmt {
            if sizeof.is_void_type() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "The void type isn't a value.".into(),
                    None,
                    *span,
                ));
            }

            return Ok(());
        }

        /* ######################################################################


            TYPE CHECKER BUILTINS - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER EXPRESSIONS - START


        ########################################################################*/

        expressions::validate_expression(self, stmt)

        /* ######################################################################


            TYPE CHECKER EXPRESSIONS - END


        ########################################################################*/
    }

    pub fn init(&mut self) {
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
                        .new_asm_function(name, (types, attributes.has_public_attribute()));
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
    }

    pub fn advance(&mut self) {
        if !self.is_eof() {
            self.position += 1;
        }
    }

    pub fn peek(&self) -> &'type_checker Ast<'type_checker> {
        &self.ast[self.position]
    }

    pub fn is_eof(&self) -> bool {
        self.position >= self.ast.len()
    }
}

impl TypeChecker<'_> {
    pub fn add_warning(&mut self, warning: ThrushCompilerIssue) {
        self.warnings.push(warning);
    }

    pub fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }

    pub fn add_bug(&mut self, error: ThrushCompilerIssue) {
        self.bugs.push(error);
    }
}

impl TypeChecker<'_> {
    pub fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    pub fn end_scope(&mut self) {
        self.symbols.end_scope();
    }
}
