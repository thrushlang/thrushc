use ahash::AHashMap as HashMap;

use table::LinterSymbolsTable;

use crate::{
    core::{
        compiler::options::CompilerFile,
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
        errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    },
    frontend::{
        lexer::span::Span,
        types::parser::stmts::{stmt::ThrushStatement, traits::ThrushAttributesExtensions},
    },
};

pub mod attributes;
mod table;

#[derive(Debug)]
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
            warnings: Vec::with_capacity(100),
            bugs: Vec::with_capacity(100),
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

        self.bugs.iter().for_each(|bug: &ThrushCompilerIssue| {
            self.diagnostician.build_diagnostic(bug, LoggingType::Bug);
        });

        self.warnings.iter().for_each(|warn: &ThrushCompilerIssue| {
            self.diagnostician
                .build_diagnostic(warn, LoggingType::Warning);
        });
    }

    pub fn analyze_stmt(&mut self, stmt: &'linter ThrushStatement) {
        /* ######################################################################


            LINTER STMTS | START


        ########################################################################*/

        if let ThrushStatement::EntryPoint { body, .. } = stmt {
            self.analyze_stmt(body);
        }

        if let ThrushStatement::Enum { fields, .. } = stmt {
            fields.iter().for_each(|field| {
                let field_expr: &ThrushStatement = &field.1;
                self.analyze_stmt(field_expr);
            });
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

        if let ThrushStatement::Return {
            expression: Some(expr),
            ..
        } = stmt
        {
            self.analyze_stmt(expr);
        }
        /* ######################################################################


            LINTER STMTS | END


        ########################################################################*/

        /* ######################################################################


            LINTER EXPRESSIONS | START


        ########################################################################*/

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

        if let ThrushStatement::Write {
            write_to,
            write_value,
            ..
        } = stmt
        {
            if let Some(reference_name) = write_to.0 {
                if let Some(lli) = self.symbols.get_lli_info(reference_name) {
                    lli.1 = true;
                }
            }

            if let Some(expr) = &write_to.1 {
                self.analyze_stmt(expr);
            }

            self.analyze_stmt(write_value);
        }

        if let ThrushStatement::CastPtr { from, .. } = stmt {
            self.analyze_stmt(from);
        }

        if let ThrushStatement::Cast { from, .. } = stmt {
            self.analyze_stmt(from);
        }

        if let ThrushStatement::CastRaw { from, .. } = stmt {
            self.analyze_stmt(from);
        }

        if let ThrushStatement::RawPtr { from, .. } = stmt {
            self.analyze_stmt(from);
        }

        if let ThrushStatement::Address { name, span, .. } = stmt {
            if let Some(local) = self.symbols.get_lli_info(name) {
                local.1 = true;
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

        if let ThrushStatement::Load { load, .. } = stmt {
            if let Some(reference_name) = load.0 {
                if let Some(lli) = self.symbols.get_lli_info(reference_name) {
                    lli.1 = true;
                }
            }

            if let Some(expr) = &load.1 {
                self.analyze_stmt(expr);
            }
        }

        if let ThrushStatement::Constructor {
            name,
            arguments,
            span,
            ..
        } = stmt
        {
            arguments.1.iter().for_each(|arg| {
                let stmt: &ThrushStatement = &arg.1;
                self.analyze_stmt(stmt);
            });

            if let Some(structure) = self.symbols.get_struct_info(name) {
                structure.2 = true;
                return;
            }

            self.add_bug(ThrushCompilerIssue::Bug(
                String::from("Structure not caught"),
                format!("Could not get named struct with name '{}'.", name),
                *span,
                CompilationPosition::Linter,
                line!(),
            ));
        }

        if let ThrushStatement::AsmCall {
            name, span, args, ..
        } = stmt
        {
            if let Some(asm_function) = self.symbols.get_asm_function_info(name) {
                asm_function.1 = true;

                args.iter().for_each(|arg| {
                    self.analyze_stmt(arg);
                });

                return;
            }

            self.add_bug(ThrushCompilerIssue::Bug(
                String::from("Call not caught"),
                format!("Could not get named assembler function '{}'.", name),
                *span,
                CompilationPosition::Linter,
                line!(),
            ));
        }

        if let ThrushStatement::Call {
            name, span, args, ..
        } = stmt
        {
            if let Some(function) = self.symbols.get_function_info(name) {
                function.1 = true;

                args.iter().for_each(|arg| {
                    self.analyze_stmt(arg);
                });

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

        if let ThrushStatement::EnumValue {
            name, value, span, ..
        } = stmt
        {
            if let Some((enum_name, field_name)) = self.symbols.split_enum_field_name(name) {
                if let Some(union) = self.symbols.get_enum_info(enum_name) {
                    union.2 = true;
                }

                if let Some(enum_field) = self.symbols.get_enum_field_info(enum_name, field_name) {
                    enum_field.1 = true;
                }

                self.analyze_stmt(value);

                return;
            }

            self.add_bug(ThrushCompilerIssue::Bug(
                String::from("Enum value not caught"),
                format!("Could not get correct name of the enum field '{}'.", name),
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
        self.stmts
            .iter()
            .filter(|instruction| instruction.is_constant())
            .for_each(|instruction| {
                if let ThrushStatement::Const {
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

        self.stmts
            .iter()
            .filter(|stmt| stmt.is_struct())
            .for_each(|stmt| {
                if let ThrushStatement::Struct {
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

        self.stmts
            .iter()
            .filter(|stmt| stmt.is_enum())
            .for_each(|stmt| {
                if let ThrushStatement::Enum {
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

        self.stmts
            .iter()
            .filter(|stmt| stmt.is_function())
            .for_each(|stmt| {
                if let ThrushStatement::Function {
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

        self.stmts
            .iter()
            .filter(|stmt| stmt.is_asm_function())
            .for_each(|stmt| {
                if let ThrushStatement::AssemblerFunction {
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
