#![allow(clippy::only_used_in_recursion)]

use table::TypeCheckerSymbolsTable;

use crate::{
    core::{
        compiler::options::CompilerFile, console::logging::LoggingType,
        diagnostic::diagnostician::Diagnostician, errors::standard::ThrushCompilerIssue,
    },
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        semantic::typechecker::position::TypeCheckerPosition,
        types::{
            ast::Ast,
            lexer::{
                ThrushType,
                traits::{ThrushTypeMutableExtensions, ThrushTypeNumericExtensions},
            },
            parser::stmts::traits::ThrushAttributesExtensions,
        },
    },
};

mod builtins;
mod call;
mod casts;
mod conditionals;
mod constant;
mod deref;
mod expressions;
mod exprvalidations;
mod functions;
mod lli;
mod local;
mod loops;
mod position;
mod terminator;

mod table;

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

        if let Ast::Struct { .. } = stmt {
            return Ok(());
        }

        if let Ast::Enum { .. } = stmt {
            return Ok(());
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

        if let Ast::If { .. } = stmt {
            conditionals::validate_conditional(self, stmt)?;

            return Ok(());
        }

        if let Ast::Elif { .. } = stmt {
            conditionals::validate_conditional(self, stmt)?;

            return Ok(());
        }

        if let Ast::Else { .. } = stmt {
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

    fn validate_type_cast(
        &self,
        from_type: &ThrushType,
        cast_type: &ThrushType,
        is_allocated_ref: bool,
        span: &Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if (from_type.is_numeric_type() && cast_type.is_numeric_type())
            || (from_type.is_numeric_type()
                || from_type.is_array_type()
                || from_type.is_fixed_array_type()
                || from_type.is_struct_type() && is_allocated_ref && cast_type.is_ptr_type())
            || (from_type.is_mut_numeric_type() && cast_type.is_numeric_type())
            || (from_type.is_mut_type() || from_type.is_ptr_type() && cast_type.is_ptr_type())
            || (from_type.is_ptr_type() || cast_type.is_mut_type())
            || (from_type.is_str_type() && is_allocated_ref && cast_type.is_ptr_type())
        {
            Ok(())
        } else {
            Err(ThrushCompilerIssue::Error(
                "Type error".into(),
                format!("Cannot cast '{}' to '{}'", from_type, cast_type),
                None,
                *span,
            ))
        }
    }

    pub fn validate_types(
        &self,
        target_type: &ThrushType,
        from_type: &ThrushType,
        expression: Option<&Ast>,
        operator: Option<&TokenType>,
        position: Option<TypeCheckerPosition>,
        span: &Span,
    ) -> Result<(), ThrushCompilerIssue> {
        let error: ThrushCompilerIssue = ThrushCompilerIssue::Error(
            String::from("Mismatched types"),
            format!("Expected '{}' but found '{}'.", target_type, from_type),
            None,
            *span,
        );

        if let Some(Ast::BinaryOp {
            operator,
            kind: expression_type,
            ..
        }) = expression
        {
            return self.validate_types(
                target_type,
                expression_type,
                None,
                Some(operator),
                position,
                span,
            );
        }

        if let Some(Ast::UnaryOp {
            operator,
            kind: expression_type,
            ..
        }) = expression
        {
            return self.validate_types(
                target_type,
                expression_type,
                None,
                Some(operator),
                position,
                span,
            );
        }

        if let Some(Ast::Group {
            expression,
            kind: expression_type,
            ..
        }) = expression
        {
            return self.validate_types(
                target_type,
                expression_type,
                Some(expression),
                None,
                position,
                span,
            );
        }

        match (target_type, from_type, operator) {
            (ThrushType::Char, ThrushType::Char, None) => Ok(()),
            (ThrushType::Str, ThrushType::Str, None) => Ok(()),
            (ThrushType::Struct(_, target_fields), ThrushType::Struct(_, from_fields), None) => {
                if target_fields.len() != from_fields.len() {
                    return Err(error);
                }

                target_fields.iter().zip(from_fields.iter()).try_for_each(
                    |(target_field, from_field)| {
                        self.validate_types(target_field, from_field, None, None, position, span)
                    },
                )?;

                Ok(())
            }

            (ThrushType::Addr, ThrushType::Addr, None) => Ok(()),

            (
                ThrushType::FixedArray(type_a, size_a),
                ThrushType::FixedArray(type_b, size_b),
                None,
            ) => {
                if size_a == size_b {
                    self.validate_types(type_a, type_b, None, None, position, span)?;
                    return Ok(());
                }

                Err(error)
            }

            (ThrushType::Array(target_type), ThrushType::Array(from_type), None) => {
                self.validate_types(target_type, from_type, None, None, position, span)?;

                Ok(())
            }

            (
                ThrushType::Mut(target_type),
                from_type,
                Some(
                    TokenType::BangEq
                    | TokenType::EqEq
                    | TokenType::LessEq
                    | TokenType::Less
                    | TokenType::Greater
                    | TokenType::GreaterEq
                    | TokenType::And
                    | TokenType::Or
                    | TokenType::Bang
                    | TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift,
                )
                | None,
            ) if position.is_some_and(|position| position.at_local())
                && !from_type.is_mut_type() && !from_type.is_ptr_type() =>
            {
                self.validate_types(target_type, from_type, expression, operator, position, span)?;

                Ok(())
            }

            (
                ThrushType::Mut(..),
                ThrushType::Mut(..),
                Some(
                    TokenType::BangEq
                    | TokenType::EqEq
                    | TokenType::LessEq
                    | TokenType::Less
                    | TokenType::Greater
                    | TokenType::GreaterEq
                    | TokenType::And
                    | TokenType::Or
                    | TokenType::Bang
                    | TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift,
                )
                | None,
            ) => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Memory aliasing isn't allowed at high-level pointers; use Low Level Instructions (LLI) instead.".into(),
                None,
                *span,
            )),

            (ThrushType::Ptr(None), ThrushType::Ptr(None), None) => Ok(()),
            (ThrushType::Ptr(Some(target_type)), ThrushType::Ptr(Some(from_type)), None) => {
                self.validate_types(target_type, from_type, expression, operator, position, span)?;

                Ok(())
            }

            (
                ThrushType::Bool,
                ThrushType::Bool,
                Some(
                    TokenType::BangEq
                    | TokenType::EqEq
                    | TokenType::LessEq
                    | TokenType::Less
                    | TokenType::Greater
                    | TokenType::GreaterEq
                    | TokenType::And
                    | TokenType::Or
                    | TokenType::Bang,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::S8,
                ThrushType::S8,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::S16,
                ThrushType::S16 | ThrushType::S8,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::S32,
                ThrushType::S32 | ThrushType::S16 | ThrushType::S8,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::S64,
                ThrushType::S64 | ThrushType::S32 | ThrushType::S16 | ThrushType::S8,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::U8,
                ThrushType::U8,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::U16,
                ThrushType::U16 | ThrushType::U8,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::U32,
                ThrushType::U32 | ThrushType::U16 | ThrushType::U8,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::U64,
                ThrushType::U64 | ThrushType::U32 | ThrushType::U16 | ThrushType::U8,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::F32,
                ThrushType::F32,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::F64,
                ThrushType::F64 | ThrushType::F32,
                Some(
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::LShift
                    | TokenType::RShift
                    | TokenType::PlusPlus
                    | TokenType::MinusMinus,
                )
                | None,
            ) => Ok(()),

            (ThrushType::Void, ThrushType::Void, None) => Ok(()),

            _ => Err(error),
        }
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

    pub fn add_warning(&mut self, warning: ThrushCompilerIssue) {
        self.warnings.push(warning);
    }

    pub fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }

    pub fn add_bug(&mut self, error: ThrushCompilerIssue) {
        self.bugs.push(error);
    }

    pub fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    pub fn end_scope(&mut self) {
        self.symbols.end_scope();
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
