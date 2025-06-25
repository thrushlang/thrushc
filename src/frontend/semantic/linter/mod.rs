use ahash::AHashMap as HashMap;

use table::LinterSymbolsTable;

use crate::{
    core::{
        compiler::options::CompilerFile,
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
        errors::standard::ThrushCompilerIssue,
    },
    frontend::{
        lexer::span::Span, types::ast::Ast,
        types::parser::stmts::traits::ThrushAttributesExtensions,
    },
};

pub mod attributes;

mod builtins;
mod casts;
mod conditionals;
mod deref;
mod enums;
mod expressions;
mod functions;
mod lli;
mod local;
mod loops;
mod table;
mod terminator;

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
        self.init();

        while !self.is_eof() {
            let stmt: &Ast = self.peek();

            self.analyze_ast(stmt);

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

    pub fn analyze_ast(&mut self, stmt: &'linter Ast) {
        /* ######################################################################


            LINTER FUNCTIONS | START


        ########################################################################*/

        if let Ast::EntryPoint { .. } | Ast::Function { .. } = stmt {
            functions::analyze_function(self, stmt);
        }

        /* ######################################################################


            LINTER FUNCTIONS | END


        ########################################################################*/

        /* ######################################################################


            LINTER STATEMENTS | START


        ########################################################################*/

        if let Ast::Block { stmts, .. } = stmt {
            self.begin_scope();

            stmts.iter().for_each(|stmt| {
                self.analyze_ast(stmt);
            });

            self.generate_scoped_warnings();

            self.end_scope();
        }

        /* ######################################################################


            LINTER STATEMENTS | END


        ########################################################################*/

        /* ######################################################################


            LINTER DECLARATIONS | START


        ########################################################################*/

        if let Ast::Enum { .. } = stmt {
            enums::analyze_enum(self, stmt);
        }

        if let Ast::LLI { .. } = stmt {
            lli::analyze_lli(self, stmt);
        }

        if let Ast::Local { .. } = stmt {
            local::analyze_local(self, stmt);
        }

        /* ######################################################################


            LINTER TERMINATOR | START


        ########################################################################*/

        if let Ast::Return { .. } = stmt {
            terminator::analyze_terminator(self, stmt);
        }

        /* ######################################################################


            LINTER TERMINATOR | END


        ########################################################################*/

        /* ######################################################################


            LINTER CONDITIONALS | START


        ########################################################################*/

        if let Ast::If { .. } = stmt {
            conditionals::analyze_conditional(self, stmt);
        }

        if let Ast::Elif { .. } = stmt {
            conditionals::analyze_conditional(self, stmt);
        }

        if let Ast::Else { .. } = stmt {
            conditionals::analyze_conditional(self, stmt);
        }

        /* ######################################################################


            LINTER CONDITIONALS | END


        ########################################################################*/

        /* ######################################################################


            LINTER LOOPS | START


        ########################################################################*/

        if let Ast::For { .. } = stmt {
            loops::analyze_loop(self, stmt);
        }

        if let Ast::While { .. } = stmt {
            loops::analyze_loop(self, stmt);
        }

        if let Ast::Loop { .. } = stmt {
            loops::analyze_loop(self, stmt);
        }

        /* ######################################################################


            LINTER LOOPS | END


        ########################################################################*/

        /* ######################################################################


            LINTER DEREFERENCE | START


        ########################################################################*/

        if let Ast::Deref { .. } = stmt {
            deref::analyze_dereference(self, stmt);
        }

        /* ######################################################################


            LINTER DEREFERENCE | END


        ########################################################################*/

        /* ######################################################################


            LINTER LLI | START


        ########################################################################*/

        if let Ast::Write { .. } = stmt {
            lli::analyze_lli(self, stmt);
        }

        if let Ast::Address { .. } = stmt {
            lli::analyze_lli(self, stmt);
        }

        if let Ast::Load { .. } = stmt {
            lli::analyze_lli(self, stmt);
        }

        /* ######################################################################


            LINTER LLI | END


        ########################################################################*/

        /* ######################################################################


            LINTER CASTS | START


        ########################################################################*/

        if let Ast::As { .. } = stmt {
            casts::analyze_cast(self, stmt);
        }

        /* ######################################################################


            LINTER CASTS | END


        ########################################################################*/

        /* ######################################################################


            LINTER BUILTINS | START


        ########################################################################*/

        if let Ast::Builtin { builtin, .. } = stmt {
            builtins::analyze_builtin(self, builtin);
        }

        /* ######################################################################


            LINTER BUILTINS | END


        ########################################################################*/

        /* ######################################################################


            LINTER EXPRESSIONS | START


        ########################################################################*/

        expressions::analyze_expression(self, stmt);

        /* ######################################################################


            LINTER EXPRESSIONS | END


        ########################################################################*/
    }

    pub fn generate_scoped_warnings(&mut self) {
        self.symbols
            .get_all_function_parameters()
            .iter()
            .for_each(|parameter| {
                let name: &str = parameter.0;
                let span: Span = parameter.1.0;
                let used: bool = parameter.1.1;
                let is_mutable_used: bool = parameter.1.2;

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

        if let Some(last_scope) = self.symbols.get_all_llis().last() {
            last_scope.iter().for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Low Level Instruction not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });
        }
    }

    pub fn generate_warnings(&mut self) {
        self.symbols
            .get_all_constants()
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

    pub fn init(&mut self) {
        self.ast
            .iter()
            .filter(|instruction| instruction.is_constant())
            .for_each(|instruction| {
                if let Ast::Const {
                    name,
                    span,
                    attributes,
                    ..
                } = instruction
                {
                    self.symbols
                        .new_constant(name, (*span, attributes.has_public_attribute()));
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
