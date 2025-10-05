use symbols::AnalyzerSymbolsTable;

use crate::{
    core::{
        compiler::options::CompilationUnit, console::logging::LoggingType,
        diagnostic::diagnostician::Diagnostician, errors::standard::ThrushCompilerIssue,
    },
    frontends::classical::types::{ast::Ast, parser::stmts::traits::ThrushAttributesExtensions},
};

mod builtins;
mod constants;
mod declarations;
mod expressions;
mod statements;
mod symbols;

#[derive(Debug)]
pub struct Analyzer<'analyzer> {
    ast: &'analyzer [Ast<'analyzer>],
    position: usize,

    bugs: Vec<ThrushCompilerIssue>,
    errors: Vec<ThrushCompilerIssue>,
    warnings: Vec<ThrushCompilerIssue>,

    symbols: AnalyzerSymbolsTable<'analyzer>,
    diagnostician: Diagnostician,
}

impl<'analyzer> Analyzer<'analyzer> {
    pub fn new(ast: &'analyzer [Ast<'analyzer>], file: &'analyzer CompilationUnit) -> Self {
        Self {
            ast,
            position: 0,

            bugs: Vec::with_capacity(100),
            errors: Vec::with_capacity(100),
            warnings: Vec::with_capacity(100),

            symbols: AnalyzerSymbolsTable::new(),
            diagnostician: Diagnostician::new(file),
        }
    }
}

impl<'analyzer> Analyzer<'analyzer> {
    pub fn check(&mut self) -> bool {
        self.declare_forward();

        while !self.is_eof() {
            let node: &Ast = self.peek();

            if let Err(error) = self.analyze_decl(node) {
                self.add_error(error);
            }

            self.advance();
        }

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

    pub fn analyze_decl(&mut self, node: &'analyzer Ast) -> Result<(), ThrushCompilerIssue> {
        /* ######################################################################


            TYPE CHECKER DECLARATIONS - START


        ########################################################################*/

        if let Ast::EntryPoint { .. } = node {
            return declarations::functions::validate(self, node);
        }

        if let Ast::AssemblerFunction { .. } = node {
            return declarations::functions::validate(self, node);
        }

        if let Ast::Function { .. } = node {
            return declarations::functions::validate(self, node);
        }

        if let Ast::Enum { .. } | Ast::Struct { .. } | Ast::GlobalAssembler { .. } = node {
            return Ok(());
        }

        if let Ast::Static { .. } = node {
            return statements::staticvar::validate(self, node);
        }

        if let Ast::Const { .. } = node {
            return statements::constant::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER DECLARATIONS - END


        ########################################################################*/

        Ok(())
    }

    pub fn analyze_stmt(&mut self, node: &'analyzer Ast) -> Result<(), ThrushCompilerIssue> {
        /* ######################################################################


            TYPE CHECKER STATEMENTS - START


        ########################################################################*/

        if let Ast::Static { .. } = node {
            return statements::staticvar::validate(self, node);
        }

        if let Ast::Const { .. } = node {
            return statements::constant::validate(self, node);
        }

        if let Ast::Local { .. } = node {
            return statements::local::validate(self, node);
        }

        if let Ast::LLI { .. } = node {
            return statements::lli::validate(self, node);
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


            TYPE CHECKER DEREFERENCE - START


        ########################################################################*/

        if let Ast::Defer { .. } = node {
            return expressions::deref::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER DEREFERENCE - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER CASTS - START


        ########################################################################*/

        if let Ast::As { .. } = node {
            return expressions::cast::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER CASTS - END


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

        /* ######################################################################


            TYPE CHECKER LLI - START


        ########################################################################*/

        if let Ast::Write { .. } = node {
            return statements::lli::validate(self, node);
        }

        if let Ast::Address { .. } = node {
            return statements::lli::validate(self, node);
        }

        if let Ast::Load { .. } = node {
            return statements::lli::validate(self, node);
        }

        /* ######################################################################


            TYPE CHECKER LLI - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER BUILTINS - START


        ########################################################################*/
        if let Ast::Builtin { builtin, .. } = node {
            return builtins::validate_builtin(self, builtin);
        }

        /* ######################################################################


            TYPE CHECKER BUILTINS - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER EXPRESSIONS - START


        ########################################################################*/

        expressions::validate(self, node)

        /* ######################################################################


            TYPE CHECKER EXPRESSIONS - END


        ########################################################################*/
    }

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
}

impl<'analyzer> Analyzer<'analyzer> {
    #[inline]
    pub fn advance(&mut self) {
        if !self.is_eof() {
            self.position += 1;
        }
    }

    #[inline]
    pub fn peek(&self) -> &'analyzer Ast<'analyzer> {
        &self.ast[self.position]
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.position >= self.ast.len()
    }
}

impl Analyzer<'_> {
    #[inline]
    pub fn add_warning(&mut self, warning: ThrushCompilerIssue) {
        self.warnings.push(warning);
    }

    #[inline]
    pub fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }

    #[inline]
    pub fn add_bug(&mut self, error: ThrushCompilerIssue) {
        self.bugs.push(error);
    }
}

impl Analyzer<'_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.symbols.end_scope();
    }
}
