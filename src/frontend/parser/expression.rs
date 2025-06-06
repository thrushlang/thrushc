use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokenkind::TokenKind},
        types::{
            lexer::{ThrushType, decompose_struct_property},
            parser::stmts::{
                ident::ReferenceIndentificator,
                sites::LLIAllocationSite,
                stmt::ThrushStatement,
                traits::{
                    ConstructorExtensions, EnumExtensions, EnumFieldsExtensions, FoundSymbolEither,
                    FoundSymbolExtension, StructExtensions, TokenExtensions,
                },
                types::{Constructor, EnumField, EnumFields},
            },
            symbols::{
                traits::{
                    ConstantSymbolExtensions, FunctionExtensions, LLISymbolExtensions,
                    LocalSymbolExtensions, MethodExtensions, MethodsExtensions,
                },
                types::{
                    AssemblerFunction, ConstantSymbol, FoundSymbolId, Function, LLISymbol,
                    LocalSymbol, MethodDef, Methods, ParameterSymbol, Struct,
                },
            },
        },
    },
};

use super::{ParserContext, contexts::SyncPosition, parse, stmt, typegen};

pub fn build_expression<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
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

    let expression: ThrushStatement = self::or(parser_ctx)?;

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(expression)
}

pub fn build_expr<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
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

    let expr: ThrushStatement = or(parser_ctx)?;

    Ok(expr)
}

fn or<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::and(parser_ctx)?;

    while parser_ctx.match_token(TokenKind::Or)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: ThrushStatement = self::and(parser_ctx)?;

        expression = ThrushStatement::BinaryOp {
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
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::equality(parser_ctx)?;

    while parser_ctx.match_token(TokenKind::And)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: ThrushStatement = self::equality(parser_ctx)?;

        expression = ThrushStatement::BinaryOp {
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
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::casts(parser_ctx)?;

    if parser_ctx.match_token(TokenKind::BangEq)? || parser_ctx.match_token(TokenKind::EqEq)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: ThrushStatement = self::casts(parser_ctx)?;

        expression = ThrushStatement::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: ThrushType::Bool,
            span,
        }
    }

    Ok(expression)
}

fn casts<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::cmp(parser_ctx)?;

    if parser_ctx.match_token(TokenKind::CastRaw)? {
        let expression_span: Span = expression.get_span();

        let span: Span = parser_ctx.previous().span;

        if !expression.is_allocated()? {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected an allocated value."),
                None,
                expression_span,
            ));
        }

        let mut cast: ThrushType = typegen::build_type(parser_ctx)?;

        if !cast.is_mut_type() {
            cast = ThrushType::Mut(cast.into());
        }

        expression = ThrushStatement::CastRaw {
            from: expression.into(),
            cast,
            span,
        };
    } else if parser_ctx.match_token(TokenKind::CastPtr)? {
        let expression_span: Span = expression.get_span();

        let span: Span = parser_ctx.previous().span;

        if !expression.is_allocated()? {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected an allocated value."),
                None,
                expression_span,
            ));
        }

        let cast: ThrushType = typegen::build_type(parser_ctx)?;

        expression = ThrushStatement::CastPtr {
            from: expression.into(),
            cast,
            span,
        };
    } else if parser_ctx.match_token(TokenKind::Cast)? {
        let expression_span: Span = expression.get_span();

        let span: Span = parser_ctx.previous().span;

        if !expression.is_allocated()? {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected an allocated value."),
                None,
                expression_span,
            ));
        }

        let cast: ThrushType = typegen::build_type(parser_ctx)?;

        expression = ThrushStatement::Cast {
            from: expression.into(),
            cast,
            span,
        };
    }

    Ok(expression)
}

fn cmp<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::term(parser_ctx)?;

    if parser_ctx.match_token(TokenKind::Greater)?
        || parser_ctx.match_token(TokenKind::GreaterEq)?
        || parser_ctx.match_token(TokenKind::Less)?
        || parser_ctx.match_token(TokenKind::LessEq)?
    {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: ThrushStatement = self::term(parser_ctx)?;

        expression = ThrushStatement::BinaryOp {
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
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::factor(parser_ctx)?;

    while parser_ctx.match_token(TokenKind::Plus)?
        || parser_ctx.match_token(TokenKind::Minus)?
        || parser_ctx.match_token(TokenKind::LShift)?
        || parser_ctx.match_token(TokenKind::RShift)?
    {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: ThrushStatement = self::factor(parser_ctx)?;

        let left_type: &ThrushType = expression.get_value_type()?;
        let right_type: &ThrushType = right.get_value_type()?;

        let kind: &ThrushType = left_type.precompute_type(right_type);

        expression = ThrushStatement::BinaryOp {
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
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = unary(parser_ctx)?;

    while parser_ctx.match_token(TokenKind::Slash)? || parser_ctx.match_token(TokenKind::Star)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: ThrushStatement = self::unary(parser_ctx)?;

        let left_type: &ThrushType = expression.get_value_type()?;
        let right_type: &ThrushType = right.get_value_type()?;

        let kind: &ThrushType = left_type.precompute_type(right_type);

        expression = ThrushStatement::BinaryOp {
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
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    if parser_ctx.match_token(TokenKind::Bang)? {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let expression: ThrushStatement = self::primary(parser_ctx)?;

        return Ok(ThrushStatement::UnaryOp {
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

        let mut expression: ThrushStatement = self::primary(parser_ctx)?;

        expression.cast_signess(operator);

        let expression_type: &ThrushType = expression.get_value_type()?;

        return Ok(ThrushStatement::UnaryOp {
            operator,
            expression: expression.clone().into(),
            kind: expression_type.clone(),
            is_pre: false,
            span,
        });
    }

    let instr: ThrushStatement = primary(parser_ctx)?;

    Ok(instr)
}

fn primary<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let primary: ThrushStatement = match &parser_ctx.peek().kind {
        TokenKind::Deref => self::build_deref(parser_ctx)?,

        TokenKind::RawPtr => {
            let raw_ptr_tk: &Token = parser_ctx.advance()?;
            let span: Span = raw_ptr_tk.span;

            let reference_tk: &Token = parser_ctx.consume(
                TokenKind::Identifier,
                "Syntax error".into(),
                "Expected 'reference'.".into(),
            )?;

            let ref_name: &str = reference_tk.lexeme;
            let ref_span: Span = reference_tk.span;

            let reference: ThrushStatement = self::build_reference(parser_ctx, ref_name, ref_span)?;

            if !reference.is_reference_allocated() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected allocated reference."),
                    None,
                    span,
                ));
            }

            let mut reference_type: ThrushType = reference.get_value_type()?.clone();

            if !reference_type.is_ptr_type() {
                if !reference_type.is_mut_type() {
                    reference_type = ThrushType::Ptr(Some(reference_type.into()));
                } else {
                    let defered_type: ThrushType = reference_type.defer_mut_all();
                    reference_type = ThrushType::Ptr(Some(defered_type.into()));
                }
            }

            ThrushStatement::RawPtr {
                from: reference.into(),
                kind: reference_type,
                span,
            }
        }

        TokenKind::Alloc => {
            let alloc_tk: &Token = parser_ctx.advance()?;
            let span: Span = alloc_tk.span;

            let site_allocation: LLIAllocationSite = match parser_ctx.peek().kind {
                TokenKind::Heap => {
                    parser_ctx.only_advance()?;
                    LLIAllocationSite::Heap
                }
                TokenKind::Stack => {
                    parser_ctx.only_advance()?;
                    LLIAllocationSite::Stack
                }
                TokenKind::Static => {
                    parser_ctx.only_advance()?;
                    LLIAllocationSite::Static
                }
                _ => {
                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Expected site allocation flag."),
                        None,
                        span,
                    ));
                }
            };

            parser_ctx.consume(
                TokenKind::LBrace,
                "Syntax error".into(),
                "Expected '{'.".into(),
            )?;

            let mut alloc_type: ThrushType = typegen::build_type(parser_ctx)?;

            alloc_type = ThrushType::Ptr(Some(alloc_type.into()));

            let attributes: Vec<LLVMAttribute> = if parser_ctx.match_token(TokenKind::LBrace)? {
                stmt::build_attributes(parser_ctx, &[TokenKind::RBrace, TokenKind::SemiColon])?
            } else {
                Vec::new()
            };

            parser_ctx.consume(
                TokenKind::RBrace,
                "Syntax error".into(),
                "Expected '}'.".into(),
            )?;

            ThrushStatement::Alloc {
                type_to_alloc: alloc_type,
                site_allocation,
                attributes,
                span,
            }
        }

        TokenKind::Load => {
            let load_tk: &Token = parser_ctx.advance()?;
            let span: Span = load_tk.span;

            let load_type: ThrushType = typegen::build_type(parser_ctx)?;

            parser_ctx.consume(
                TokenKind::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;

            if parser_ctx.check(TokenKind::Identifier) {
                let identifier_tk: &Token = parser_ctx.consume(
                    TokenKind::Identifier,
                    String::from("Syntax error"),
                    String::from("Expected 'reference'."),
                )?;

                let name: &str = identifier_tk.lexeme;

                let reference: ThrushStatement = self::build_reference(parser_ctx, name, span)?;

                if !reference.is_reference_lli() {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected Low Level Instruction (LLI) reference.".into(),
                        None,
                        span,
                    ));
                }

                if !reference.is_reference_allocated() {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected Low Level Instruction (LLI) allocated.".into(),
                        None,
                        span,
                    ));
                }

                return Ok(ThrushStatement::Load {
                    load: (Some(name), None),
                    kind: load_type,
                    span,
                });
            }

            let expression: ThrushStatement = self::build_expr(parser_ctx)?;

            ThrushStatement::Load {
                load: (None, Some(expression.into())),
                kind: load_type,
                span,
            }
        }

        TokenKind::Write => {
            let write_tk: &Token = parser_ctx.advance()?;
            let span: Span = write_tk.span;

            if parser_ctx.match_token(TokenKind::Identifier)? {
                let identifier_tk: &Token = parser_ctx.previous();

                let name: &str = identifier_tk.lexeme;

                let reference: ThrushStatement = self::build_reference(parser_ctx, name, span)?;

                if !reference.is_reference_lli() {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected LLI, reference.".into(),
                        None,
                        span,
                    ));
                }

                parser_ctx.consume(
                    TokenKind::Comma,
                    "Syntax error".into(),
                    "Expected ','.".into(),
                )?;

                let write_type: ThrushType = typegen::build_type(parser_ctx)?;

                let value: ThrushStatement = self::build_expr(parser_ctx)?;

                return Ok(ThrushStatement::Write {
                    write_to: (Some(name), None),
                    write_value: value.clone().into(),
                    write_type,
                    span,
                });
            }

            let expression: ThrushStatement = self::build_expr(parser_ctx)?;
            let expression_type: &ThrushType = expression.get_value_type()?;

            if !expression_type.is_ptr_type() && !expression_type.is_address_type() {
                return Err(ThrushCompilerIssue::Error(
                    "Attemping to access an invalid pointer".into(),
                    format!(
                        "Write is only allowed for pointer types or memory address, not '{}'. ",
                        expression_type
                    ),
                    None,
                    span,
                ));
            }

            parser_ctx.consume(
                TokenKind::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;

            let write_type: ThrushType = typegen::build_type(parser_ctx)?;

            let value: ThrushStatement = self::build_expr(parser_ctx)?;

            ThrushStatement::Write {
                write_to: (None, Some(expression.into())),
                write_value: value.clone().into(),
                write_type,
                span,
            }
        }

        TokenKind::Address => {
            parser_ctx.only_advance()?;

            let identifier_tk: &Token = parser_ctx.consume(
                TokenKind::Identifier,
                String::from("Syntax error"),
                String::from("Expected 'reference'."),
            )?;

            let name: &str = identifier_tk.lexeme;
            let span: Span = identifier_tk.span;

            parser_ctx.consume(
                TokenKind::LBrace,
                String::from("Syntax error"),
                String::from("Expected '{'."),
            )?;

            return self::build_address(parser_ctx, name, span);
        }

        TokenKind::PlusPlus => {
            let operator_tk: &Token = parser_ctx.advance()?;
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let expression: ThrushStatement = self::build_expr(parser_ctx)?;

            if !expression.is_reference() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Only local references can be pre-incremented.".into(),
                    None,
                    parser_ctx.previous().span,
                ));
            }

            let reftype: ThrushType = expression.get_reference_type()?;

            let unaryop: ThrushStatement = ThrushStatement::UnaryOp {
                operator,
                expression: expression.into(),
                kind: reftype,
                is_pre: true,
                span,
            };

            return Ok(unaryop);
        }

        TokenKind::MinusMinus => {
            let operator_tk: &Token = parser_ctx.advance()?;
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let expression: ThrushStatement = self::build_expr(parser_ctx)?;

            if !expression.is_reference() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Only local references can be pre-decremented.".into(),
                    None,
                    parser_ctx.previous().span,
                ));
            }

            let reftype: ThrushType = expression.get_reference_type()?;

            let unaryop: ThrushStatement = ThrushStatement::UnaryOp {
                operator,
                expression: expression.into(),
                kind: reftype,
                is_pre: true,
                span,
            };

            return Ok(unaryop);
        }

        TokenKind::LParen => {
            let span: Span = parser_ctx.advance()?.span;

            let expression: ThrushStatement = build_expr(parser_ctx)?;

            let kind: &ThrushType = expression.get_value_type()?;

            if !expression.is_binary() && !expression.is_group() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Grouping '(...)' is only allowed with binary expressions or other grouped expressions.".into(),
                    None,
                    span,
                ));
            }

            parser_ctx.consume(
                TokenKind::RParen,
                "Syntax error".into(),
                "Expected ')'.".into(),
            )?;

            return Ok(ThrushStatement::Group {
                expression: expression.clone().into(),
                kind: kind.clone(),
                span,
            });
        }

        TokenKind::Str => {
            let str_tk: &Token = parser_ctx.advance()?;
            let lexeme: &str = str_tk.lexeme;
            let span: Span = str_tk.span;

            ThrushStatement::new_str(ThrushType::Str, lexeme.to_bytes(span)?, span)
        }

        TokenKind::Char => {
            let char_tk: &Token = parser_ctx.advance()?;
            let span: Span = char_tk.span;
            let lexeme: &str = char_tk.lexeme;

            ThrushStatement::new_char(ThrushType::Char, lexeme.get_first_byte(), span)
        }

        TokenKind::NullPtr => ThrushStatement::NullPtr {
            span: parser_ctx.advance()?.span,
        },

        TokenKind::Integer => {
            let integer_tk: &Token = parser_ctx.advance()?;
            let integer: &str = integer_tk.lexeme;
            let span: Span = integer_tk.span;

            let parsed_integer: (ThrushType, u64) = parse::integer(integer, span)?;

            let integer_type: ThrushType = parsed_integer.0;
            let integer_value: u64 = parsed_integer.1;

            ThrushStatement::new_integer(integer_type, integer_value, false, span)
        }

        TokenKind::Float => {
            let float_tk: &Token = parser_ctx.advance()?;

            let float: &str = float_tk.lexeme;
            let span: Span = float_tk.span;

            let parsed_float: (ThrushType, f64) = parse::float(float, span)?;

            let float_type: ThrushType = parsed_float.0;
            let float_value: f64 = parsed_float.1;

            ThrushStatement::new_float(float_type, float_value, false, span)
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
                        "Syntax error".into(),
                        "Constants cannot be mutated.".into(),
                        None,
                        span,
                    ));
                }

                if object.is_lli() {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "LLI's cannot be mutated.".into(),
                        None,
                        span,
                    ));
                }

                let local_position: (&str, usize) = object.expected_local(span)?;

                let local: &LocalSymbol = parser_ctx.get_symbols().get_local_by_id(
                    local_position.0,
                    local_position.1,
                    span,
                )?;

                let local_span: Span = local.get_span();

                let local_type: ThrushType = local.0.clone();

                if !local.is_mutable() {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Make mutable the local.".into(),
                        None,
                        local_span,
                    ));
                }

                let expression: ThrushStatement = build_expr(parser_ctx)?;

                return Ok(ThrushStatement::Mut {
                    source: (Some(name), None),
                    value: expression.into(),
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
                let property: ThrushStatement = build_property(parser_ctx, name, span)?;

                if parser_ctx.match_token(TokenKind::Eq)? {
                    let expr: ThrushStatement = build_expr(parser_ctx)?;

                    if !property.is_mutable() {
                        return Err(ThrushCompilerIssue::Error(
                            "Syntax error".into(),
                            "Make mutable the parameter or local with this property.".into(),
                            None,
                            property.get_span(),
                        ));
                    }

                    return Ok(ThrushStatement::Mut {
                        source: (None, Some(property.clone().into())),
                        value: expr.into(),
                        kind: property.get_value_type()?.clone(),
                        span,
                    });
                }

                return Ok(property);
            }

            if parser_ctx.match_token(TokenKind::ColonColon)? {
                return self::build_method_call(parser_ctx, name, span);
            }

            if symbol.is_enum() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Enums cannot be used as types; use properties instead with their types."
                        .into(),
                    None,
                    span,
                ));
            }

            if symbol.is_function() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Functions cannot be used as types; call it instead.".into(),
                    None,
                    span,
                ));
            }

            self::build_reference(parser_ctx, name, span)?
        }

        TokenKind::True => {
            ThrushStatement::new_boolean(ThrushType::Bool, 1, parser_ctx.advance()?.span)
        }

        TokenKind::False => {
            ThrushStatement::new_boolean(ThrushType::Bool, 0, parser_ctx.advance()?.span)
        }

        TokenKind::This => self::build_this(parser_ctx)?,
        TokenKind::New => self::build_constructor(parser_ctx)?,

        TokenKind::Pass => ThrushStatement::Pass {
            span: parser_ctx.advance()?.span,
        },

        _ => {
            let previous: &Token = parser_ctx.advance()?;

            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                format!("Statement '{}' don't allowed.", previous.lexeme),
                None,
                previous.span,
            ));
        }
    };

    Ok(primary)
}

fn build_method_call<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    let structure_id: &str = symbol.expected_struct(span)?;

    let structure: Struct = parser_ctx
        .get_symbols()
        .get_struct_by_id(structure_id, span)?;

    let method_tk: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected method name."),
    )?;

    let method_name: &str = method_tk.lexeme;

    let methods: Methods = structure.get_methods();

    if !methods.contains_method(method_name) {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Not found '{}' method inside the methods of '{}' struct.",
                method_name, name
            ),
            None,
            span,
        ));
    }

    let method: MethodDef = methods.get_method(method_name);

    let method_name: &str = method.get_name();
    let method_type: ThrushType = method.get_type();

    parser_ctx.consume(
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut args: Vec<ThrushStatement> = Vec::with_capacity(10);

    while parser_ctx.peek().kind != TokenKind::RParen {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        let expression: ThrushStatement = self::build_expr(parser_ctx)?;

        args.push(expression);
    }

    parser_ctx.consume(
        TokenKind::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    let canonical_name: String = format!("{}.{}", name, method_name);

    Ok(ThrushStatement::MethodCall {
        name: canonical_name,
        args,
        kind: method_type,
        span,
    })
}

fn build_property<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    let local_position: (&str, usize) = symbol.expected_local(span)?;

    let local: &LocalSymbol =
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

    let decomposed: (ThrushType, Vec<(ThrushType, u32)>) = decompose_struct_property(
        0,
        property_names,
        local_type.clone(),
        parser_ctx.get_symbols(),
        span,
    )?;

    Ok(ThrushStatement::Property {
        name,
        indexes: decomposed.1,
        kind: decomposed.0,
        is_mutable,
        span,
    })
}

fn build_reference<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    if symbol.is_constant() {
        let const_id: &str = symbol.expected_constant(span)?;
        let constant: ConstantSymbol = parser_ctx.get_symbols().get_const_by_id(const_id, span)?;
        let constant_type: ThrushType = constant.get_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: constant_type,
            span,
            identificator: ReferenceIndentificator::Constant,
            is_allocated: true,
        });
    }

    if symbol.is_parameter() {
        let parameter_id: &str = symbol.expected_parameter(span)?;
        let parameter: ParameterSymbol = parser_ctx
            .get_symbols()
            .get_parameter_by_id(parameter_id, span)?;
        let parameter_type: ThrushType = parameter.get_type();

        let is_allocated: bool = parameter_type.is_mut_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: parameter_type,
            span,
            identificator: ReferenceIndentificator::FunctionParameter,
            is_allocated,
        });
    }

    if symbol.is_lli() {
        let lli_id: (&str, usize) = symbol.expected_lli(span)?;

        let lli_name: &str = lli_id.0;
        let scope_idx: usize = lli_id.1;

        let parameter: &LLISymbol = parser_ctx
            .get_symbols()
            .get_lli_by_id(lli_name, scope_idx, span)?;

        let lli_type: ThrushType = parameter.get_type();

        let is_allocated: bool = lli_type.is_ptr_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: lli_type,
            span,
            identificator: ReferenceIndentificator::LowLevelInstruction,
            is_allocated,
        });
    }

    let local_position: (&str, usize) = symbol.expected_local(span)?;

    let local: &LocalSymbol =
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

    let reference: ThrushStatement = ThrushStatement::Reference {
        name,
        kind: local_type.clone(),
        span,
        identificator: ReferenceIndentificator::Local,
        is_allocated: true,
    };

    if parser_ctx.match_token(TokenKind::PlusPlus)?
        | parser_ctx.match_token(TokenKind::MinusMinus)?
    {
        let operator_tk: &Token = parser_ctx.previous();
        let operator: TokenKind = operator_tk.kind;
        let span: Span = operator_tk.span;

        let unaryop: ThrushStatement = ThrushStatement::UnaryOp {
            operator,
            expression: reference.into(),
            kind: local_type,
            is_pre: false,
            span,
        };

        return Ok(unaryop);
    }

    Ok(reference)
}

fn build_enum_field<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;
    let enum_id: &str = object.expected_enum(span)?;

    let union: EnumFields = parser_ctx
        .get_symbols()
        .get_enum_by_id(enum_id, span)?
        .get_fields();

    let field_tk: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected enum name."),
    )?;

    let field_name: &str = field_tk.lexeme;

    if !union.contain_field(field_name) {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Not found '{}' field in '{}' enum.", name, field_name),
            None,
            span,
        ));
    }

    let field: EnumField = union.get_field(field_name);
    let field_value: ThrushStatement = field.1;
    let field_type: ThrushType = field_value.get_value_type()?.clone();

    let canonical_name: String = format!("{}.{}", name, field_name);

    Ok(ThrushStatement::EnumValue {
        name: canonical_name,
        value: field_value.into(),
        kind: field_type,
        span,
    })
}

fn build_address<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    object.expected_lli(span)?;

    let mut indexes: Vec<ThrushStatement> = Vec::with_capacity(10);

    let index: ThrushStatement = self::build_expr(parser_ctx)?;

    indexes.push(index);

    while parser_ctx.match_token(TokenKind::Comma)? {
        let index: ThrushStatement = self::build_expr(parser_ctx)?;

        indexes.push(index);
    }

    parser_ctx.consume(
        TokenKind::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if indexes.is_empty() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            "At least one index was expected.".into(),
            None,
            span,
        ));
    }

    Ok(ThrushStatement::Address {
        name,
        indexes,
        kind: ThrushType::Addr,
        span,
    })
}

fn build_function_call<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_ctx.get_symbols().get_symbols_id(name, span)?;

    let function_type: ThrushType = if object.is_function_asm() {
        let asm_function_id: &str = object.expected_asm_function(span)?;
        let asm_function: AssemblerFunction = parser_ctx
            .get_symbols()
            .get_asm_function_by_id(span, asm_function_id)?;

        asm_function.get_type()
    } else {
        let function_id: &str = object.expected_function(span)?;
        let function: Function = parser_ctx
            .get_symbols()
            .get_function_by_id(span, function_id)?;

        function.get_type()
    };

    let mut args: Vec<ThrushStatement> = Vec::with_capacity(10);

    loop {
        if parser_ctx.check(TokenKind::RParen) {
            break;
        }

        let expression: ThrushStatement = self::build_expr(parser_ctx)?;

        if expression.is_constructor() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Constructor should be stored in a local variable."),
                None,
                expression.get_span(),
            ));
        }

        args.push(expression);

        if parser_ctx.check(TokenKind::RParen) {
            break;
        } else {
            parser_ctx.consume(
                TokenKind::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_ctx.consume(
        TokenKind::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ThrushStatement::Call {
        name,
        args,
        kind: function_type,
        span,
    })
}

fn build_this<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let this_tk: &Token = parser_ctx.consume(
        TokenKind::This,
        String::from("Syntax error"),
        String::from("Expected 'this' keyword."),
    )?;

    let span: Span = this_tk.span;

    if !parser_ctx
        .get_type_ctx()
        .get_this_methods_type()
        .is_struct_type()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected 'this' inside the a methods definition context."),
            None,
            span,
        ));
    }

    if !parser_ctx
        .get_mut_control_ctx()
        .get_instr_position()
        .is_method()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected 'this' inside the a method definition context."),
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
        .get_this_methods_type()
        .dissamble();

    let is_mutable: bool = parser_ctx.match_token(TokenKind::Mut)?;

    Ok(ThrushStatement::This {
        kind: this_type,
        is_mutable,
        span,
    })
}

fn build_constructor<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
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

    loop {
        if parser_ctx.check(TokenKind::RBrace) {
            break;
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

            let expression: ThrushStatement = self::build_expr(parser_ctx)?;

            if expression.is_constructor() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Constructor should be stored in a local variable."),
                    None,
                    field_span,
                ));
            }

            if let Some(target_type) = struct_found.get_field_type(field_name) {
                arguments
                    .1
                    .push((field_name, expression, target_type, amount as u32));
            }

            amount += 1;

            if parser_ctx.check(TokenKind::RBrace) {
                break;
            }

            if parser_ctx.match_token(TokenKind::Comma)? {
                if parser_ctx.check(TokenKind::RBrace) {
                    break;
                }
            } else if parser_ctx.check_to(TokenKind::Identifier, 0) {
                parser_ctx.consume(
                    TokenKind::Comma,
                    String::from("Syntax error"),
                    String::from("Expected ','."),
                )?;
            } else {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected identifier."),
                    None,
                    parser_ctx.previous().span,
                ));
            }
        } else {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected field name."),
                None,
                span,
            ));
        }
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

    Ok(ThrushStatement::Constructor {
        name: struct_name,
        arguments: arguments.clone(),
        kind: arguments.get_type(),
        span,
    })
}

pub fn build_deref<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let initial_deref_tk: &Token = parser_ctx.advance()?;
    let span: Span = initial_deref_tk.span;

    let mut deref_count: u64 = 1;

    let mut current_expr: ThrushStatement = {
        while parser_ctx.check(TokenKind::Deref) {
            parser_ctx.consume(
                TokenKind::Deref,
                "Syntax error".into(),
                "Expected 'deref'.".into(),
            )?;
            deref_count += 1;
        }

        let reference_tk: &Token = parser_ctx.consume(
            TokenKind::Identifier,
            "Syntax error".into(),
            "Expected 'identifier'.".into(),
        )?;

        let ref_name: &str = reference_tk.lexeme;
        let ref_span: Span = reference_tk.span;

        let reference: ThrushStatement = self::build_reference(parser_ctx, ref_name, ref_span)?;

        if !reference.is_allocated()? {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected an allocated value."),
                None,
                ref_span,
            ));
        }

        reference
    };

    let mut current_type: ThrushType = current_expr.get_value_type()?.clone();

    for _ in 0..deref_count {
        current_expr = ThrushStatement::Deref {
            load: current_expr.clone().into(),
            kind: current_type.deref_ptr(),
            span,
        };

        current_type = current_type.deref_ptr();
    }

    Ok(current_expr)
}
