use ahash::AHashMap as HashMap;

use symbols::LinterSymbolsTable;

use crate::{
    core::{
        compiler::options::CompilerFile,
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
        errors::standard::ThrushCompilerIssue,
    },
    frontends::classical::{
        lexer::span::Span,
        semantic::linter::expressions::deref,
        types::{ast::Ast, parser::stmts::traits::ThrushAttributesExtensions},
    },
};

pub mod attributes;

mod builtins;
mod declarations;
mod expressions;
mod marks;
mod statements;
mod symbols;

#[derive(Debug)]
pub struct Linter<'linter> {
    ast: &'linter [Ast<'linter>],
    current: usize,
    warnings: Vec<ThrushCompilerIssue>,
    bugs: Vec<ThrushCompilerIssue>,
    diagnostician: Diagnostician,
    symbols: LinterSymbolsTable<'linter>,
}

impl<'linter> Linter<'linter> {
    pub fn new(ast: &'linter [Ast], file: &'linter CompilerFile) -> Self {
        Self {
            ast,
            current: 0,
            warnings: Vec::with_capacity(100),
            bugs: Vec::with_capacity(100),
            diagnostician: Diagnostician::new(file),
            symbols: LinterSymbolsTable::new(),
        }
    }

    pub fn check(&mut self) {
        self.forward_all();

        while !self.is_eof() {
            let current_node: &Ast = self.peek();

            self.analyze_decl(current_node);

            self.advance();
        }

        self.generate_warnings();

        self.bugs.iter().for_each(|bug: &ThrushCompilerIssue| {
            self.diagnostician.build_diagnostic(bug, LoggingType::Bug);
        });

        self.warnings.iter().for_each(|warn: &ThrushCompilerIssue| {
            self.diagnostician
                .build_diagnostic(warn, LoggingType::Warning);
        });
    }

    pub fn analyze_decl(&mut self, node: &'linter Ast) {
        /* ######################################################################


            LINTER DECLARATIONS | START


        ########################################################################*/

        if let Ast::EntryPoint { .. } | Ast::Function { .. } = node {
            return declarations::functions::analyze(self, node);
        }

        if let Ast::Enum { .. } = node {
            return declarations::union::analyze(self, node);
        }

        if let Ast::GlobalAssembler { .. } = node {}

        /* ######################################################################


            LINTER DECLARATIONS | END


        ########################################################################*/
    }

    pub fn analyze_stmt(&mut self, node: &'linter Ast) {
        /* ######################################################################


            LINTER STATEMENTS | START


        ########################################################################*/

        if let Ast::Static { .. } = node {
            return statements::staticvar::analyze(self, node);
        }

        if let Ast::Const { .. } = node {
            return statements::constant::analyze(self, node);
        }

        if let Ast::Block { stmts, .. } = node {
            self.begin_scope();

            stmts.iter().for_each(|node| {
                self.analyze_stmt(node);
            });

            self.generate_scoped_warnings();

            self.end_scope();

            return;
        }

        /* ######################################################################


            LINTER VARIABLES | START


        ########################################################################*/

        if let Ast::LLI { .. } = node {
            return statements::lli::analyze(self, node);
        }

        if let Ast::Local { .. } = node {
            return statements::local::analyze(self, node);
        }

        /* ######################################################################


            LINTER VARIABLES | END


        ########################################################################*/

        /* ######################################################################


            LINTER TERMINATOR | START


        ########################################################################*/

        if let Ast::Return { .. } = node {
            return statements::terminator::analyze(self, node);
        }

        /* ######################################################################


            LINTER TERMINATOR | END


        ########################################################################*/

        /* ######################################################################


            LINTER CONDITIONALS | START


        ########################################################################*/

        if let Ast::If { .. } = node {
            return statements::conditional::analyze(self, node);
        }

        if let Ast::Elif { .. } = node {
            return statements::conditional::analyze(self, node);
        }

        if let Ast::Else { .. } = node {
            return statements::conditional::analyze(self, node);
        }

        /* ######################################################################


            LINTER CONDITIONALS | END


        ########################################################################*/

        /* ######################################################################


            LINTER LOOPS | START


        ########################################################################*/

        if let Ast::For { .. } = node {
            return statements::loops::analyze(self, node);
        }

        if let Ast::While { .. } = node {
            return statements::loops::analyze(self, node);
        }

        if let Ast::Loop { .. } = node {
            return statements::loops::analyze(self, node);
        }

        /* ######################################################################


            LINTER LOOPS | END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER LOOP CONTROL FLOW - START


        ########################################################################*/

        if let Ast::Continue { .. } | Ast::Break { .. } = node {
            return;
        }

        /* ######################################################################


            TYPE CHECKER LOOP CONTROL FLOW - END


        ########################################################################*/

        /* ######################################################################


            LINTER DEREFERENCE | START


        ########################################################################*/

        if let Ast::Deref { .. } = node {
            return deref::analyze_dereference(self, node);
        }

        /* ######################################################################


            LINTER DEREFERENCE | END


        ########################################################################*/

        /* ######################################################################


            LINTER LLI | START


        ########################################################################*/

        if let Ast::Write { .. } = node {
            return statements::lli::analyze(self, node);
        }

        if let Ast::Address { .. } = node {
            return statements::lli::analyze(self, node);
        }

        if let Ast::Load { .. } = node {
            return statements::lli::analyze(self, node);
        }

        /* ######################################################################


            LINTER LLI | END


        ########################################################################*/

        /* ######################################################################


            LINTER CASTS | START


        ########################################################################*/

        if let Ast::As { .. } = node {
            return expressions::cast::analyze_cast(self, node);
        }

        /* ######################################################################


            LINTER CASTS | END


        ########################################################################*/

        /* ######################################################################


            LINTER BUILTINS | START


        ########################################################################*/

        if let Ast::Builtin { builtin, .. } = node {
            return builtins::analyze_builtin(self, builtin);
        }

        /* ######################################################################


            LINTER BUILTINS | END


        ########################################################################*/

        /* ######################################################################


            LINTER STATEMENTS | END


        ########################################################################*/

        /* ######################################################################


            LINTER EXPRESSIONS | START


        ########################################################################*/

        self.analyze_expr(node);
    }

    pub fn analyze_expr(&mut self, expr: &'linter Ast) {
        expressions::analyze(self, expr);
    }

    pub fn generate_scoped_warnings(&mut self) {
        if let Some(last_scope) = self.symbols.get_all_locals().last() {
            last_scope.iter().for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;
                let is_mutable_used: bool = info.2;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Local not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }

                if !is_mutable_used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Mutable local not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });
        }

        if let Some(last_scope) = self.symbols.get_all_local_constants().last() {
            last_scope.iter().for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Local constant not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });
        }

        if let Some(last_scope) = self.symbols.get_all_locals_statics().last() {
            last_scope.iter().for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;
                let is_mutable_used: bool = info.2;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Local Static not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }

                if !is_mutable_used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Local mutable static not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });
        }

        if let Some(last_scope) = self.symbols.get_all_llis().last() {
            last_scope.iter().for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("LLI not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });
        }

        if let Some(last_scope) = self.symbols.get_all_function_parameters().last() {
            last_scope.iter().for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;
                let is_mutable_used: bool = info.2;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Parameter not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }

                if !is_mutable_used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Mutable parameter not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });
        }
    }

    pub fn generate_warnings(&mut self) {
        self.symbols
            .get_all_global_statics()
            .iter()
            .for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        "Static not used".into(),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });

        self.symbols
            .get_all_global_constants()
            .iter()
            .for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Constant not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });

        self.symbols
            .get_all_functions()
            .iter()
            .for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Function not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });

        self.symbols
            .get_all_asm_functions()
            .iter()
            .for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Assembler function not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });

        self.symbols
            .get_all_enums()
            .iter()
            .for_each(|(name, info)| {
                let span: Span = info.1;
                let used: bool = info.2;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Enum not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }

                let fields: &HashMap<&str, (Span, bool)> = &info.0;

                fields.iter().for_each(|(name, info)| {
                    let span: Span = info.0;
                    let used: bool = info.1;

                    if !used {
                        self.warnings.push(ThrushCompilerIssue::Warning(
                            String::from("Enum field not used"),
                            format!("'{}' not used.", name),
                            span,
                        ));
                    }
                });
            });

        self.symbols
            .get_all_structs()
            .iter()
            .for_each(|(name, info)| {
                let span: Span = info.1;
                let used: bool = info.2;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Structure not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }

                let fields: &HashMap<&str, (Span, bool)> = &info.0;

                fields.iter().for_each(|(name, info)| {
                    let span: Span = info.0;
                    let used: bool = info.1;

                    if !used {
                        self.warnings.push(ThrushCompilerIssue::Warning(
                            String::from("Structure field not used"),
                            format!("'{}' not used.", name),
                            span,
                        ));
                    }
                });
            });
    }

    pub fn forward_all(&mut self) {
        self.ast
            .iter()
            .filter(|ast| ast.is_static())
            .for_each(|ast| {
                if let Ast::Static {
                    name,
                    metadata,
                    span,
                    ..
                } = ast
                {
                    self.symbols
                        .new_global_static(name, (*span, false, !metadata.is_mutable()));
                }
            });

        self.ast
            .iter()
            .filter(|ast| ast.is_constant())
            .for_each(|ast| {
                if let Ast::Const { name, span, .. } = ast {
                    self.symbols.new_global_constant(name, (*span, false));
                }
            });

        self.ast
            .iter()
            .filter(|stmt| stmt.is_struct())
            .for_each(|stmt| {
                if let Ast::Struct {
                    name,
                    fields,
                    span,
                    attributes,
                    ..
                } = stmt
                {
                    let mut converted_fields: HashMap<&str, (Span, bool)> =
                        HashMap::with_capacity(100);

                    for field in fields.1.iter() {
                        let field_name: &str = field.0;
                        let span: Span = field.3;

                        converted_fields.insert(field_name, (span, false));
                    }

                    self.symbols.new_struct(
                        name,
                        (converted_fields, *span, attributes.has_public_attribute()),
                    );
                }
            });

        self.ast
            .iter()
            .filter(|stmt| stmt.is_enum())
            .for_each(|stmt| {
                if let Ast::Enum {
                    name, fields, span, ..
                } = stmt
                {
                    let mut converted_fields: HashMap<&str, (Span, bool)> =
                        HashMap::with_capacity(100);

                    for field in fields.iter() {
                        let field_name: &str = field.0;
                        let expr_span: Span = field.1.get_span();

                        converted_fields.insert(field_name, (expr_span, false));
                    }

                    self.symbols
                        .new_enum(name, (converted_fields, *span, false));
                }
            });

        self.ast
            .iter()
            .filter(|stmt| stmt.is_function())
            .for_each(|stmt| {
                if let Ast::Function {
                    name,
                    span,
                    attributes,
                    ..
                } = stmt
                {
                    self.symbols
                        .new_function(name, (*span, attributes.has_public_attribute()));
                }
            });

        self.ast
            .iter()
            .filter(|stmt| stmt.is_asm_function())
            .for_each(|stmt| {
                if let Ast::AssemblerFunction {
                    name,
                    span,
                    attributes,
                    ..
                } = stmt
                {
                    self.symbols
                        .new_asm_function(name, (*span, attributes.has_public_attribute()));
                }
            });
    }

    fn add_bug(&mut self, bug: ThrushCompilerIssue) {
        self.bugs.push(bug);
    }

    fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    fn end_scope(&mut self) {
        self.symbols.end_scope();
    }

    fn advance(&mut self) {
        if !self.is_eof() {
            self.current += 1;
        }
    }

    fn peek(&self) -> &'linter Ast<'linter> {
        self.ast.get(self.current).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                "Attempting to get instruction in invalid current position.",
            );

            unreachable!()
        })
    }

    fn is_eof(&self) -> bool {
        self.current >= self.ast.len()
    }
}
