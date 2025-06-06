#![allow(clippy::only_used_in_recursion)]

use marks::{TypeCheckerTypeCheckSource, TypeCheckerTypeContext, TypeCheckerTypePosition};
use table::TypeCheckerSymbolsTable;

use crate::{
    core::{
        compiler::options::CompilerFile,
        console::logging::LoggingType,
        diagnostic::diagnostician::Diagnostician,
        errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    },
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::ThrushAttributesExtensions},
        },
    },
};

mod marks;
mod table;

#[derive(Debug)]
pub struct TypeChecker<'type_checker> {
    stmts: &'type_checker [ThrushStatement<'type_checker>],
    position: usize,
    errors: Vec<ThrushCompilerIssue>,
    type_ctx: TypeCheckerTypeContext<'type_checker>,
    symbols: TypeCheckerSymbolsTable<'type_checker>,
    diagnostician: Diagnostician,
}

impl<'type_checker> TypeChecker<'type_checker> {
    pub fn new(
        stmts: &'type_checker [ThrushStatement<'type_checker>],
        file: &'type_checker CompilerFile,
    ) -> Self {
        Self {
            stmts,
            position: 0,
            errors: Vec::with_capacity(100),
            type_ctx: TypeCheckerTypeContext::new(),
            symbols: TypeCheckerSymbolsTable::new(),
            diagnostician: Diagnostician::new(file),
        }
    }

    pub fn check(&mut self) -> bool {
        self.init();

        while !self.is_eof() {
            let current_stmt: &ThrushStatement = self.peek();

            if let Err(type_error) = self.analyze_stmt(current_stmt) {
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

    fn analyze_stmt(
        &mut self,
        stmt: &'type_checker ThrushStatement,
    ) -> Result<(), ThrushCompilerIssue> {
        if let ThrushStatement::EntryPoint { body, .. } = stmt {
            if let Err(type_error) = self.analyze_stmt(body) {
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
            if let Err(type_error) = self.analyze_stmt(local) {
                self.add_error(type_error);
            }

            if let Err(type_error) = self.analyze_stmt(cond) {
                self.add_error(type_error);
            }

            if let Err(type_error) = self.analyze_stmt(actions) {
                self.add_error(type_error);
            }

            if let Err(type_error) = self.analyze_stmt(block) {
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

            if let Err(mismatch_type_error) = self.validate_types(
                target_type,
                from_type,
                Some(value),
                None,
                span,
                TypeCheckerTypeCheckSource::default(),
            ) {
                self.add_error(mismatch_type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::AssemblerFunction {
            parameters, span, ..
        } = stmt
        {
            for parameter in parameters.iter() {
                if self.check_mismatch_type(&ThrushType::Void, parameter.get_value_type()?) {
                    self.add_error(ThrushCompilerIssue::Error(
                        String::from("Type not allowed"),
                        String::from("The void type isn't valid a runtime value."),
                        None,
                        *span,
                    ));
                }
            }

            return Ok(());
        }

        if let ThrushStatement::Function {
            parameters,
            body,
            return_type,
            span,
            ..
        } = stmt
        {
            self.type_ctx
                .set_type_position(TypeCheckerTypePosition::Function);
            self.type_ctx.set_function_type(return_type);

            for parameter in parameters.iter() {
                if self.check_mismatch_type(&ThrushType::Void, parameter.get_value_type()?) {
                    self.add_error(ThrushCompilerIssue::Error(
                        String::from("Type not allowed"),
                        String::from("The void type isn't valid a runtime value."),
                        None,
                        *span,
                    ));
                }
            }

            if body.is_block() {
                if let Err(type_error) = self.analyze_stmt(body) {
                    self.add_error(type_error);
                }

                if !body.has_return() {
                    if let Err(mismatch_type_error) = self.validate_types(
                        return_type,
                        &ThrushType::Void,
                        None,
                        None,
                        span,
                        TypeCheckerTypeCheckSource::default(),
                    ) {
                        self.add_error(mismatch_type_error);
                    }
                }
            }

            self.type_ctx
                .set_type_position(TypeCheckerTypePosition::None);

            self.type_ctx.set_function_type(&ThrushType::Void);

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

            if self.check_mismatch_type(&ThrushType::Void, local_type) {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "The void type isn't valid value.".into(),
                    None,
                    *span,
                ));
            }
            let local_value_type: &ThrushType = local_value.get_value_type()?;

            if let Err(mismatch_type_error) = self.validate_types(
                local_type,
                local_value_type,
                Some(local_value),
                None,
                span,
                TypeCheckerTypeCheckSource::Local,
            ) {
                self.add_error(mismatch_type_error);
            }

            if let Err(type_error) = self.analyze_stmt(local_value) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::LLI {
            name,
            kind: lli_type,
            value: lli_value,
            span,
            ..
        } = stmt
        {
            self.symbols.new_lli(name, (lli_type, *span));

            let lli_value_type: &ThrushType = lli_value.get_value_type()?;

            if self.check_mismatch_type(&ThrushType::Void, lli_type) {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "The void type isn't valid value.".into(),
                    None,
                    *span,
                ));
            }

            if !lli_value_type.is_ptr_type() && !lli_value_type.is_address_type() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected always 'ptr<T>' or addr type.".into(),
                    None,
                    *span,
                ));
            }

            if let Err(mismatch_type_error) = self.validate_types(
                lli_type,
                lli_value_type,
                Some(lli_value),
                None,
                span,
                TypeCheckerTypeCheckSource::LLI,
            ) {
                self.add_error(mismatch_type_error);
            }

            if let Err(type_error) = self.analyze_stmt(lli_value) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::If {
            cond, elfs, span, ..
        } = stmt
        {
            if let Err(error) = self.validate_types(
                &ThrushType::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                span,
                TypeCheckerTypeCheckSource::default(),
            ) {
                self.add_error(error);
            }

            for elif in elfs.iter() {
                if let ThrushStatement::Elif { cond, span, .. } = elif {
                    if let Err(error) = self.validate_types(
                        &ThrushType::Bool,
                        cond.get_value_type()?,
                        Some(cond),
                        None,
                        span,
                        TypeCheckerTypeCheckSource::default(),
                    ) {
                        self.add_error(error);
                    }
                }
            }

            return Ok(());
        }

        if let ThrushStatement::Return {
            expression,
            kind,
            span,
        } = stmt
        {
            if let Err(error) = self.validate_types(
                self.type_ctx.get_function_type(),
                kind,
                expression.as_deref(),
                None,
                span,
                TypeCheckerTypeCheckSource::default(),
            ) {
                self.add_error(error);
            }

            if let Some(expr) = expression {
                self.analyze_stmt(expr)?;
            }

            return Ok(());
        }

        /* ######################################################################


            TYPE CHECKER EXPRESSIONS - START


        ########################################################################*/

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

            if let Err(type_error) = self.analyze_stmt(left) {
                self.add_error(type_error);
            }

            if let Err(type_error) = self.analyze_stmt(right) {
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

            if let Err(type_error) = self.analyze_stmt(expression) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::Group { expression, .. } = stmt {
            if let Err(type_error) = self.analyze_stmt(expression) {
                self.add_error(type_error);
            }

            return Ok(());
        }

        if let ThrushStatement::Call {
            name, args, span, ..
        } = stmt
        {
            if let Some(function) = self.symbols.get_function(name) {
                return self.validate_call(*function, args, span);
            }

            if let Some(asm_function) = self.symbols.get_asm_function(name) {
                return self.validate_call(*asm_function, args, span);
            }

            self.errors.push(ThrushCompilerIssue::Bug(
                String::from("Call not caught"),
                format!("Could not get named any function '{}'.", name),
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

                    for (target_type, expr) in types.iter().zip(args.iter()) {
                        let from_type: &ThrushType = expr.get_value_type()?;
                        let span: Span = expr.get_span();

                        if let Err(error) = self.validate_types(
                            target_type,
                            from_type,
                            Some(expr),
                            None,
                            &span,
                            TypeCheckerTypeCheckSource::Call,
                        ) {
                            self.add_error(error);
                        }

                        self.analyze_stmt(expr)?;
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
                    if let Err(error) = self.validate_types(
                        local_type,
                        value.get_value_type()?,
                        Some(value),
                        None,
                        span,
                        TypeCheckerTypeCheckSource::default(),
                    ) {
                        self.add_error(error);
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
                if let Err(error) = self.validate_types(
                    expression.get_value_type()?,
                    value.get_value_type()?,
                    Some(value),
                    None,
                    span,
                    TypeCheckerTypeCheckSource::default(),
                ) {
                    self.add_error(error);
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

        if let ThrushStatement::Constructor { arguments, .. } = stmt {
            let args: &[(&str, ThrushStatement<'_>, ThrushType, u32)] = &arguments.1;

            for arg in args.iter() {
                let expression: &ThrushStatement = &arg.1;
                let expression_span: Span = expression.get_span();

                let target_type: &ThrushType = &arg.2;
                let from_type: &ThrushType = expression.get_value_type()?;

                if let Err(error) = self.validate_types(
                    target_type,
                    from_type,
                    Some(expression),
                    None,
                    &expression_span,
                    TypeCheckerTypeCheckSource::default(),
                ) {
                    self.add_error(error);
                }
            }

            return Ok(());
        }

        if let ThrushStatement::Block { stmts, .. } = stmt {
            self.begin_scope();

            stmts.iter().for_each(|stmt| {
                if let Err(type_error) = self.analyze_stmt(stmt) {
                    self.add_error(type_error);
                }
            });

            self.end_scope();

            return Ok(());
        }

        if let ThrushStatement::Address {
            name,
            indexes,
            span,
            ..
        } = stmt
        {
            if let Some(lli) = self.symbols.get_lli(name) {
                let any_type: &ThrushType = lli.0;

                if !any_type.is_ptr_type() && !any_type.is_address_type() {
                    self.add_error(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected 'ptr<T>' or addr type.".into(),
                        None,
                        *span,
                    ));
                }
            }

            for indexe in indexes {
                if !indexe.is_unsigned_integer()? || !indexe.is_anyu32bit_integer()? {
                    self.add_error(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected any unsigned integer less than or equal to 32 bits.".into(),
                        None,
                        *span,
                    ));
                }
            }

            return Ok(());
        }

        if let ThrushStatement::Load { load, .. } = stmt {
            if let Some(name) = load.0 {
                if let Some(any) = self.symbols.get_lli(name) {
                    let any_type: &ThrushType = any.0;
                    let any_span: Span = any.1;

                    if !any_type.is_ptr_type() && !any_type.is_address_type() {
                        self.add_error(ThrushCompilerIssue::Error(
                            "Syntax error".into(),
                            "Expected 'mut ptr<T>', ptr<T> or 'addr' type.".into(),
                            None,
                            any_span,
                        ));
                    }
                }
            }

            if let Some(any) = &load.1 {
                let any_type: &ThrushType = any.get_value_type()?;
                let any_span: Span = any.get_span();

                if !any_type.is_ptr_type() && !any_type.is_address_type() {
                    self.add_error(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected ' ptr<T> or 'addr' type.".into(),
                        None,
                        any_span,
                    ));
                }
            }

            return Ok(());
        }

        if let ThrushStatement::Deref { load, .. } = stmt {
            let load_type: &ThrushType = load.get_value_type()?;
            let load_span: Span = load.get_span();

            if !load_type.is_ptr_type() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected 'ptr<T>' || 'ptr' type.".into(),
                    None,
                    load_span,
                ));
            }

            self.analyze_stmt(load)?;

            return Ok(());
        }

        if let ThrushStatement::CastRaw { from, cast, span } = stmt {
            let from_type: &ThrushType = from.get_value_type()?;
            let from_span: Span = from.get_span();

            if !from_type.is_ptr_type() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected 'ptr<T>' type.".into(),
                    None,
                    from_span,
                ));
            }

            if cast.is_ptr_type() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "A non-raw type 'ptr<T>' was expected.".into(),
                    None,
                    from_span,
                ));
            }

            if !from_type.match_first_depth(cast) {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    format!("Cannot cast '{}' to '{}'.", from_type, cast),
                    None,
                    *span,
                ));
            }

            self.analyze_stmt(from)?;

            return Ok(());
        }

        if let ThrushStatement::CastPtr { from, cast, span } = stmt {
            let from_type: &ThrushType = from.get_value_type()?;

            if !from_type.is_ptr_type() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected 'ptr<T>' type.".into(),
                    None,
                    *span,
                ));
            }

            if !cast.is_ptr_type() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected 'ptr<T>' type.".into(),
                    None,
                    *span,
                ));
            }

            self.analyze_stmt(from)?;

            return Ok(());
        }

        if let ThrushStatement::Cast { from, cast, span } = stmt {
            let from_type: &ThrushType = from.get_value_type()?;

            if let Err(error) = self.validate_type_cast(cast, from_type, span) {
                self.add_error(error);
            }

            return Ok(());
        }

        if let ThrushStatement::Write {
            write_value,
            write_type,
            ..
        } = stmt
        {
            let write_value_type: &ThrushType = write_value.get_value_type()?;
            let write_value_span: Span = write_value.get_span();

            if let Err(error) = self.validate_types(
                write_type,
                write_value_type,
                Some(write_value),
                None,
                &write_value_span,
                TypeCheckerTypeCheckSource::default(),
            ) {
                self.add_error(error);
            }

            return Ok(());
        }

        if let ThrushStatement::Alloc { .. } | ThrushStatement::RawPtr { .. } = stmt {
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

        /* ######################################################################


            TYPE CHECKER EXPRESSIONS - END


        ########################################################################*/

        self.errors.push(ThrushCompilerIssue::Bug(
            String::from("Expression not caught"),
            String::from("The expression could not be caught for processing."),
            stmt.get_span(),
            CompilationPosition::TypeChecker,
            line!(),
        ));

        Ok(())
    }

    fn validate_type_cast(
        &self,
        cast_type: &ThrushType,
        from_type: &ThrushType,
        span: &Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if (cast_type.is_integer_type() && from_type.is_integer_type())
            || (cast_type.is_float_type() && from_type.is_float_type())
            || (cast_type.is_bool_type() && from_type.is_bool_type())
        {
            Ok(())
        } else {
            Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                format!("Cannot cast '{}' to '{}'", from_type, cast_type),
                None,
                *span,
            ))
        }
    }

    fn validate_call(
        &mut self,
        data: (&[ThrushType], bool),
        args: &'type_checker [ThrushStatement],
        span: &Span,
    ) -> Result<(), ThrushCompilerIssue> {
        let (parameter_types, ignore_more_arguments) = data;

        let parameter_types_size: usize = parameter_types.len();
        let mut parameter_types_displayed: String = String::with_capacity(100);

        parameter_types.iter().for_each(|parameter_type| {
            parameter_types_displayed.push_str(&format!("{} ", parameter_type));
        });

        if args.len() != parameter_types_size && !ignore_more_arguments {
            self.add_error(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                format!(
                    "Expected {} arguments with types '{}', got {}.",
                    parameter_types_size,
                    parameter_types_displayed,
                    args.len()
                ),
                None,
                *span,
            ));
            return Ok(());
        }

        for (target_type, expr) in parameter_types.iter().zip(args.iter()) {
            let from_type: &ThrushType = expr.get_value_type()?;
            let expr_span: Span = expr.get_span();

            if let Err(error) = self.validate_types(
                target_type,
                from_type,
                Some(expr),
                None,
                &expr_span,
                TypeCheckerTypeCheckSource::Call,
            ) {
                self.add_error(error);
            }

            self.analyze_stmt(expr)?;
        }

        Ok(())
    }

    pub fn validate_types(
        &self,
        target_type: &ThrushType,
        from_type: &ThrushType,
        expression: Option<&ThrushStatement>,
        operator: Option<&TokenType>,
        span: &Span,
        source: TypeCheckerTypeCheckSource,
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
            return self.validate_types(
                target_type,
                expression_type,
                None,
                Some(operator),
                span,
                source,
            );
        }

        if let Some(ThrushStatement::UnaryOp {
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
                span,
                source,
            );
        }

        if let Some(ThrushStatement::Group {
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
                span,
                source,
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
                        self.validate_types(target_field, from_field, None, None, span, source)
                    },
                )?;

                Ok(())
            }

            (ThrushType::Struct(_, _), ThrushType::Ptr(_), None) => Ok(()),
            (ThrushType::Addr, ThrushType::Addr, None) => Ok(()),

            (
                target_type,
                ThrushType::Mut(from_type),
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
            ) if !target_type.is_mut_type() && source.is_local() => {
                self.validate_types(target_type, from_type, expression, operator, span, source)?;

                Ok(())
            }

            (
                ThrushType::Mut(target_type),
                any_type,
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
            ) if !any_type.is_mut_type() && source.is_local() => {
                self.validate_types(target_type, any_type, expression, operator, span, source)?;

                Ok(())
            }

            (
                ThrushType::Mut(target_type),
                ThrushType::Mut(from_type),
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
            ) => {
                self.validate_types(target_type, from_type, expression, operator, span, source)?;

                Ok(())
            }

            (ThrushType::Ptr(None), ThrushType::Ptr(None), None) => Ok(()),
            (ThrushType::Ptr(Some(target_type)), ThrushType::Ptr(Some(from_type)), None) => {
                self.validate_types(target_type, from_type, expression, operator, span, source)?;

                Ok(())
            }

            (ThrushType::Ptr(any), other, None) if source.is_lli() => {
                if let Some(ptr_sub_type) = any {
                    if **ptr_sub_type == *other {
                        return Ok(());
                    }

                    self.validate_types(ptr_sub_type, other, expression, operator, span, source)?;
                } else {
                    return Err(error);
                }

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
                ThrushType::S8 | ThrushType::U8,
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
                ThrushType::S16 | ThrushType::S8 | ThrushType::U16 | ThrushType::U8,
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
                ThrushType::S32
                | ThrushType::S16
                | ThrushType::S8
                | ThrushType::U32
                | ThrushType::U16
                | ThrushType::U8,
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
                ThrushType::S64
                | ThrushType::S32
                | ThrushType::S16
                | ThrushType::S8
                | ThrushType::U64
                | ThrushType::U32
                | ThrushType::U16
                | ThrushType::U8,
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

    fn check_mismatch_type(&self, target_type: &ThrushType, from_type: &ThrushType) -> bool {
        target_type == from_type
    }

    fn check_binaryop(
        &self,
        operator: &TokenType,
        a: &ThrushType,
        b: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        match operator {
            TokenType::Star | TokenType::Slash | TokenType::Minus | TokenType::Plus => {
                self.check_binary_arithmetic(operator, a, b, span)
            }
            TokenType::BangEq | TokenType::EqEq => self.check_binary_equality(operator, a, b, span),
            TokenType::LessEq | TokenType::Less | TokenType::GreaterEq | TokenType::Greater => {
                self.check_binary_comparasion(operator, a, b, span)
            }
            TokenType::LShift | TokenType::RShift => self.check_binary_shift(operator, a, b, span),
            TokenType::And | TokenType::Or => self.check_binary_gate(operator, a, b, span),
            _ => Ok(()),
        }
    }

    fn check_unary(
        &self,
        operator: &TokenType,
        a: &ThrushType,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        match operator {
            TokenType::Minus | TokenType::PlusPlus | TokenType::MinusMinus => {
                self.check_general_unary(operator, a, span)
            }
            TokenType::Bang => self.check_unary_instr_bang(a, span),
            _ => Ok(()),
        }
    }

    fn check_binary_arithmetic(
        &self,
        operator: &TokenType,
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
        operator: &TokenType,
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
        operator: &TokenType,
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
        operator: &TokenType,
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
        operator: &TokenType,
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
        operator: &TokenType,
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
            .filter(|stmt| stmt.is_asm_function())
            .for_each(|stmt| {
                if let ThrushStatement::AssemblerFunction {
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
                if let ThrushStatement::Methods { name, methods, .. } = stmt {
                    let methods: Vec<(&str, &[ThrushType])> = methods
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

                    self.symbols.new_methods(name, methods);
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

    fn peek(&self) -> &'type_checker ThrushStatement<'type_checker> {
        &self.stmts[self.position]
    }

    fn is_eof(&self) -> bool {
        self.position >= self.stmts.len()
    }
}
