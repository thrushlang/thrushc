use table::LinterSymbolsTable;

use crate::{
    frontend::lexer::span::Span,
    standard::{
        constants::{MINIMAL_BUGS_CAPACITY, MINIMAL_WARNINGS_CAPACITY},
        diagnostic::Diagnostician,
        errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
        logging::{self, LoggingType},
        misc::CompilerFile,
    },
    types::frontend::parser::stmts::stmt::ThrushStatement,
};

mod table;

pub struct Linter<'linter> {
    stmts: &'linter [ThrushStatement<'linter>],
    current: usize,
    warnings: Vec<ThrushCompilerIssue>,
    bugs: Vec<ThrushCompilerIssue>,
    diagnostician: Diagnostician,
    symbols: LinterSymbolsTable<'linter>,
}

impl<'linter> Linter<'linter> {
    pub fn new(stmts: &'linter [ThrushStatement], file: &'linter CompilerFile) -> Self {
        Self {
            stmts,
            current: 0,
            warnings: Vec::with_capacity(MINIMAL_WARNINGS_CAPACITY),
            bugs: Vec::with_capacity(MINIMAL_BUGS_CAPACITY),
            diagnostician: Diagnostician::new(file),
            symbols: LinterSymbolsTable::new(),
        }
    }

    pub fn check(&mut self) {
        self.init();

        while !self.is_eof() {
            let stmt: &ThrushStatement = self.peek();

            self.analyze_stmt(stmt);

            self.advance();
        }

        self.generate_warnings();

        self.bugs.iter().for_each(|warn| {
            self.diagnostician.build_diagnostic(warn, LoggingType::Bug);
        });

        self.warnings.iter().for_each(|warn| {
            self.diagnostician
                .build_diagnostic(warn, LoggingType::Warning);
        });
    }

    pub fn analyze_stmt(&mut self, stmt: &'linter ThrushStatement) {
        if let ThrushStatement::EntryPoint { body, .. } = stmt {
            self.analyze_stmt(body);
        }

        if let ThrushStatement::Function {
            parameters, body, ..
        } = stmt
        {
            if body.is_block() {
                self.symbols.bulk_declare_parameters(parameters);

                self.analyze_stmt(body);

                self.symbols.destroy_all_parameters();
            }
        }

        if let ThrushStatement::BinaryOp { left, right, .. } = stmt {
            self.analyze_stmt(left);
            self.analyze_stmt(right);
        }

        if let ThrushStatement::UnaryOp { expression, .. } = stmt {
            self.analyze_stmt(expression);
        }

        if let ThrushStatement::Block { stmts, .. } = stmt {
            self.begin_scope();

            stmts.iter().for_each(|stmt| {
                self.analyze_stmt(stmt);
            });

            self.generate_scoped_warnings();

            self.end_scope();
        }

        if let ThrushStatement::For {
            local,
            actions,
            cond,
            block,
            ..
        } = stmt
        {
            self.analyze_stmt(local);
            self.analyze_stmt(actions);
            self.analyze_stmt(cond);
            self.analyze_stmt(block);
        }

        if let ThrushStatement::LLI {
            name, span, value, ..
        } = stmt
        {
            self.symbols.new_lli(name, (*span, false));

            self.analyze_stmt(value);
        }

        if let ThrushStatement::Local {
            name,
            value,
            span,
            is_mutable,
            ..
        } = stmt
        {
            self.symbols.new_local(name, (*span, false, !is_mutable));

            self.analyze_stmt(value);
        }

        if let ThrushStatement::Constructor { arguments, .. } = stmt {
            arguments.1.iter().for_each(|arg| {
                let stmt: &ThrushStatement = &arg.1;
                self.analyze_stmt(stmt);
            });
        }

        if let ThrushStatement::Call { name, span, .. } = stmt {
            if let Some(function) = self.symbols.get_function_info(name) {
                function.1 = true;
                return;
            }

            self.add_bug(ThrushCompilerIssue::Bug(
                String::from("Call not caught"),
                format!("Could not get named function '{}'.", name),
                *span,
                CompilationPosition::Linter,
                line!(),
            ));
        }

        if let ThrushStatement::Reference { name, span, .. } = stmt {
            if let Some(local) = self.symbols.get_local_info(name) {
                local.1 = true;
                return;
            }

            if let Some(parameter) = self.symbols.get_parameter_info(name) {
                parameter.1 = true;
                return;
            }

            if let Some(lli) = self.symbols.get_lli_info(name) {
                lli.1 = true;
                return;
            }

            if let Some(constant) = self.symbols.get_constant_info(name) {
                constant.1 = true;
                return;
            }

            self.add_bug(ThrushCompilerIssue::Bug(
                String::from("Reference not caught"),
                format!("Could not get reference with name '{}'.", name),
                *span,
                CompilationPosition::Linter,
                line!(),
            ));
        }

        if let ThrushStatement::Mut { source, span, .. } = stmt {
            if let Some(local_name) = source.0 {
                if let Some(local) = self.symbols.get_local_info(local_name) {
                    local.1 = true;
                    return;
                }

                self.add_bug(ThrushCompilerIssue::Bug(
                    String::from("Mutable expression not caught"),
                    format!("Could not mutable reference with name '{}'.", local_name),
                    *span,
                    CompilationPosition::Linter,
                    line!(),
                ));
            }
        }
    }

    pub fn generate_scoped_warnings(&mut self) {
        self.symbols
            .get_all_parameters()
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
    }

    pub fn init(&mut self) {
        self.stmts
            .iter()
            .filter(|stmt| stmt.is_function())
            .for_each(|stmt| {
                if let ThrushStatement::Function { name, span, .. } = stmt {
                    self.symbols.new_function(name, (*span, false));
                }
            });

        self.stmts
            .iter()
            .filter(|instruction| instruction.is_constant())
            .for_each(|instruction| {
                if let ThrushStatement::Const { name, span, .. } = instruction {
                    self.symbols.new_constant(name, (*span, false));
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

    fn peek(&self) -> &'linter ThrushStatement<'linter> {
        self.stmts.get(self.current).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                "Attempting to get instruction in invalid current position.",
            );

            unreachable!()
        })
    }

    fn is_eof(&self) -> bool {
        self.current >= self.stmts.len()
    }
}
