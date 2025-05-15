use crate::{
    middle::types::frontend::{
        lexer::{
            tokenkind::TokenKind,
            types::{self, ThrushType},
        },
        parser::{
            stmts::{
                instruction::Instruction,
                traits::{
                    ConstructorExtensions, EnumExtensions, EnumFieldsExtensions, FoundSymbolEither,
                    FoundSymbolExtension, StructExtensions, TokenExtensions,
                },
                types::{Constructor, EnumField, EnumFields},
            },
            symbols::{
                traits::{
                    BindExtensions, BindingsExtensions, ConstantExtensions, FunctionExtensions,
                    LocalExtensions,
                },
                types::{
                    Bind, Bindings, Constant, FoundSymbolId, Function, Local, Parameters, Struct,
                },
            },
        },
    },
    standard::error::ThrushCompilerIssue,
};

use super::{
    contexts::SyncPosition,
    lexer::{Span, Token},
    parser::ParserContext,
    typecheck, typegen, utils,
};

pub fn build_expression<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Expression);

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            parser_ctx.peek().span,
        ));
    }

    let instruction: Instruction = or(parser_ctx)?;

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(instruction)
}

pub fn build_expr<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Expression);

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            parser_ctx.peek().span,
        ));
    }

    let instruction: Instruction = or(parser_ctx)?;

    Ok(instruction)
}

fn or<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let mut expression: Instruction = and(parser_ctx)?;

    while parser_ctx.match_token(TokenKind::Or)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Instruction = and(parser_ctx)?;

        typecheck::check_binaryop(&operator, expression.get_type(), right.get_type(), span)?;

        expression = Instruction::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: ThrushType::Bool,
            span,
        }
    }

    Ok(expression)
}

fn and<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let mut expression: Instruction = equality(parser_ctx)?;

    while parser_ctx.match_token(TokenKind::And)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Instruction = equality(parser_ctx)?;

        typecheck::check_binaryop(&operator, expression.get_type(), right.get_type(), span)?;

        expression = Instruction::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: ThrushType::Bool,
            span,
        }
    }

    Ok(expression)
}

fn equality<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let mut expression: Instruction = comparison(parser_ctx)?;

    if parser_ctx.match_token(TokenKind::BangEq)? || parser_ctx.match_token(TokenKind::EqEq)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Instruction = comparison(parser_ctx)?;

        typecheck::check_binaryop(&operator, expression.get_type(), right.get_type(), span)?;

        expression = Instruction::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: ThrushType::Bool,
            span,
        }
    }

    Ok(expression)
}

fn comparison<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let mut expression: Instruction = term(parser_ctx)?;

    if parser_ctx.match_token(TokenKind::Greater)?
        || parser_ctx.match_token(TokenKind::GreaterEq)?
        || parser_ctx.match_token(TokenKind::Less)?
        || parser_ctx.match_token(TokenKind::LessEq)?
    {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Instruction = term(parser_ctx)?;

        typecheck::check_binaryop(&operator, expression.get_type(), right.get_type(), span)?;

        expression = Instruction::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: ThrushType::Bool,
            span,
        };
    }

    Ok(expression)
}

fn term<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let mut expression: Instruction = factor(parser_ctx)?;

    while parser_ctx.match_token(TokenKind::Plus)?
        || parser_ctx.match_token(TokenKind::Minus)?
        || parser_ctx.match_token(TokenKind::LShift)?
        || parser_ctx.match_token(TokenKind::RShift)?
    {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Instruction = factor(parser_ctx)?;

        let left_type: &ThrushType = expression.get_type();
        let right_type: &ThrushType = right.get_type();

        typecheck::check_binaryop(&operator, left_type, right_type, span)?;

        let kind: &ThrushType = left_type.precompute_type(right_type);

        expression = Instruction::BinaryOp {
            left: expression.clone().into(),
            operator,
            right: right.into(),
            kind: kind.clone(),
            span,
        };
    }

    Ok(expression)
}

fn factor<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let mut expression: Instruction = unary(parser_ctx)?;

    while parser_ctx.match_token(TokenKind::Slash)? || parser_ctx.match_token(TokenKind::Star)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Instruction = unary(parser_ctx)?;

        let left_type: &ThrushType = expression.get_type();
        let right_type: &ThrushType = right.get_type();

        typecheck::check_binaryop(&operator, left_type, right_type, parser_ctx.previous().span)?;

        let kind: &ThrushType = left_type.precompute_type(right_type);

        expression = Instruction::BinaryOp {
            left: expression.clone().into(),
            operator,
            right: right.into(),
            kind: kind.clone(),
            span,
        };
    }

    Ok(expression)
}

fn unary<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    if parser_ctx.match_token(TokenKind::Bang)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let expression: Instruction = primary(parser_ctx)?;

        typecheck::check_unary(&operator, expression.get_type(), parser_ctx.previous().span)?;

        return Ok(Instruction::UnaryOp {
            operator,
            expression: expression.into(),
            kind: ThrushType::Bool,
            is_pre: false,
            span,
        });
    }

    if parser_ctx.match_token(TokenKind::Minus)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let mut expression: Instruction = primary(parser_ctx)?;

        expression.cast_signess(operator);

        let expression_type: &ThrushType = expression.get_type();

        typecheck::check_unary(&operator, expression_type, parser_ctx.previous().span)?;

        return Ok(Instruction::UnaryOp {
            operator,
            expression: expression.clone().into(),
            kind: expression_type.clone(),
            is_pre: false,
            span,
        });
    }

    let instr: Instruction = primary(parser_ctx)?;

    Ok(instr)
}

fn primary<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let primary: Instruction = match &parser_ctx.peek().kind {
        TokenKind::Carry => {
            let carry_tk: &Token = parser_ctx.advance()?;
            let span: Span = carry_tk.span;

            parser_ctx.consume(
                TokenKind::LBracket,
                String::from("Syntax error"),
                String::from("Expected '['."),
            )?;

            let carry_type: ThrushType = typegen::build_type(parser_ctx)?;

            parser_ctx.consume(
                TokenKind::RBracket,
                String::from("Syntax error"),
                String::from("Expected ']'."),
            )?;

            if parser_ctx.check(TokenKind::Identifier) {
                let identifier_tk: &Token = parser_ctx.consume(
                    TokenKind::Identifier,
                    String::from("Syntax error"),
                    String::from("Expected 'identifier'."),
                )?;

                let name: &str = identifier_tk.lexeme;

                build_ref(parser_ctx, name, span)?;

                return Ok(Instruction::Carry {
                    name,
                    expression: None,
                    carry_type,
                    span,
                });
            }

            let expression: Instruction = build_expr(parser_ctx)?;

            let expression_type: &ThrushType = expression.get_type();

            if !expression_type.is_ptr_type() && !expression_type.is_address_type() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Attemping to access an invalid pointer"),
                    format!(
                        "Carry is only allowed for pointer types or memory address, not '{}'. ",
                        expression_type
                    ),
                    None,
                    span,
                ));
            }

            Instruction::Carry {
                name: "",
                expression: Some(expression.into()),
                carry_type,
                span,
            }
        }

        TokenKind::Write => {
            let write_tk: &Token = parser_ctx.advance()?;
            let span: Span = write_tk.span;

            parser_ctx.consume(
                TokenKind::LBracket,
                String::from("Syntax error"),
                String::from("Expected '['."),
            )?;

            let write_type: ThrushType = typegen::build_type(parser_ctx)?;

            parser_ctx.consume(
                TokenKind::RBracket,
                String::from("Syntax error"),
                String::from("Expected ']'."),
            )?;

            let value: Instruction = build_expr(parser_ctx)?;

            parser_ctx.mismatch_types(
                &write_type,
                value.get_type(),
                value.get_span(),
                Some(&value),
            );

            parser_ctx.consume(
                TokenKind::Arrow,
                String::from("Syntax error"),
                String::from("Expected '->'."),
            )?;

            if parser_ctx.check(TokenKind::Identifier) {
                let identifier_tk: &Token = parser_ctx.consume(
                    TokenKind::Identifier,
                    String::from("Syntax error"),
                    String::from("Expected 'identifier'."),
                )?;

                let name: &str = identifier_tk.lexeme;

                build_ref(parser_ctx, name, span)?;

                return Ok(Instruction::Write {
                    write_to: (name, None),
                    write_value: value.into(),
                    write_type,
                    span,
                });
            }

            let expression: Instruction = build_expr(parser_ctx)?;

            let expression_type: &ThrushType = expression.get_type();

            if !expression_type.is_ptr_type() && !expression_type.is_address_type() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Attemping to access an invalid pointer"),
                    format!(
                        "Write is only allowed for pointer types or memory address, not '{}'. ",
                        expression_type
                    ),
                    None,
                    span,
                ));
            }

            Instruction::Write {
                write_to: ("", Some(expression.into())),
                write_value: value.into(),
                write_type,
                span,
            }
        }

        TokenKind::Address => {
            parser_ctx.only_advance()?;

            let identifier_tk: &Token = parser_ctx.consume(
                TokenKind::Identifier,
                String::from("Syntax error"),
                String::from("Expected 'identifier'."),
            )?;

            let name: &str = identifier_tk.lexeme;
            let span: Span = identifier_tk.span;

            parser_ctx.consume(
                TokenKind::LBracket,
                String::from("Syntax error"),
                String::from("Expected '['."),
            )?;

            return build_address(parser_ctx, name, span);
        }

        TokenKind::PlusPlus => {
            let operator_tk: &Token = parser_ctx.advance()?;
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let expression: Instruction = build_expr(parser_ctx)?;

            if !expression.is_local_ref() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Only local references can be pre-incremented."),
                    None,
                    parser_ctx.previous().span,
                ));
            }

            let unaryop: Instruction = Instruction::UnaryOp {
                operator,
                expression: expression.into(),
                kind: ThrushType::Void,
                is_pre: true,
                span,
            };

            return Ok(unaryop);
        }

        TokenKind::MinusMinus => {
            let operator_tk: &Token = parser_ctx.advance()?;
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let expression: Instruction = build_expr(parser_ctx)?;

            if !expression.is_local_ref() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Only local references can be pre-decremented."),
                    None,
                    parser_ctx.previous().span,
                ));
            }

            let unaryop: Instruction = Instruction::UnaryOp {
                operator,
                expression: expression.into(),
                kind: ThrushType::Void,
                is_pre: true,
                span,
            };

            return Ok(unaryop);
        }

        TokenKind::LParen => {
            let span: Span = parser_ctx.advance()?.span;

            let expression: Instruction = build_expr(parser_ctx)?;

            let kind: &ThrushType = expression.get_type();

            if !expression.is_binary() && !expression.is_group() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from(
                        "Grouping '(...)' is only allowed with binary expressions or other grouped expressions.",
                    ),
                    None,
                    span,
                ));
            }

            parser_ctx.consume(
                TokenKind::RParen,
                String::from("Syntax error"),
                String::from("Expected ')'."),
            )?;

            return Ok(Instruction::Group {
                expression: expression.clone().into(),
                kind: kind.clone(),
                span,
            });
        }

        TokenKind::Str => {
            let str_tk: &Token = parser_ctx.advance()?;
            let lexeme: &str = str_tk.lexeme;
            let span: Span = str_tk.span;

            Instruction::Str(ThrushType::Str, lexeme.parse_scapes(span)?, span)
        }

        TokenKind::Char => {
            let char_tk: &Token = parser_ctx.advance()?;
            let span: Span = char_tk.span;
            let lexeme: &str = char_tk.lexeme;

            Instruction::Char(ThrushType::Char, lexeme.get_first_byte(), span)
        }

        TokenKind::NullPtr => Instruction::NullPtr {
            span: parser_ctx.advance()?.span,
        },

        TokenKind::Integer => {
            let integer_tk: &Token = parser_ctx.advance()?;
            let integer: &str = integer_tk.lexeme;
            let span: Span = integer_tk.span;

            let parsed_integer: (ThrushType, f64) = utils::parse_number(integer, span)?;

            let integer_type: ThrushType = parsed_integer.0;
            let integer_value: f64 = parsed_integer.1;

            Instruction::Integer(integer_type, integer_value, false, span)
        }

        TokenKind::Float => {
            let float_tk: &Token = parser_ctx.advance()?;
            let float: &str = float_tk.lexeme;
            let span: Span = float_tk.span;

            let parsed_float: (ThrushType, f64) = utils::parse_number(float, span)?;

            let float_type: ThrushType = parsed_float.0;
            let float_value: f64 = parsed_float.1;

            Instruction::Float(float_type, float_value, false, span)
        }

        TokenKind::Identifier => {
            let identifier_tk: &Token = parser_ctx.advance()?;

            let name: &str = identifier_tk.lexeme;
            let span: Span = identifier_tk.span;

            let symbol: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

            if parser_ctx.match_token(TokenKind::Eq)? {
                let object: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

                if object.is_constant() {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Constants cannot be modified."),
                        None,
                        span,
                    ));
                }

                let local_position: (&str, usize) = object.expected_local(span)?;

                let local: &Local = parser_ctx.get_symbols().get_local_by_id(
                    local_position.0,
                    local_position.1,
                    span,
                )?;

                let local_span: Span = local.get_span();

                let local_type: ThrushType = local.0.clone();

                if !local.is_mutable() {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Expected mutable reference"),
                        String::from("Make mutable with 'mut' keyword before the identifier."),
                        None,
                        local_span,
                    ));
                }

                let expression: Instruction = build_expr(parser_ctx)?;

                parser_ctx.mismatch_types(
                    &local_type.clone(),
                    expression.get_type(),
                    span,
                    Some(&expression),
                );

                return Ok(Instruction::LocalMut {
                    source: (Some(name), None),
                    target: expression.into(),
                    kind: local_type,
                    span,
                });
            }

            if parser_ctx.match_token(TokenKind::Arrow)? {
                return build_enum_field(parser_ctx, name, span);
            }

            if parser_ctx.match_token(TokenKind::LParen)? {
                return build_function_call(parser_ctx, name, span);
            }

            if parser_ctx.match_token(TokenKind::Dot)? {
                let property: Instruction = build_property(parser_ctx, name, span)?;

                if parser_ctx.match_token(TokenKind::Eq)? {
                    let expr: Instruction = build_expr(parser_ctx)?;

                    parser_ctx.mismatch_types(
                        property.get_type(),
                        expr.get_type(),
                        expr.get_span(),
                        Some(&expr),
                    );

                    if !property.is_mutable() {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Expected mutable type"),
                            String::from("Make mutable the parameter or local of this property."),
                            None,
                            property.get_span(),
                        ));
                    }

                    return Ok(Instruction::LocalMut {
                        source: (None, Some(property.clone().into())),
                        target: expr.into(),
                        kind: property.get_type().clone(),
                        span,
                    });
                }

                return Ok(property);
            }

            if parser_ctx.match_token(TokenKind::ColonColon)? {
                return build_binding_call(parser_ctx, name, span);
            }

            if symbol.is_enum() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Invalid type"),
                    String::from(
                        "Enums cannot be used as types; use properties instead with their types.",
                    ),
                    None,
                    span,
                ));
            }

            if symbol.is_function() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Invalid type"),
                    String::from("Functions cannot be used as types; call it instead."),
                    None,
                    span,
                ));
            }

            build_ref(parser_ctx, name, span)?
        }

        TokenKind::True => Instruction::Boolean(ThrushType::Bool, true, parser_ctx.advance()?.span),
        TokenKind::False => {
            Instruction::Boolean(ThrushType::Bool, false, parser_ctx.advance()?.span)
        }

        TokenKind::This => build_this(parser_ctx)?,
        TokenKind::New => build_constructor(parser_ctx)?,

        _ => {
            let previous: &Token = parser_ctx.advance()?;

            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                format!("Statement '{}' don't allowed.", previous.lexeme),
                None,
                previous.span,
            ));
        }
    };

    Ok(primary)
}

fn build_binding_call<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    let structure_id: &str = symbol.expected_struct(span)?;

    let structure: Struct = parser_ctx
        .get_symbols()
        .get_struct_by_id(structure_id, span)?;

    let bind_tk: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected bind name."),
    )?;

    let bind_name: &str = bind_tk.lexeme;

    let bindings: Bindings = structure.get_bindings();

    if !bindings.contains_binding(bind_name) {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Not found '{}' bind inside the bindings of '{}' struct.",
                bind_name, name
            ),
            None,
            span,
        ));
    }

    let bind: Bind = bindings.get_bind(bind_name);
    let bind_name: &str = bind.get_name();
    let bind_type: ThrushType = bind.get_type();
    let bind_parameters_type: &[ThrushType] = bind.get_parameters_types();

    parser_ctx.consume(
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut args: Vec<Instruction> = Vec::with_capacity(10);

    while parser_ctx.peek().kind != TokenKind::RParen {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        let expression: Instruction = build_expr(parser_ctx)?;

        args.push(expression);
    }

    parser_ctx.consume(
        TokenKind::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    if args.len() != bind_parameters_type.len() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Expected {} arguments, not {}.",
                bind_parameters_type.len(),
                args.len()
            ),
            None,
            span,
        ));
    }

    for (index, argument) in args.iter().enumerate() {
        let target_type: &ThrushType = &bind_parameters_type[index];
        let from_type: &ThrushType = argument.get_type();

        parser_ctx.mismatch_types(target_type, from_type, argument.get_span(), Some(argument));
    }

    let canonical_name: String = format!("{}.{}", name, bind_name);

    Ok(Instruction::BindCall {
        name: canonical_name,
        args,
        kind: bind_type,
        span,
    })
}

fn build_property<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    let local_position: (&str, usize) = symbol.expected_local(span)?;

    let local: &Local =
        parser_ctx
            .get_symbols()
            .get_local_by_id(local_position.0, local_position.1, span)?;

    let local_type: ThrushType = local.get_type();
    let is_mutable: bool = local.is_mutable();

    let mut property_names: Vec<&'instr str> = Vec::with_capacity(10);

    let first_property: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected property name."),
    )?;

    let mut span: Span = first_property.span;

    property_names.push(first_property.lexeme);

    while parser_ctx.match_token(TokenKind::Dot)? {
        let property: &Token = parser_ctx.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected property name."),
        )?;

        span = property.span;

        property_names.push(property.lexeme);
    }

    property_names.reverse();

    let decomposed: (ThrushType, Vec<(ThrushType, u32)>) = types::decompose_struct_property(
        0,
        property_names,
        local_type.clone(),
        parser_ctx.get_symbols(),
        span,
    )?;

    Ok(Instruction::Property {
        name,
        indexes: decomposed.1,
        kind: decomposed.0,
        is_mutable,
        span,
    })
}

fn build_ref<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    if symbol.is_constant() {
        let const_id: &str = symbol.expected_constant(span)?;
        let constant: Constant = parser_ctx.get_symbols().get_const_by_id(const_id, span)?;
        let constant_type: ThrushType = constant.get_type();

        return Ok(Instruction::ConstRef {
            name,
            kind: constant_type,
            span,
        });
    }

    let local_position: (&str, usize) = symbol.expected_local(span)?;

    let local: &Local =
        parser_ctx
            .get_symbols()
            .get_local_by_id(local_position.0, local_position.1, span)?;

    let local_type: ThrushType = local.get_type();

    if local.is_undefined() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Local reference '{}' is undefined.", name),
            None,
            span,
        ));
    }

    let localref: Instruction = Instruction::LocalRef {
        name,
        kind: local_type.clone(),
        span,
    };

    if parser_ctx.match_token(TokenKind::PlusPlus)?
        | parser_ctx.match_token(TokenKind::MinusMinus)?
    {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        typecheck::check_unary(&operator, &local_type, span)?;

        let unaryop: Instruction = Instruction::UnaryOp {
            operator,
            expression: localref.into(),
            kind: ThrushType::Void,
            is_pre: false,
            span,
        };

        return Ok(unaryop);
    }

    Ok(localref)
}

fn build_enum_field<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;
    let enum_id: &str = object.expected_enum(span)?;

    let union: EnumFields = parser_ctx
        .get_symbols()
        .get_enum_by_id(enum_id, span)?
        .get_fields();

    let field: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected enum field identifier."),
    )?;

    let field_name: &str = field.lexeme;

    if !union.contain_field(field_name) {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Not found '{}' field in '{}' enum.", name, field_name),
            None,
            span,
        ));
    }

    let field: EnumField = union.get_field(field_name);
    let field_value: Instruction = field.1;

    Ok(field_value)
}

fn build_address<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;
    let local_id: (&str, usize) = object.expected_local(span)?;

    let local: &Local = parser_ctx
        .get_symbols()
        .get_local_by_id(local_id.0, local_id.1, span)?;

    let local_type: ThrushType = local.0.clone();

    if !local_type.is_ptr_type() && !local_type.is_struct_type() && !local_type.is_str_type() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Indexe is only allowed for pointers and structs, not '{}'. ",
                local_type
            ),
            None,
            local.get_span(),
        ));
    }

    let mut indexes: Vec<Instruction> = Vec::with_capacity(10);

    let index: Instruction = build_expr(parser_ctx)?;

    if !index.is_unsigned_integer() || !index.is_anyu32bit_integer() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Expected unsigned integer type (u8, u16, u32), not {}. ",
                index.get_type(),
            ),
            None,
            index.get_span(),
        ));
    }

    parser_ctx.consume(
        TokenKind::RBracket,
        String::from("Syntax error"),
        String::from("Expected ']'."),
    )?;

    indexes.push(index);

    while parser_ctx.match_token(TokenKind::LBracket)? {
        let index: Instruction = build_expr(parser_ctx)?;

        if !index.is_unsigned_integer() || !index.is_anyu32bit_integer() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                format!(
                    "Expected unsigned integer type (u8, u16, u32), not {}. ",
                    index.get_type(),
                ),
                None,
                index.get_span(),
            ));
        }

        parser_ctx.consume(
            TokenKind::RBracket,
            String::from("Syntax error"),
            String::from("Expected ']'."),
        )?;

        indexes.push(index);
    }

    Ok(Instruction::Address {
        name,
        indexes,
        kind: local_type,
        span,
    })
}

fn build_function_call<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    let function_id: &str = object.expected_function(span)?;
    let function: Function = parser_ctx
        .get_symbols()
        .get_function_by_id(span, function_id)?;

    let function_type: ThrushType = function.get_type();
    let ignore_more_args: bool = function.ignore_more_args();

    let parameters: &Parameters = function.get_parameters();
    let maximun_arguments: usize = function.get_parameters_size();

    let mut args: Vec<Instruction> = Vec::with_capacity(10);

    while parser_ctx.peek().kind != TokenKind::RParen {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        let expression: Instruction = build_expr(parser_ctx)?;

        if expression.is_constructor() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Constructor should be stored in a local variable."),
                None,
                expression.get_span(),
            ));
        }

        args.push(expression);
    }

    parser_ctx.consume(
        TokenKind::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    let arguments_size: usize = args.len();

    if args.len() > maximun_arguments && !ignore_more_args {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Expected '{}' arguments, not '{}'.",
                maximun_arguments,
                args.len()
            ),
            None,
            span,
        ));
    }

    if arguments_size != maximun_arguments && !ignore_more_args {
        let display_args_types: String = if !args.is_empty() {
            args.iter()
                .map(|parameter| parameter.get_type().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::from("none")
        };

        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Expected '{}' arguments with types '{}', not '{}'.",
                maximun_arguments,
                function.get_parameters(),
                display_args_types,
            ),
            None,
            span,
        ));
    }

    if !ignore_more_args {
        for (position, argument) in args.iter().enumerate() {
            let from_type: &ThrushType = argument.get_type();
            let target_type: &ThrushType = &parameters.0[position];

            parser_ctx.mismatch_types(target_type, from_type, argument.get_span(), Some(argument));
        }
    }

    Ok(Instruction::Call {
        name,
        args,
        kind: function_type,
        span,
    })
}

fn build_this<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let this_tk: &Token = parser_ctx.consume(
        TokenKind::This,
        String::from("Syntax error"),
        String::from("Expected 'this' keyword."),
    )?;

    let span: Span = this_tk.span;

    if !parser_ctx
        .get_type_ctx()
        .get_this_bindings_type()
        .is_struct_type()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected 'this' inside the a bindings definition context."),
            None,
            span,
        ));
    }

    if !parser_ctx
        .get_mut_control_ctx()
        .get_instr_position()
        .is_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected 'this' inside the a bind definition context."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_type_ctx().get_bind_instance() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from(
                "Expected that 'this' was already declared within the definition of the a previous bind parameter.",
            ),
            None,
            span,
        ));
    }

    if parser_ctx.match_token(TokenKind::Dot)? {
        return build_property(parser_ctx, "this", span);
    }

    let this_type: ThrushType = parser_ctx
        .get_type_ctx()
        .get_this_bindings_type()
        .dissamble();

    let is_mutable: bool = parser_ctx.match_token(TokenKind::Mut)?;

    Ok(Instruction::This {
        kind: this_type,
        is_mutable,
        span,
    })
}

fn build_constructor<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let new_tk: &Token = parser_ctx.consume(
        TokenKind::New,
        String::from("Syntax error"),
        String::from("Expected 'new' keyword."),
    )?;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            new_tk.span,
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected structure reference."),
    )?;

    let span: Span = name.span;
    let struct_name: &str = name.lexeme;

    let struct_found: Struct = parser_ctx.get_symbols().get_struct(struct_name, span)?;

    let fields_required: usize = struct_found.get_fields().1.len();

    parser_ctx.consume(
        TokenKind::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut arguments: Constructor = (struct_name, Vec::with_capacity(10));

    let mut amount: usize = 0;

    while parser_ctx.peek().kind != TokenKind::RBrace {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        if parser_ctx.match_token(TokenKind::Identifier)? {
            let field_tk: &Token = parser_ctx.previous();
            let field_span: Span = field_tk.span;
            let field_name: &str = field_tk.lexeme;

            parser_ctx.consume(
                TokenKind::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            if !struct_found.contains_field(field_name) {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected existing structure field name."),
                    None,
                    field_span,
                ));
            }

            if amount >= fields_required {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Too many fields in structure"),
                    format!("Expected '{}' fields, not '{}'.", fields_required, amount),
                    None,
                    span,
                ));
            }

            let expression: Instruction = build_expr(parser_ctx)?;

            if expression.is_constructor() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Constructor should be stored in a local variable."),
                    None,
                    field_span,
                ));
            }

            let expression_type: &ThrushType = expression.get_type();

            if let Some(target_type) = struct_found.get_field_type(field_name) {
                parser_ctx.mismatch_types(
                    &target_type,
                    expression_type,
                    expression.get_span(),
                    Some(&expression),
                );

                arguments
                    .1
                    .push((field_name, expression, target_type, amount as u32));
            }

            amount += 1;
            continue;
        }

        parser_ctx.only_advance()?;
    }

    let amount_fields: usize = arguments.1.len();

    if amount_fields != fields_required {
        return Err(ThrushCompilerIssue::Error(
            String::from("Missing fields in structure"),
            format!(
                "Expected '{}' arguments, but '{}' was gived.",
                fields_required, amount_fields
            ),
            None,
            span,
        ));
    }

    parser_ctx.consume(
        TokenKind::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    Ok(Instruction::Constructor {
        arguments: arguments.clone(),
        kind: arguments.get_type(),
        span,
    })
}
