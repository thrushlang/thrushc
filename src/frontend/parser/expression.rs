use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::expression,
        types::{
            lexer::{ThrushType, decompose_struct_property, traits::ThrushTypeMutableExtensions},
            parser::stmts::{
                ident::ReferenceIdentificator,
                sites::LLIAllocationSite,
                stmt::ThrushStatement,
                traits::{
                    ConstructorExtensions, EnumExtensions, EnumFieldsExtensions, FoundSymbolEither,
                    FoundSymbolExtension, StructExtensions, TokenExtensions,
                },
                types::{Constructor, EnumField, EnumFields, ThrushAttributes},
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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_context
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Expression);

    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            parser_context.peek().get_span(),
        ));
    }

    let expression: ThrushStatement = self::or(parser_context)?;

    parser_context.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(expression)
}

pub fn build_expr<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_context
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Expression);

    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            parser_context.peek().get_span(),
        ));
    }

    let expr: ThrushStatement = or(parser_context)?;

    Ok(expr)
}

fn or<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::and(parser_context)?;

    while parser_context.match_token(TokenType::Or)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.get_span();

        let right: ThrushStatement = self::and(parser_context)?;

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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::equality(parser_context)?;

    while parser_context.match_token(TokenType::And)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: ThrushStatement = self::equality(parser_context)?;

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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::casts(parser_context)?;

    if parser_context.match_token(TokenType::BangEq)?
        || parser_context.match_token(TokenType::EqEq)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.get_span();

        let right: ThrushStatement = self::casts(parser_context)?;

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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::cmp(parser_context)?;

    if parser_context.match_token(TokenType::CastRaw)? {
        let span: Span = parser_context.previous().span;

        let mut cast: ThrushType = typegen::build_type(parser_context)?;

        if !cast.is_mut_type() {
            cast = ThrushType::Mut(cast.into());
        }

        expression = ThrushStatement::CastRaw {
            from: expression.into(),
            cast,
            span,
        };
    } else if parser_context.match_token(TokenType::CastPtr)? {
        let span: Span = parser_context.previous().get_span();

        let cast: ThrushType = typegen::build_type(parser_context)?;

        expression = ThrushStatement::CastPtr {
            from: expression.into(),
            cast,
            span,
        };
    } else if parser_context.match_token(TokenType::Cast)? {
        let span: Span = parser_context.previous().get_span();

        let cast: ThrushType = typegen::build_type(parser_context)?;

        expression = ThrushStatement::Cast {
            from: expression.into(),
            cast,
            span,
        };
    }

    Ok(expression)
}

fn cmp<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::term(parser_context)?;

    if parser_context.match_token(TokenType::Greater)?
        || parser_context.match_token(TokenType::GreaterEq)?
        || parser_context.match_token(TokenType::Less)?
        || parser_context.match_token(TokenType::LessEq)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: ThrushStatement = self::term(parser_context)?;

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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = self::factor(parser_context)?;

    while parser_context.match_token(TokenType::Plus)?
        || parser_context.match_token(TokenType::Minus)?
        || parser_context.match_token(TokenType::LShift)?
        || parser_context.match_token(TokenType::RShift)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: ThrushStatement = self::factor(parser_context)?;

        let left_type: &ThrushType = expression.get_value_type()?;
        let right_type: &ThrushType = right.get_value_type()?;

        let kind: &ThrushType = left_type.precompute_numeric_type(right_type);

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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let mut expression: ThrushStatement = unary(parser_context)?;

    while parser_context.match_token(TokenType::Slash)?
        || parser_context.match_token(TokenType::Star)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: ThrushStatement = self::unary(parser_context)?;

        let left_type: &ThrushType = expression.get_value_type()?;
        let right_type: &ThrushType = right.get_value_type()?;

        let kind: &ThrushType = left_type.precompute_numeric_type(right_type);

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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    if parser_context.match_token(TokenType::Bang)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.span;

        let expression: ThrushStatement = self::primary(parser_context)?;

        return Ok(ThrushStatement::UnaryOp {
            operator,
            expression: expression.into(),
            kind: ThrushType::Bool,
            is_pre: false,
            span,
        });
    }

    if parser_context.match_token(TokenType::Minus)? {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let mut expression: ThrushStatement = self::primary(parser_context)?;

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

    let instr: ThrushStatement = primary(parser_context)?;

    Ok(instr)
}

fn primary<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let primary: ThrushStatement = match &parser_context.peek().kind {
        TokenType::LBracket => self::build_array(parser_context)?,

        TokenType::Deref => self::build_deref(parser_context)?,

        TokenType::Raw => {
            let raw_tk: &Token = parser_context.advance()?;
            let span: Span = raw_tk.get_span();

            let identifier_tk: &Token = parser_context.consume(
                TokenType::Identifier,
                String::from("Syntax error"),
                String::from("Expected 'identifier'."),
            )?;

            let reference_name: &str = identifier_tk.get_lexeme();

            let reference: ThrushStatement =
                self::build_reference(parser_context, reference_name, span)?;

            let mut reference_type: ThrushType = reference.get_value_type()?.clone();

            if reference_type.is_mut_type() {
                let deferenced_type: ThrushType = reference_type.defer_mut_all();
                reference_type = deferenced_type;
            }

            reference_type = ThrushType::Ptr(Some(reference_type.into()));

            ThrushStatement::Raw {
                reference: (reference_name, reference.into()),
                kind: reference_type,
                span,
            }
        }

        TokenType::Alloc => {
            let alloc_tk: &Token = parser_context.advance()?;
            let span: Span = alloc_tk.get_span();

            let site_allocation: LLIAllocationSite = match parser_context.peek().kind {
                TokenType::Heap => {
                    parser_context.only_advance()?;
                    LLIAllocationSite::Heap
                }
                TokenType::Stack => {
                    parser_context.only_advance()?;
                    LLIAllocationSite::Stack
                }
                TokenType::Static => {
                    parser_context.only_advance()?;
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

            parser_context.consume(
                TokenType::LBrace,
                "Syntax error".into(),
                "Expected '{'.".into(),
            )?;

            let mut alloc_type: ThrushType = typegen::build_type(parser_context)?;

            alloc_type = ThrushType::Ptr(Some(alloc_type.into()));

            let attributes: Vec<LLVMAttribute> = if parser_context.match_token(TokenType::LBrace)? {
                stmt::build_attributes(parser_context, &[TokenType::RBrace, TokenType::SemiColon])?
            } else {
                Vec::new()
            };

            parser_context.consume(
                TokenType::RBrace,
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

        TokenType::Load => {
            let load_tk: &Token = parser_context.advance()?;
            let span: Span = load_tk.get_span();

            let load_type: ThrushType = typegen::build_type(parser_context)?;

            parser_context.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;

            if parser_context.check(TokenType::Identifier) {
                let identifier_tk: &Token = parser_context.consume(
                    TokenType::Identifier,
                    String::from("Syntax error"),
                    String::from("Expected 'identifier'."),
                )?;

                let reference_name: &str = identifier_tk.get_lexeme();

                let reference: ThrushStatement =
                    self::build_reference(parser_context, reference_name, span)?;

                return Ok(ThrushStatement::Load {
                    value: (Some((reference_name, reference.into())), None),
                    kind: load_type,
                    span,
                });
            }

            let expression: ThrushStatement = self::build_expr(parser_context)?;

            ThrushStatement::Load {
                value: (None, Some(expression.into())),
                kind: load_type,
                span,
            }
        }

        TokenType::Write => {
            let write_tk: &Token = parser_context.advance()?;
            let span: Span = write_tk.span;

            if parser_context.match_token(TokenType::Identifier)? {
                let identifier_tk: &Token = parser_context.previous();
                let name: &str = identifier_tk.get_lexeme();

                let reference: ThrushStatement = self::build_reference(parser_context, name, span)?;

                parser_context.consume(
                    TokenType::Comma,
                    "Syntax error".into(),
                    "Expected ','.".into(),
                )?;

                let write_type: ThrushType = typegen::build_type(parser_context)?;

                let value: ThrushStatement = self::build_expr(parser_context)?;

                return Ok(ThrushStatement::Write {
                    write_to: (Some((name, reference.into())), None),
                    write_value: value.clone().into(),
                    write_type,
                    span,
                });
            }

            let expression: ThrushStatement = self::build_expr(parser_context)?;

            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;

            let write_type: ThrushType = typegen::build_type(parser_context)?;
            let value: ThrushStatement = self::build_expr(parser_context)?;

            ThrushStatement::Write {
                write_to: (None, Some(expression.into())),
                write_value: value.clone().into(),
                write_type,
                span,
            }
        }

        TokenType::Address => {
            let address_tk: &Token = parser_context.advance()?;
            let address_span: Span = address_tk.get_span();

            if parser_context.match_token(TokenType::Identifier)? {
                let identifier_tk: &Token = parser_context.previous();

                let name: &str = identifier_tk.get_lexeme();
                let span: Span = identifier_tk.get_span();

                let reference: ThrushStatement = self::build_reference(parser_context, name, span)?;

                let indexes: Vec<ThrushStatement> =
                    self::build_address_indexes(parser_context, span)?;

                return Ok(ThrushStatement::Address {
                    address_to: (Some((name, reference.into())), None),
                    indexes,
                    kind: ThrushType::Addr,
                    span: address_span,
                });
            }

            let expr: ThrushStatement = self::build_expr(parser_context)?;
            let expr_span: Span = expr.get_span();

            let indexes: Vec<ThrushStatement> =
                self::build_address_indexes(parser_context, expr_span)?;

            return Ok(ThrushStatement::Address {
                address_to: (None, Some(expr.into())),
                indexes,
                kind: ThrushType::Addr,
                span: address_span,
            });
        }

        TokenType::PlusPlus => {
            let operator_tk: &Token = parser_context.advance()?;
            let operator: TokenType = operator_tk.get_type();
            let span: Span = operator_tk.get_span();

            let expression: ThrushStatement = self::build_expr(parser_context)?;

            if !expression.is_reference() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Only local references can be pre-incremented.".into(),
                    None,
                    expression.get_span(),
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

        TokenType::MinusMinus => {
            let operator_tk: &Token = parser_context.advance()?;
            let operator: TokenType = operator_tk.get_type();
            let span: Span = operator_tk.get_span();

            let expression: ThrushStatement = self::build_expr(parser_context)?;

            if !expression.is_reference() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Only local references can be pre-decremented.".into(),
                    None,
                    expression.get_span(),
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

        TokenType::LParen => {
            let span: Span = parser_context.advance()?.get_span();

            let expression: ThrushStatement = build_expr(parser_context)?;

            let kind: &ThrushType = expression.get_value_type()?;

            if !expression.is_binary() && !expression.is_group() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Grouping '(...)' is only allowed with binary expressions or other grouped expressions.".into(),
                    None,
                    span,
                ));
            }

            parser_context.consume(
                TokenType::RParen,
                "Syntax error".into(),
                "Expected ')'.".into(),
            )?;

            return Ok(ThrushStatement::Group {
                expression: expression.clone().into(),
                kind: kind.clone(),
                span,
            });
        }

        TokenType::Str => {
            let str_tk: &Token = parser_context.advance()?;
            let span: Span = str_tk.get_span();

            let bytes: Vec<u8> = str_tk.fix_lexeme_scapes(span)?;

            if let Ok(size) = u32::try_from(bytes.len()) {
                return Ok(ThrushStatement::new_str(
                    ThrushType::Str(ThrushType::FixedArray(ThrushType::U8.into(), size).into()),
                    bytes,
                    span,
                ));
            }

            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Could not get the size of a string, because it's too large.".into(),
                None,
                span,
            ));
        }

        TokenType::Char => {
            let char_tk: &Token = parser_context.advance()?;
            let span: Span = char_tk.get_span();

            ThrushStatement::new_char(ThrushType::Char, char_tk.get_lexeme_first_byte(), span)
        }

        TokenType::NullPtr => ThrushStatement::NullPtr {
            span: parser_context.advance()?.span,
        },

        TokenType::Integer => {
            let integer_tk: &Token = parser_context.advance()?;
            let integer: &str = integer_tk.get_lexeme();
            let span: Span = integer_tk.get_span();

            let parsed_integer: (ThrushType, u64) = parse::integer(integer, span)?;

            let integer_type: ThrushType = parsed_integer.0;
            let integer_value: u64 = parsed_integer.1;

            ThrushStatement::new_integer(integer_type, integer_value, false, span)
        }

        TokenType::Float => {
            let float_tk: &Token = parser_context.advance()?;

            let float: &str = float_tk.get_lexeme();
            let span: Span = float_tk.get_span();

            let parsed_float: (ThrushType, f64) = parse::float(float, span)?;

            let float_type: ThrushType = parsed_float.0;
            let float_value: f64 = parsed_float.1;

            ThrushStatement::new_float(float_type, float_value, false, span)
        }

        TokenType::Identifier => {
            let identifier_tk: &Token = parser_context.advance()?;

            let name: &str = identifier_tk.get_lexeme();
            let span: Span = identifier_tk.get_span();

            let symbol: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

            if parser_context.match_token(TokenType::Eq)? {
                let reference: ThrushStatement = self::build_reference(parser_context, name, span)?;
                let reference_type: ThrushType = reference.get_value_type()?.clone();

                let expression: ThrushStatement = self::build_expr(parser_context)?;

                return Ok(ThrushStatement::Mut {
                    source: (Some((name, reference.clone().into())), None),
                    value: expression.into(),
                    kind: ThrushType::Void,
                    cast_type: reference_type,
                    span,
                });
            }

            if parser_context.match_token(TokenType::LBracket)? {
                let index: ThrushStatement = self::build_index(parser_context, name, span)?;
                let index_type: ThrushType = index.get_value_type()?.clone();

                if parser_context.match_token(TokenType::Eq)? {
                    let expr: ThrushStatement = self::build_expr(parser_context)?;

                    return Ok(ThrushStatement::Mut {
                        source: (None, Some(index.clone().into())),
                        value: expr.into(),
                        kind: ThrushType::Void,
                        cast_type: index_type,
                        span,
                    });
                }

                return Ok(index);
            }

            if parser_context.match_token(TokenType::Arrow)? {
                return self::build_enum_field(parser_context, name, span);
            }

            if parser_context.match_token(TokenType::LParen)? {
                return self::build_function_call(parser_context, name, span);
            }

            if parser_context.match_token(TokenType::Dot)? {
                let property: ThrushStatement = self::build_property(parser_context, name, span)?;
                let property_type: ThrushType = property.get_value_type()?.clone();

                if parser_context.match_token(TokenType::Eq)? {
                    let expr: ThrushStatement = self::build_expr(parser_context)?;

                    return Ok(ThrushStatement::Mut {
                        source: (None, Some(property.clone().into())),
                        value: expr.into(),
                        kind: ThrushType::Void,
                        cast_type: property_type,
                        span,
                    });
                }

                return Ok(property);
            }

            if parser_context.match_token(TokenType::ColonColon)? {
                return self::build_method_call(parser_context, name, span);
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

            self::build_reference(parser_context, name, span)?
        }

        TokenType::True => {
            ThrushStatement::new_boolean(ThrushType::Bool, 1, parser_context.advance()?.span)
        }

        TokenType::False => {
            ThrushStatement::new_boolean(ThrushType::Bool, 0, parser_context.advance()?.span)
        }

        TokenType::This => self::build_this(parser_context)?,
        TokenType::New => self::build_constructor(parser_context)?,

        TokenType::Asm => self::build_asm_code_block(parser_context)?,

        TokenType::Pass => ThrushStatement::Pass {
            span: parser_context.advance()?.get_span(),
        },

        _ => {
            let previous: &Token = parser_context.advance()?;

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
    parser_context: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

    let structure_id: &str = symbol.expected_struct(span)?;

    let structure: Struct = parser_context
        .get_symbols()
        .get_struct_by_id(structure_id, span)?;

    let method_tk: &Token = parser_context.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected method name."),
    )?;

    let method_name: &str = method_tk.get_lexeme();

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

    parser_context.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut args: Vec<ThrushStatement> = Vec::with_capacity(10);

    loop {
        if parser_context.check(TokenType::RParen) {
            break;
        }

        let expression: ThrushStatement = self::build_expr(parser_context)?;

        if expression.is_constructor() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Constructor should be stored in a local variable."),
                None,
                expression.get_span(),
            ));
        }

        args.push(expression);

        if parser_context.check(TokenType::RParen) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
        TokenType::RParen,
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
    parser_context: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let reference: ThrushStatement = self::build_reference(parser_context, name, span)?;
    let reference_type: ThrushType = reference.get_stmt_type()?.clone();

    let mut property_names: Vec<&'instr str> = Vec::with_capacity(10);

    let first_property: &Token = parser_context.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected property name."),
    )?;

    let mut span: Span = first_property.span;

    property_names.push(first_property.get_lexeme());

    while parser_context.match_token(TokenType::Dot)? {
        let property: &Token = parser_context.consume(
            TokenType::Identifier,
            String::from("Syntax error"),
            String::from("Expected property name."),
        )?;

        span = property.span;

        property_names.push(property.get_lexeme());
    }

    property_names.reverse();

    let decomposed: (ThrushType, Vec<(ThrushType, u32)>) = decompose_struct_property(
        0,
        property_names,
        reference_type,
        parser_context.get_symbols(),
        span,
    )?;

    Ok(ThrushStatement::Property {
        name,
        reference: reference.into(),
        indexes: decomposed.1,
        kind: decomposed.0,
        span,
    })
}

fn build_reference<'instr>(
    parser_context: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let symbol: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

    if symbol.is_constant() {
        let const_id: &str = symbol.expected_constant(span)?;

        let constant: ConstantSymbol = parser_context
            .get_symbols()
            .get_const_by_id(const_id, span)?;

        let constant_type: ThrushType = constant.get_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: constant_type,
            span,
            identificator: ReferenceIdentificator::Constant,
            is_mutable: false,
            is_allocated: true,
        });
    }

    if symbol.is_parameter() {
        let parameter_id: &str = symbol.expected_parameter(span)?;

        let parameter: ParameterSymbol = parser_context
            .get_symbols()
            .get_parameter_by_id(parameter_id, span)?;

        let parameter_type: ThrushType = parameter.get_type();

        let is_mutable: bool = parameter.is_mutable();

        let is_allocated: bool = parameter_type.is_mut_type()
            || parameter_type.is_ptr_type()
            || parameter_type.is_address_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: parameter_type,
            span,
            is_mutable,
            identificator: ReferenceIdentificator::FunctionParameter,
            is_allocated,
        });
    }

    if symbol.is_lli() {
        let lli_id: (&str, usize) = symbol.expected_lli(span)?;

        let lli_name: &str = lli_id.0;
        let scope_idx: usize = lli_id.1;

        let parameter: &LLISymbol = parser_context
            .get_symbols()
            .get_lli_by_id(lli_name, scope_idx, span)?;

        let lli_type: ThrushType = parameter.get_type();

        let is_allocated: bool = lli_type.is_ptr_type() || lli_type.is_address_type();

        return Ok(ThrushStatement::Reference {
            name,
            kind: lli_type,
            span,
            is_mutable: false,
            identificator: ReferenceIdentificator::LowLevelInstruction,
            is_allocated,
        });
    }

    let local_position: (&str, usize) = symbol.expected_local(span)?;

    let local: &LocalSymbol =
        parser_context
            .get_symbols()
            .get_local_by_id(local_position.0, local_position.1, span)?;

    let is_mutable: bool = local.is_mutable();

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
        is_mutable,
        identificator: ReferenceIdentificator::Local,
        is_allocated: true,
    };

    if parser_context.match_token(TokenType::PlusPlus)?
        | parser_context.match_token(TokenType::MinusMinus)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

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
    parser_context: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;
    let enum_id: &str = object.expected_enum(span)?;

    let union: EnumFields = parser_context
        .get_symbols()
        .get_enum_by_id(enum_id, span)?
        .get_fields();

    let field_tk: &Token = parser_context.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected enum name."),
    )?;

    let field_name: &str = field_tk.get_lexeme();

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

fn build_function_call<'instr>(
    parser_context: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

    let function_type: ThrushType = if object.is_function_asm() {
        let asm_function_id: &str = object.expected_asm_function(span)?;
        let asm_function: AssemblerFunction = parser_context
            .get_symbols()
            .get_asm_function_by_id(span, asm_function_id)?;

        asm_function.get_type()
    } else {
        let function_id: &str = object.expected_function(span)?;
        let function: Function = parser_context
            .get_symbols()
            .get_function_by_id(span, function_id)?;

        function.get_type()
    };

    let mut args: Vec<ThrushStatement> = Vec::with_capacity(10);

    loop {
        if parser_context.check(TokenType::RParen) {
            break;
        }

        let expression: ThrushStatement = self::build_expr(parser_context)?;

        if expression.is_constructor() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Constructor should be stored in a local variable."),
                None,
                expression.get_span(),
            ));
        }

        args.push(expression);

        if parser_context.check(TokenType::RParen) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
        TokenType::RParen,
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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let this_tk: &Token = parser_context.consume(
        TokenType::This,
        String::from("Syntax error"),
        String::from("Expected 'this' keyword."),
    )?;

    let span: Span = this_tk.get_span();

    if !parser_context
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

    if !parser_context
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

    if !parser_context.get_type_ctx().get_bind_instance() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from(
                "Expected that 'this' was already declared within the definition of the a previous bind parameter.",
            ),
            None,
            span,
        ));
    }

    if parser_context.match_token(TokenType::Dot)? {
        return build_property(parser_context, "this", span);
    }

    let this_type: ThrushType = parser_context
        .get_type_ctx()
        .get_this_methods_type()
        .dissamble();

    let is_mutable: bool = parser_context.match_token(TokenType::Mut)?;

    Ok(ThrushStatement::This {
        kind: this_type,
        is_mutable,
        span,
    })
}

fn build_constructor<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let new_tk: &Token = parser_context.consume(
        TokenType::New,
        String::from("Syntax error"),
        String::from("Expected 'new' keyword."),
    )?;

    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            new_tk.span,
        ));
    }

    let name: &Token = parser_context.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected structure reference."),
    )?;

    let struct_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let struct_found: Struct = parser_context.get_symbols().get_struct(struct_name, span)?;
    let fields_required: usize = struct_found.get_fields().1.len();

    parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut arguments: Constructor = (struct_name, Vec::with_capacity(10));

    let mut amount: usize = 0;

    loop {
        if parser_context.check(TokenType::RBrace) {
            break;
        }

        if parser_context.match_token(TokenType::Identifier)? {
            let field_tk: &Token = parser_context.previous();
            let field_span: Span = field_tk.span;
            let field_name: &str = field_tk.get_lexeme();

            parser_context.consume(
                TokenType::Colon,
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

            let expression: ThrushStatement = self::build_expr(parser_context)?;

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

            if parser_context.check(TokenType::RBrace) {
                break;
            }

            if parser_context.match_token(TokenType::Comma)? {
                if parser_context.check(TokenType::RBrace) {
                    break;
                }
            } else if parser_context.check_to(TokenType::Identifier, 0) {
                parser_context.consume(
                    TokenType::Comma,
                    String::from("Syntax error"),
                    String::from("Expected ','."),
                )?;
            } else {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected identifier."),
                    None,
                    parser_context.previous().get_span(),
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

    parser_context.consume(
        TokenType::RBrace,
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
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let initial_deref_tk: &Token = parser_context.advance()?;
    let span: Span = initial_deref_tk.span;

    let mut deref_count: u64 = 1;

    let mut current_expr: ThrushStatement = {
        while parser_context.check(TokenType::Deref) {
            parser_context.consume(
                TokenType::Deref,
                "Syntax error".into(),
                "Expected 'deref'.".into(),
            )?;
            deref_count += 1;
        }

        let expr: ThrushStatement = expression::build_expr(parser_context)?;

        expr
    };

    let mut current_type: ThrushType = current_expr.get_value_type()?.clone();

    for _ in 0..deref_count {
        current_expr = ThrushStatement::Deref {
            value: current_expr.clone().into(),
            kind: current_type.deref(),
            span,
        };

        current_type = current_type.deref();
    }

    Ok(current_expr)
}

fn build_asm_code_block<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let asm_tk: &Token = parser_context.consume(
        TokenType::Asm,
        String::from("Syntax error"),
        String::from("Expected 'asm' keyword."),
    )?;

    let span: Span = asm_tk.get_span();

    let mut args: Vec<ThrushStatement> = Vec::with_capacity(10);

    let attributes: ThrushAttributes =
        stmt::build_attributes(parser_context, &[TokenType::LParen, TokenType::LBrace])?;

    if parser_context.match_token(TokenType::LParen)? {
        loop {
            if parser_context.check(TokenType::RParen) {
                break;
            }

            let expr: ThrushStatement = self::build_expression(parser_context)?;

            if expr.is_constructor() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Constructor should be stored in a local variable."),
                    None,
                    expr.get_span(),
                ));
            }

            args.push(expr);

            if parser_context.check(TokenType::RParen) {
                break;
            } else {
                parser_context.consume(
                    TokenType::Colon,
                    String::from("Syntax error"),
                    String::from("Expected ','."),
                )?;
            }
        }

        parser_context.consume(
            TokenType::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;
    }

    parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut assembler: String = String::with_capacity(100);
    let mut assembler_pos: usize = 0;

    loop {
        if parser_context.check(TokenType::RBrace) {
            break;
        }

        let raw_str: ThrushStatement = self::build_expr(parser_context)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let assembly: &str = raw_str.get_str_content()?;

        if assembler_pos != 0 {
            assembler.push('\n');
        }

        assembler.push_str(assembly);

        if parser_context.check(TokenType::RBrace) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }

        assembler_pos += 1;
    }

    parser_context.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut constraints: String = String::with_capacity(100);
    let mut constraint_pos: usize = 0;

    loop {
        if parser_context.check(TokenType::RBrace) {
            break;
        }

        let raw_str: ThrushStatement = self::build_expr(parser_context)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let constraint: &str = raw_str.get_str_content()?;

        if constraint_pos != 0 {
            constraints.push('\n');
        }

        constraints.push_str(constraint);

        if parser_context.check(TokenType::RBrace) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }

        constraint_pos += 1;
    }

    parser_context.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    let asm_type: ThrushType = typegen::build_type(parser_context)?;

    Ok(ThrushStatement::AsmValue {
        assembler,
        constraints,
        args,
        kind: asm_type,
        attributes,
        span,
    })
}

fn build_array<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let array_start_tk: &Token = parser_context.consume(
        TokenType::LBracket,
        String::from("Syntax error"),
        String::from("Expected '['."),
    )?;

    let span: Span = array_start_tk.get_span();

    let mut array_type: ThrushType = ThrushType::Void;
    let mut items: Vec<ThrushStatement> = Vec::with_capacity(100);

    loop {
        if parser_context.check(TokenType::RBracket) {
            break;
        }

        let item: ThrushStatement = self::build_expr(parser_context)?;

        if item.is_constructor() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Constructor should be stored in a local variable."),
                None,
                item.get_span(),
            ));
        }

        items.push(item);

        if parser_context.check(TokenType::RBracket) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
        TokenType::RBracket,
        String::from("Syntax error"),
        String::from("Expected ']'."),
    )?;

    if let Some(item) = items.iter().max_by(|x, y| {
        let x_type: &ThrushType = x.get_value_type().unwrap_or(&ThrushType::Void);
        let y_type: &ThrushType = y.get_value_type().unwrap_or(&ThrushType::Void);

        x_type
            .get_fixed_array_type_herarchy()
            .cmp(&y_type.get_fixed_array_type_herarchy())
    }) {
        if let Ok(size) = u32::try_from(items.len()) {
            array_type = ThrushType::FixedArray(item.get_value_type()?.clone().into(), size)
        } else {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "The size limit of an array was exceeded.".into(),
                None,
                span,
            ));
        }
    }

    Ok(ThrushStatement::Array {
        items,
        kind: array_type,
        span,
    })
}

pub fn build_index<'instr>(
    parser_context: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let reference: ThrushStatement = self::build_reference(parser_context, name, span)?;
    let reference_type: &ThrushType = reference.get_value_type()?;

    let mut indexes: Vec<ThrushStatement> = Vec::with_capacity(50);

    loop {
        if parser_context.check(TokenType::RBracket) {
            break;
        }

        let indexe: ThrushStatement = expression::build_expr(parser_context)?;

        indexes.push(indexe);

        if parser_context.check(TokenType::RBracket) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
        TokenType::RBracket,
        String::from("Syntax error"),
        String::from("Expected ']'."),
    )?;

    let mut index_type: ThrushType = ThrushType::Mut(
        reference_type
            .get_type_with_depth(indexes.len())
            .clone()
            .into(),
    );

    if parser_context.match_token(TokenType::LParen)? {
        index_type = typegen::build_type(parser_context)?;

        parser_context.consume(
            TokenType::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;
    }

    Ok(ThrushStatement::Index {
        name,
        reference: reference.into(),
        indexes,
        kind: index_type,
        span,
    })
}

fn build_address_indexes<'instr>(
    parser_context: &mut ParserContext<'instr>,
    span: Span,
) -> Result<Vec<ThrushStatement<'instr>>, ThrushCompilerIssue> {
    parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut indexes: Vec<ThrushStatement> = Vec::with_capacity(10);

    loop {
        if parser_context.check(TokenType::RBrace) {
            break;
        }

        let index: ThrushStatement = self::build_expr(parser_context)?;

        indexes.push(index);

        if parser_context.check(TokenType::RBrace) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
        TokenType::RBrace,
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

    Ok(indexes)
}
