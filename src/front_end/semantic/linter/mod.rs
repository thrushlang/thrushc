use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::semantic::linter::symbols::LinterSymbolsTable;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

use ahash::AHashMap as HashMap;

mod builtins;
mod declarations;
mod expressions;
mod marks;
mod statements;
mod symbols;

#[derive(Debug)]
pub struct Linter<'linter> {
    ast: &'linter [Ast<'linter>],

    warnings: Vec<CompilationIssue>,
    bugs: Vec<CompilationIssue>,

    diagnostician: Diagnostician,

    symbols: LinterSymbolsTable<'linter>,
}

impl<'linter> Linter<'linter> {
    pub fn new(
        ast: &'linter [Ast],
        file: &'linter CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        Self {
            ast,
            warnings: Vec::with_capacity(100),
            bugs: Vec::with_capacity(100),
            diagnostician: Diagnostician::new(file, options),
            symbols: LinterSymbolsTable::new(),
        }
    }
}

impl<'linter> Linter<'linter> {
    pub fn check(&mut self) {
        self.declare_forward();

        for node in self.ast.iter() {
            self.analyze_decl(node);
        }

        self.generate_warnings();

        self.bugs.iter().for_each(|bug: &CompilationIssue| {
            self.diagnostician
                .dispatch_diagnostic(bug, LoggingType::Bug);
        });

        self.warnings.iter().for_each(|warn: &CompilationIssue| {
            self.diagnostician
                .dispatch_diagnostic(warn, LoggingType::Warning);
        });
    }
}

impl<'linter> Linter<'linter> {
    fn analyze_decl(&mut self, node: &'linter Ast) {
        match node {
            Ast::Enum { .. } => {
                declarations::glenum::analyze(self, node);
            }
            Ast::Static { .. } => {
                declarations::glstatic::analyze(self, node);
            }
            Ast::Const { .. } => {
                declarations::glconstant::analyze(self, node);
            }
            Ast::Function { .. } => {
                declarations::functions::analyze(self, node);
            }

            _ => (),
        }
    }

    fn analyze_stmt(&mut self, node: &'linter Ast) {
        match node {
            Ast::Local { .. } => statements::local::analyze(self, node),
            Ast::Enum { .. } => statements::lenum::analyze(self, node),
            Ast::Static { .. } => statements::staticvar::analyze(self, node),
            Ast::Const { .. } => statements::constant::analyze(self, node),
            Ast::CustomType { .. } | Ast::Struct { .. } => (),
            Ast::Block { nodes, .. } => {
                self.begin_scope();

                nodes.iter().for_each(|node| {
                    self.analyze_stmt(node);
                });

                self.generate_scoped_warnings();

                self.end_scope();
            }

            Ast::For { .. } | Ast::While { .. } | Ast::Loop { .. } => {
                statements::loops::analyze(self, node);
            }

            Ast::Continue { .. } | Ast::Break { .. } => (),

            Ast::If { .. } | Ast::Elif { .. } | Ast::Else { .. } => {
                statements::conditional::analyze(self, node);
            }

            Ast::Mut { .. } => statements::mutation::analyze(self, node),

            Ast::Return { .. } => statements::terminator::analyze(self, node),

            expr => self.analyze_expr(expr),
        }
    }

    fn analyze_expr(&mut self, expr: &'linter Ast) {
        expressions::analyze(self, expr);
    }
}

impl Linter<'_> {
    fn declare_forward(&mut self) {
        for ast in self.ast.iter() {
            match ast {
                Ast::Static {
                    name,
                    metadata,
                    span,
                    ..
                } => {
                    self.symbols
                        .new_global_static(name, (*span, false, !metadata.is_mutable()));
                }
                Ast::Const { name, span, .. } => {
                    self.symbols.new_global_constant(name, (*span, false));
                }
                Ast::Struct {
                    name,
                    fields,
                    span,
                    attributes,
                    ..
                } => {
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

                Ast::Enum {
                    name, fields, span, ..
                } => {
                    let mut converted_fields: HashMap<&str, (Span, bool)> =
                        HashMap::with_capacity(100);

                    for field in fields.iter() {
                        let field_name: &str = field.0;
                        let expr_span: Span = field.2.get_span();

                        converted_fields.insert(field_name, (expr_span, false));
                    }

                    self.symbols
                        .new_enum(name, (converted_fields, *span, false));
                }

                Ast::Function {
                    name,
                    span,
                    attributes,
                    ..
                } => {
                    self.symbols
                        .new_function(name, (*span, attributes.has_public_attribute()));
                }

                Ast::Intrinsic {
                    name,
                    span,
                    attributes,
                    ..
                } => {
                    self.symbols
                        .new_intrinsic(name, (*span, attributes.has_public_attribute()));
                }

                Ast::AssemblerFunction {
                    name,
                    span,
                    attributes,
                    ..
                } => {
                    self.symbols
                        .new_asm_function(name, (*span, attributes.has_public_attribute()));
                }

                _ => (),
            }
        }
    }
}

impl Linter<'_> {
    fn generate_scoped_warnings(&mut self) {
        let mut warnings: Vec<CompilationIssue> = Vec::with_capacity(u8::MAX.into());

        if let Some(last_scope) = self.symbols.get_all_locals().last() {
            for (name, info) in last_scope.iter() {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    warnings.push(CompilationIssue::Warning(
                        CompilationIssueCode::W0005,
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            }
        }

        if let Some(last_scope) = self.symbols.get_all_local_constants().last() {
            for (name, info) in last_scope.iter() {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    warnings.push(CompilationIssue::Warning(
                        CompilationIssueCode::W0010,
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            }
        }

        if let Some(last_scope) = self.symbols.get_all_locals_statics().last() {
            for (name, info) in last_scope.iter() {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    warnings.push(CompilationIssue::Warning(
                        CompilationIssueCode::W0009,
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            }
        }

        if let Some(last_scope) = self.symbols.get_all_llis().last() {
            for (name, info) in last_scope.iter() {
                let span: Span = info.0;
                let used: bool = info.1;

                if !used {
                    warnings.push(CompilationIssue::Warning(
                        CompilationIssueCode::W0007,
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            }
        }

        self.add_bulk_warnings(warnings);
    }

    fn generate_scoped_function_warnings(&mut self) {
        let mut warnings: Vec<CompilationIssue> = Vec::with_capacity(u8::MAX.into());

        for (name, info) in self.symbols.get_all_function_parameters().iter() {
            let span: Span = info.0;
            let used: bool = info.1;

            if !used {
                warnings.push(CompilationIssue::Warning(
                    CompilationIssueCode::W0008,
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        }

        self.add_bulk_warnings(warnings);
    }

    fn generate_warnings(&mut self) {
        let mut warnings: Vec<CompilationIssue> = Vec::with_capacity(u8::MAX.into());

        for (name, info) in self.symbols.get_all_global_statics().iter() {
            let span: Span = info.0;
            let used: bool = info.1;

            if !used {
                warnings.push(CompilationIssue::Warning(
                    CompilationIssueCode::W0009,
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        }

        for (name, info) in self.symbols.get_all_global_constants().iter() {
            let span: Span = info.0;
            let used: bool = info.1;

            if !used {
                warnings.push(CompilationIssue::Warning(
                    CompilationIssueCode::W0010,
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        }

        for (name, info) in self.symbols.get_all_functions().iter() {
            let span: Span = info.0;
            let used: bool = info.1;

            if !used {
                warnings.push(CompilationIssue::Warning(
                    CompilationIssueCode::W0017,
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        }

        for (name, info) in self.symbols.get_all_asm_functions().iter() {
            let span: Span = info.0;
            let used: bool = info.1;

            if !used {
                warnings.push(CompilationIssue::Warning(
                    CompilationIssueCode::W0011,
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        }

        for (name, info) in self.symbols.get_all_enums().iter() {
            let span: Span = info.1;
            let used: bool = info.2;

            if !used {
                warnings.push(CompilationIssue::Warning(
                    CompilationIssueCode::W0012,
                    format!("'{}' not used.", name),
                    span,
                ));
            }

            let fields: &HashMap<&str, (Span, bool)> = &info.0;

            for (field_name, field_info) in fields.iter() {
                let span: Span = field_info.0;
                let used: bool = field_info.1;

                if !used {
                    warnings.push(CompilationIssue::Warning(
                        CompilationIssueCode::W0013,
                        format!("'{}' not used.", field_name),
                        span,
                    ));
                }
            }
        }

        for (name, info) in self.symbols.get_all_intrinsics().iter() {
            let span: Span = info.0;
            let used: bool = info.1;

            if !used {
                warnings.push(CompilationIssue::Warning(
                    CompilationIssueCode::W0014,
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        }

        for (name, info) in self.symbols.get_all_structs().iter() {
            let span: Span = info.1;
            let used: bool = info.2;

            if !used {
                warnings.push(CompilationIssue::Warning(
                    CompilationIssueCode::W0015,
                    format!("'{}' not used.", name),
                    span,
                ));
            }

            let fields: &HashMap<&str, (Span, bool)> = &info.0;

            for (field_name, field_info) in fields.iter() {
                let span: Span = field_info.0;
                let used: bool = field_info.1;

                if !used {
                    warnings.push(CompilationIssue::Warning(
                        CompilationIssueCode::W0016,
                        format!("'{}' not used.", field_name),
                        span,
                    ));
                }
            }
        }

        self.add_bulk_warnings(warnings);
    }
}

impl Linter<'_> {
    #[inline]
    pub fn add_bulk_warnings(&mut self, warnings: Vec<CompilationIssue>) {
        self.warnings.extend(warnings);
    }
}

impl Linter<'_> {
    #[inline]
    fn add_bug(&mut self, bug: CompilationIssue) {
        self.bugs.push(bug);
    }
}

impl<'linter> Linter<'linter> {
    #[inline]
    fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    #[inline]
    fn end_scope(&mut self) {
        self.symbols.end_scope();
    }
}

impl<'linter> Linter<'linter> {
    #[inline]
    pub fn get_mut_symbols(&mut self) -> &mut LinterSymbolsTable<'linter> {
        &mut self.symbols
    }
}
