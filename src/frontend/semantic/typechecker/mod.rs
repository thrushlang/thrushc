#![allow(clippy::only_used_in_recursion)]

use marks::{TypeCheckerTypeContext, TypeCheckerTypePosition};
use table::TypeCheckerSymbolsTable;

use crate::{
    frontend::lexer::span::Span,
    standard::{
        constants::MINIMAL_ERROR_CAPACITY,
        diagnostic::Diagnostician,
        errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
        logging::LoggingType,
        misc::CompilerFile,
    },
    types::frontend::{
        lexer::{tokenkind::TokenKind, types::ThrushType},
        parser::stmts::{stmt::ThrushStatement, traits::CompilerAttributesExtensions},
    },
};

mod marks;
mod table;

#[derive(Debug)]
pub struct TypeChecker<'stmts> {
    stmts: &'stmts [ThrushStatement<'stmts>],
    position: usize,
    errors: Vec<ThrushCompilerIssue>,
    type_ctx: TypeCheckerTypeContext<'stmts>,
    symbols: TypeCheckerSymbolsTable<'stmts>,
    diagnostician: Diagnostician,
}

impl<'stmts> TypeChecker<'stmts> {
    pub fn new(stmts: &'stmts [ThrushStatement<'stmts>], file: &'stmts CompilerFile) -> Self {
        Self {
            stmts,
            position: 0,
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            type_ctx: TypeCheckerTypeContext::new(),
            symbols: TypeCheckerSymbolsTable::new(),
            diagnostician: Diagnostician::new(file),
        }
    }

    pub fn check(&mut self) -> bool {
        self.init();

        while !self.is_eof() {
            let current_stmt: &ThrushStatement = self.peek();

            if let Err(type_error) = self.check_stmt(current_stmt) {
                self.add_error(type_error);
            }

            self.advance();
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .build_diagnostic(error, LoggingType::Error);
            });

            return true;
        }

        false
    }

    fn check_stmt(&mut self, stmt: &'stmts ThrushStatement) -> Result<(), ThrushCompilerIssue> {
        if let ThrushStatement::EntryPoint { body, .. } = stmt {
            if let Err(type_error) = self.check_stmt(body) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::For {
            local,
            cond,
            actions,
            block,
            ..
        } = stmt
        {
            if let Err(type_error) = self.check_stmt(local) {
                self.add_error(type_error);
            }

            if let Err(type_error) = self.check_stmt(cond) {
                self.add_error(type_error);
            }

            if let Err(type_error) = self.check_stmt(actions) {
                self.add_error(type_error);
            }

            if let Err(type_error) = self.check_stmt(block) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::Group { expression, .. } = stmt {
            if let Err(type_error) = self.check_stmt(expression) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::Const {
            kind: target_type,
            value,
            span,
            ..
        } = stmt
        {
            let from_type: &ThrushType = value.get_value_type()?;

            if let Err(mismatch_type_error) =
                self.is_mismatch_type(target_type, from_type, Some(value), None, span)
            {
                self.add_error(mismatch_type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::Function {
            body, return_type, ..
        } = stmt
        {
            self.type_ctx
                .set_type_position(TypeCheckerTypePosition::Function);
            self.type_ctx.set_function_type(return_type);

            if let Err(type_error) = self.check_stmt(body) {
                self.add_error(type_error);
            }

            self.type_ctx
                .set_type_position(TypeCheckerTypePosition::None);
            self.type_ctx.set_function_type(&ThrushType::Void);

            return Ok(());
        }

        if let ThrushStatement::Block { stmts, .. } = stmt {
            self.begin_scope();

            stmts.iter().for_each(|stmt| {
                if let Err(type_error) = self.check_stmt(stmt) {
                    self.add_error(type_error);
                }
            });

            self.end_scope();

            return Ok(());
        }

        if let ThrushStatement::Local {
            name,
            kind: local_type,
            value: local_value,
            span,
            ..
        } = stmt
        {
            self.symbols.new_local(name, local_type);

            let local_value_type: &ThrushType = local_value.get_value_type()?;

            if let Err(mismatch_type_error) =
                self.is_mismatch_type(local_type, local_value_type, Some(local_value), None, span)
            {
                self.add_error(mismatch_type_error);
            }

            if let Err(type_error) = self.check_stmt(local_value) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::LLI {
            kind: lli_type,
            value: lli_value,
            span,
            ..
        } = stmt
        {
            let lli_value_type: &ThrushType = lli_value.get_value_type()?;

            if let Err(mismatch_type_error) =
                self.is_mismatch_type(lli_type, lli_value_type, Some(lli_value), None, span)
            {
                self.add_error(mismatch_type_error);
            }

            if let Err(type_error) = self.check_stmt(lli_value) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::BinaryOp {
            left,
            operator,
            right,
            span,
            ..
        } = stmt
        {
            if let Err(mismatch_type_error) = self.check_binaryop(
                operator,
                left.get_value_type()?,
                right.get_value_type()?,
                *span,
            ) {
                self.add_error(mismatch_type_error);
            }

            if let Err(type_error) = self.check_stmt(left) {
                self.add_error(type_error);
            }

            if let Err(type_error) = self.check_stmt(right) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::UnaryOp {
            operator,
            expression,
            span,
            ..
        } = stmt
        {
            if let Err(mismatch_type_error) =
                self.check_unary(operator, expression.get_value_type()?, *span)
            {
                self.add_error(mismatch_type_error);
            }

            if let Err(type_error) = self.check_stmt(expression) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::Call {
            name, args, span, ..
        } = stmt
        {
            if let Some(function) = self.symbols.get_function(name) {
                let parameter_types: &[ThrushType] = function.0;
                let ignore_more_arguments: bool = function.1;

                let parameter_types_size: usize = parameter_types.len();

                let mut parameter_types_displayed: String = String::with_capacity(100);

                parameter_types.iter().for_each(|parameter_type| {
                    parameter_types_displayed.push_str(&format!("{} ", parameter_type));
                });

                if args.len() != parameter_types_size && !ignore_more_arguments {
                    self.add_error(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        format!(
                            "Expected \"{}\" arguments, with types \"{}\".",
                            parameter_types_size, parameter_types_displayed
                        ),
                        None,
                        *span,
                    ));

                    return Ok(());
                }

                if !ignore_more_arguments {
                    for (target_type, expr) in parameter_types.iter().zip(args.iter()) {
                        let from_type: &ThrushType = expr.get_value_type()?;
                        let span: Span = expr.get_span();

                        if let Err(mismatched_types_error) =
                            self.is_mismatch_type(target_type, from_type, Some(expr), None, &span)
                        {
                            self.add_error(mismatched_types_error);
                        }
                    }
                }

                return Ok(());
            }

            self.errors.push(ThrushCompilerIssue::Bug(
                String::from("Call not caught"),
                format!("Could not get named function '{}'.", name),
                *span,
                CompilationPosition::TypeChecker,
                line!(),
            ));
        }

        if let ThrushStatement::MethodCall {
            name, args, span, ..
        } = stmt
        {
            if let Some((binding_name, bind_name)) = self.symbols.split_method_call_name(name) {
                if let Some(types) = self
                    .symbols
                    .get_specific_method_definition(binding_name, bind_name)
                {
                    let types_size: usize = types.len();

                    let mut types_displayed: String = String::with_capacity(100);

                    types.iter().for_each(|parameter_type| {
                        types_displayed.push_str(&format!("{}", parameter_type));
                    });

                    if args.len() != types_size {
                        self.add_error(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            format!(
                                "Expected \"{}\" arguments, with types \"{}\".",
                                types_size, types_displayed
                            ),
                            None,
                            *span,
                        ));

                        return Ok(());
                    }

                    for (target_type, arg) in types.iter().zip(args.iter()) {
                        let from_type: &ThrushType = arg.get_value_type()?;
                        let span: Span = arg.get_span();

                        if let Err(mismatched_types_error) =
                            self.is_mismatch_type(target_type, from_type, Some(arg), None, &span)
                        {
                            self.add_error(mismatched_types_error);
                        }
                    }

                    return Ok(());
                }

                self.errors.push(ThrushCompilerIssue::Bug(
                    String::from("Method canonical name not caught"),
                    format!("It was not possible to obtain the canonical name of the methods, which is the parent of '{}'.", name),
                    *span,
                    CompilationPosition::TypeChecker,
                    line!(),
                ));
            }

            self.errors.push(ThrushCompilerIssue::Bug(
                String::from("Methods definition not caught"),
                format!("Could not get named method '{}'.", name),
                *span,
                CompilationPosition::TypeChecker,
                line!(),
            ));
        }

        if let ThrushStatement::Mut {
            source,
            value,
            span,
            ..
        } = stmt
        {
            if let (Some(local_name), None) = source {
                if let Some(local_type) = self.symbols.get_local(local_name) {
                    if let Err(mismatched_types_error) = self.is_mismatch_type(
                        local_type,
                        value.get_value_type()?,
                        Some(value),
                        None,
                        span,
                    ) {
                        self.add_error(mismatched_types_error);
                    }

                    return Ok(());
                }

                self.errors.push(ThrushCompilerIssue::Bug(
                    String::from("Could not catch a local"),
                    String::from("A location could not be obtained for processing."),
                    *span,
                    CompilationPosition::TypeChecker,
                    line!(),
                ));
            }

            if let (None, Some(expression)) = source {
                if let Err(mismatched_types_error) = self.is_mismatch_type(
                    expression.get_value_type()?,
                    value.get_value_type()?,
                    Some(value),
                    None,
                    span,
                ) {
                    self.add_error(mismatched_types_error);
                }

                return Ok(());
            }

            self.errors.push(ThrushCompilerIssue::Bug(
                String::from("Non-trapped mutable expression."),
                String::from("The mutable expression could not be caught for processing."),
                *span,
                CompilationPosition::TypeChecker,
                line!(),
            ));
        }

        if let ThrushStatement::Return {
            expression, span, ..
        } = stmt
        {
            let from_type = if let Some(expression) = expression {
                expression.get_value_type()?
            } else {
                &ThrushType::Void
            };

            if let Err(mismatched_types_error) = self.is_mismatch_type(
                self.type_ctx.get_function_type(),
                from_type,
                expression.as_deref(),
                None,
                span,
            ) {
                self.add_error(mismatched_types_error);
            }
        }

        if let ThrushStatement::If {
            cond, elfs, span, ..
        } = stmt
        {
            if let Err(mismatched_types_error) = self.is_mismatch_type(
                &ThrushType::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                span,
            ) {
                self.add_error(mismatched_types_error);
            }

            for elif in elfs.iter() {
                if let ThrushStatement::Elif { cond, span, .. } = elif {
                    if let Err(mismatched_types_error) = self.is_mismatch_type(
                        &ThrushType::Bool,
                        cond.get_value_type()?,
                        Some(cond),
                        None,
                        span,
                    ) {
                        self.add_error(mismatched_types_error);
                    }
                }
            }

            return Ok(());
        }

        if let ThrushStatement::Constructor { arguments, .. } = stmt {
            let args: &[(&str, ThrushStatement<'_>, ThrushType, u32)] = &arguments.1;

            for arg in args.iter() {
                let expression: &ThrushStatement = &arg.1;
                let expression_span: Span = expression.get_span();

                let target_type: &ThrushType = &arg.2;
                let from_type: &ThrushType = expression.get_value_type()?;

                if let Err(mismatched_types_error) = self.is_mismatch_type(
                    target_type,
                    from_type,
                    Some(expression),
                    None,
                    &expression_span,
                ) {
                    self.add_error(mismatched_types_error);
                }
            }

            return Ok(());
        }

        if let ThrushStatement::Load { .. }
        | ThrushStatement::Write { .. }
        | ThrushStatement::Address { .. }
        | ThrushStatement::Alloc { .. } = stmt
        {
            return Ok(());
        }

        if let ThrushStatement::Struct { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::EnumValue { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::Reference { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::Integer { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::Boolean { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::Str { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::Float { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::Null { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::NullPtr { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::Char { .. } = stmt {
            return Ok(());
        }

        if let ThrushStatement::Pass { .. } = stmt {
            return Ok(());
        }

        self.errors.push(ThrushCompilerIssue::Bug(
            String::from("Expression not caught"),
            String::from("The expression could not be caught for processing."),
            stmt.get_span(),
            CompilationPosition::TypeChecker,
            line!(),
        ));

        Ok(())
    }

    pub fn is_mismatch_type(
        &self,
        target_type: &ThrushType,
        from_type: &ThrushType,
        expression: Option<&ThrushStatement>,
        operator: Option<&TokenKind>,
        span: &Span,
    ) -> Result<(), ThrushCompilerIssue> {
        let error: ThrushCompilerIssue = ThrushCompilerIssue::Error(
            String::from("Mismatched types"),
            format!("Expected '{}' but found '{}'.", target_type, from_type),
            None,
            *span,
        );

        if let Some(ThrushStatement::BinaryOp {
            operator,
            kind: expression_type,
            ..
        }) = expression
        {
            return self.is_mismatch_type(target_type, expression_type, None, Some(operator), span);
        }

        if let Some(ThrushStatement::UnaryOp {
            operator,
            kind: expression_type,
            ..
        }) = expression
        {
            return self.is_mismatch_type(target_type, expression_type, None, Some(operator), span);
        }

        if let Some(ThrushStatement::Group {
            expression,
            kind: expression_type,
            ..
        }) = expression
        {
            return self.is_mismatch_type(
                target_type,
                expression_type,
                Some(expression),
                None,
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
                        self.is_mismatch_type(target_field, from_field, None, None, span)
                    },
                )?;

                Ok(())
            }

            (ThrushType::Me(_), ThrushType::Me(_), None) => Ok(()),

            (ThrushType::Me(_), ThrushType::Struct(_, _), None) => Ok(()),

            (ThrushType::Struct(_, _) | ThrushType::Me(_), ThrushType::Ptr(_), None) => Ok(()),

            (
                target_type,
                ThrushType::Mut(from_type),
                Some(
                    TokenKind::BangEq
                    | TokenKind::EqEq
                    | TokenKind::LessEq
                    | TokenKind::Less
                    | TokenKind::Greater
                    | TokenKind::GreaterEq
                    | TokenKind::And
                    | TokenKind::Or
                    | TokenKind::Bang
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift,
                )
                | None,
            ) if !target_type.is_mut_type() => {
                self.is_mismatch_type(target_type, from_type, expression, operator, span)?;

                Ok(())
            }

            (
                ThrushType::Mut(target_type),
                any_type,
                Some(
                    TokenKind::BangEq
                    | TokenKind::EqEq
                    | TokenKind::LessEq
                    | TokenKind::Less
                    | TokenKind::Greater
                    | TokenKind::GreaterEq
                    | TokenKind::And
                    | TokenKind::Or
                    | TokenKind::Bang
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift,
                )
                | None,
            ) if !any_type.is_mut_type() => {
                self.is_mismatch_type(target_type, any_type, expression, operator, span)?;

                Ok(())
            }

            (
                ThrushType::Mut(target_type),
                ThrushType::Mut(from_type),
                Some(
                    TokenKind::BangEq
                    | TokenKind::EqEq
                    | TokenKind::LessEq
                    | TokenKind::Less
                    | TokenKind::Greater
                    | TokenKind::GreaterEq
                    | TokenKind::And
                    | TokenKind::Or
                    | TokenKind::Bang
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift,
                )
                | None,
            ) => {
                self.is_mismatch_type(target_type, from_type, expression, operator, span)?;

                Ok(())
            }

            (ThrushType::Ptr(None), ThrushType::Ptr(None), None) => Ok(()),
            (ThrushType::Ptr(Some(target_type)), ThrushType::Ptr(Some(from_type)), None) => {
                self.is_mismatch_type(target_type, from_type, expression, operator, span)?;

                Ok(())
            }

            (
                ThrushType::Bool,
                ThrushType::Bool,
                Some(
                    TokenKind::BangEq
                    | TokenKind::EqEq
                    | TokenKind::LessEq
                    | TokenKind::Less
                    | TokenKind::Greater
                    | TokenKind::GreaterEq
                    | TokenKind::And
                    | TokenKind::Or
                    | TokenKind::Bang,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::S8,
                ThrushType::S8 | ThrushType::U8,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::S16,
                ThrushType::S16 | ThrushType::S8 | ThrushType::U16 | ThrushType::U8,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::S32,
                ThrushType::S32
                | ThrushType::S16
                | ThrushType::S8
                | ThrushType::U32
                | ThrushType::U16
                | ThrushType::U8,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::S64,
                ThrushType::S64
                | ThrushType::S32
                | ThrushType::S16
                | ThrushType::S8
                | ThrushType::U64
                | ThrushType::U32
                | ThrushType::U16
                | ThrushType::U8,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::U8,
                ThrushType::U8,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::U16,
                ThrushType::U16 | ThrushType::U8,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::U32,
                ThrushType::U32 | ThrushType::U16 | ThrushType::U8,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::U64,
                ThrushType::U64 | ThrushType::U32 | ThrushType::U16 | ThrushType::U8,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::F32,
                ThrushType::F32,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),
            (
                ThrushType::F64,
                ThrushType::F64 | ThrushType::F32,
                Some(
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Slash
                    | TokenKind::Star
                    | TokenKind::LShift
                    | TokenKind::RShift
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus,
                )
                | None,
            ) => Ok(()),

            _ => Err(error),
        }
    }

    fn check_binaryop(
        &self,
        operator: &TokenKind,
        a: &ThrushType,
        b: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        match operator {
            TokenKind::Star | TokenKind::Slash | TokenKind::Minus | TokenKind::Plus => {
                self.check_binary_arithmetic(operator, a, b, span)
            }
            TokenKind::BangEq | TokenKind::EqEq => self.check_binary_equality(operator, a, b, span),
            TokenKind::LessEq | TokenKind::Less | TokenKind::GreaterEq | TokenKind::Greater => {
                self.check_binary_comparasion(operator, a, b, span)
            }
            TokenKind::LShift | TokenKind::RShift => self.check_binary_shift(operator, a, b, span),
            TokenKind::And | TokenKind::Or => self.check_binary_gate(operator, a, b, span),
            _ => Ok(()),
        }
    }

    fn check_unary(
        &self,
        operator: &TokenKind,
        a: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        match operator {
            TokenKind::Minus | TokenKind::PlusPlus | TokenKind::MinusMinus => {
                self.check_general_unary(operator, a, span)
            }
            TokenKind::Bang => self.check_unary_instr_bang(a, span),
            _ => Ok(()),
        }
    }

    fn check_binary_arithmetic(
        &self,
        operator: &TokenKind,
        a: &ThrushType,
        b: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        match (a, b) {
            (
                ThrushType::S8
                | ThrushType::S16
                | ThrushType::S32
                | ThrushType::S64
                | ThrushType::U8
                | ThrushType::U16
                | ThrushType::U32
                | ThrushType::U64,
                ThrushType::S8
                | ThrushType::S16
                | ThrushType::S32
                | ThrushType::S64
                | ThrushType::U8
                | ThrushType::U16
                | ThrushType::U32
                | ThrushType::U64,
            ) => Ok(()),

            (ThrushType::F32 | ThrushType::F64, ThrushType::F32 | ThrushType::F64) => Ok(()),
            (ThrushType::Mut(a_subtype), ThrushType::Mut(b_subtype)) => {
                self.check_binary_arithmetic(operator, a_subtype, b_subtype, span)
            }
            (any, ThrushType::Mut(b_subtype)) => {
                self.check_binary_arithmetic(operator, any, b_subtype, span)
            }
            (ThrushType::Mut(a_subtype), any) => {
                self.check_binary_arithmetic(operator, a_subtype, any, span)
            }

            _ => Err(ThrushCompilerIssue::Error(
                String::from("Mismatched Types"),
                format!(
                    "Arithmetic operation ({} {} {}) is not allowed.",
                    a, operator, b
                ),
                None,
                span,
            )),
        }
    }

    fn check_binary_equality(
        &self,
        operator: &TokenKind,
        a: &ThrushType,
        b: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if matches!(
            (a, b),
            (
                ThrushType::S8
                    | ThrushType::S16
                    | ThrushType::S32
                    | ThrushType::S64
                    | ThrushType::U8
                    | ThrushType::U16
                    | ThrushType::U32
                    | ThrushType::U64,
                ThrushType::S8
                    | ThrushType::S16
                    | ThrushType::S32
                    | ThrushType::S64
                    | ThrushType::U8
                    | ThrushType::U16
                    | ThrushType::U32
                    | ThrushType::U64,
            ) | (
                ThrushType::F32 | ThrushType::F64,
                ThrushType::F32 | ThrushType::F64
            ) | (ThrushType::Bool, ThrushType::Bool)
                | (ThrushType::Char, ThrushType::Char)
        ) {
            return Ok(());
        }

        if a.is_ptr_type() && b.is_ptr_type() {
            return Ok(());
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Mismatched Types"),
            format!(
                "Logical operation ({} {} {}) is not allowed.",
                a, operator, b
            ),
            None,
            span,
        ))
    }

    fn check_binary_comparasion(
        &self,
        operator: &TokenKind,
        a: &ThrushType,
        b: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let (
            ThrushType::S8
            | ThrushType::S16
            | ThrushType::S32
            | ThrushType::S64
            | ThrushType::U8
            | ThrushType::U16
            | ThrushType::U32
            | ThrushType::U64,
            ThrushType::S8
            | ThrushType::S16
            | ThrushType::S32
            | ThrushType::S64
            | ThrushType::U8
            | ThrushType::U16
            | ThrushType::U32
            | ThrushType::U64,
        ) = (a, b)
        {
            return Ok(());
        } else if let (ThrushType::F32 | ThrushType::F64, ThrushType::F32 | ThrushType::F64) =
            (a, b)
        {
            return Ok(());
        } else if let (ThrushType::Mut(a_subtype), ThrushType::Mut(b_subtype)) = (a, b) {
            return self.check_binary_comparasion(operator, a_subtype, b_subtype, span);
        } else if let (ThrushType::Mut(a_subtype), any) = (a, b) {
            return self.check_binary_comparasion(operator, a_subtype, any, span);
        } else if let (any, ThrushType::Mut(b_subtype)) = (a, b) {
            return self.check_binary_comparasion(operator, any, b_subtype, span);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Mismatched Types"),
            format!(
                "Logical operation ({} {} {}) is not allowed.",
                a, operator, b
            ),
            None,
            span,
        ))
    }

    fn check_binary_gate(
        &self,
        operator: &TokenKind,
        a: &ThrushType,
        b: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let (ThrushType::Bool, ThrushType::Bool) = (a, b) {
            return Ok(());
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Mismatched Types"),
            format!(
                "Logical operation ({} {} {}) is not allowed.",
                a, operator, b
            ),
            None,
            span,
        ))
    }

    fn check_binary_shift(
        &self,
        operator: &TokenKind,
        a: &ThrushType,
        b: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let (
            ThrushType::S8
            | ThrushType::S16
            | ThrushType::S32
            | ThrushType::S64
            | ThrushType::U8
            | ThrushType::U16
            | ThrushType::U32
            | ThrushType::U64,
            ThrushType::S8
            | ThrushType::S16
            | ThrushType::S32
            | ThrushType::S64
            | ThrushType::U8
            | ThrushType::U16
            | ThrushType::U32
            | ThrushType::U64,
        ) = (a, b)
        {
            return Ok(());
        } else if let (ThrushType::Mut(a_subtype), ThrushType::Mut(b_subtype)) = (a, b) {
            return self.check_binary_shift(operator, a_subtype, b_subtype, span);
        } else if let (ThrushType::Mut(a_subtype), any) = (a, b) {
            return self.check_binary_shift(operator, a_subtype, any, span);
        } else if let (any, ThrushType::Mut(b_subtype)) = (a, b) {
            return self.check_binary_shift(operator, any, b_subtype, span);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Mismatched Types"),
            format!(
                "Arithmetic operation ({} {} {}) is not allowed.",
                a, operator, b
            ),
            None,
            span,
        ))
    }

    fn check_general_unary(
        &self,
        operator: &TokenKind,
        a: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if a.is_integer_type() || a.is_float_type() || a.is_mut_numeric_type() {
            return Ok(());
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Mismatched Types"),
            format!(
                "Arithmetic operation '{}' with '{}' is not allowed.",
                operator, a
            ),
            None,
            span,
        ))
    }

    fn check_unary_instr_bang(
        &self,
        a: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let ThrushType::Bool = a {
            return Ok(());
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Mismatched Types"),
            format!("Logical operation (!{}) is not allowed.", a),
            None,
            span,
        ))
    }

    pub fn init(&mut self) {
        self.stmts
            .iter()
            .filter(|stmt| stmt.is_function())
            .for_each(|stmt| {
                if let ThrushStatement::Function {
                    name,
                    parameter_types: types,
                    attributes,
                    ..
                } = stmt
                {
                    self.symbols
                        .new_function(name, (types, attributes.has_public_attribute()));
                }
            });

        self.stmts
            .iter()
            .filter(|stmt| stmt.is_methods())
            .for_each(|stmt| {
                if let ThrushStatement::Methods { name, binds, .. } = stmt {
                    let binds: Vec<(&'stmts str, &'stmts [ThrushType])> = binds
                        .iter()
                        .filter_map(|stmt| match stmt {
                            ThrushStatement::Method {
                                name,
                                parameters_types,
                                ..
                            } => Some((*name, parameters_types.as_slice())),

                            _ => None,
                        })
                        .collect();

                    self.symbols.new_methods(name, binds);
                }
            });
    }

    fn begin_scope(&mut self) {
        self.symbols.begin_scope();
    }

    fn end_scope(&mut self) {
        self.symbols.end_scope();
    }

    fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }

    fn advance(&mut self) {
        if !self.is_eof() {
            self.position += 1;
        }
    }

    fn peek(&self) -> &'stmts ThrushStatement<'stmts> {
        &self.stmts[self.position]
    }

    fn is_eof(&self) -> bool {
        self.position >= self.stmts.len()
    }
}
