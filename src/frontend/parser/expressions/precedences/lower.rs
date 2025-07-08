use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{
            ParserContext, attributes, builtins, expr,
            expressions::{
                address, array, asm, call, constructor, deref, enumv, farray, index, property,
                reference, sizeof,
            },
            parse, typegen,
        },
        types::{
            ast::Ast,
            parser::{
                stmts::{
                    sites::AllocationSite,
                    traits::{FoundSymbolExtension, TokenExtensions},
                },
                symbols::types::FoundSymbolId,
            },
        },
        typesystem::types::Type,
    },
};

pub fn lower_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let primary: Ast = match &parser_context.peek().kind {
        TokenType::Fixed => farray::build_fixed_array(parser_context)?,
        TokenType::LBracket => array::build_array(parser_context)?,
        TokenType::Deref => deref::build_dereference(parser_context)?,

        TokenType::New => constructor::build_constructor(parser_context)?,

        TokenType::SizeOf => sizeof::build_sizeof(parser_context)?,

        TokenType::Halloc => builtins::build_halloc(parser_context)?,
        TokenType::MemSet => builtins::build_memset(parser_context)?,
        TokenType::MemMove => builtins::build_memmove(parser_context)?,
        TokenType::MemCpy => builtins::build_memcpy(parser_context)?,

        TokenType::AlignOf => builtins::build_alignof(parser_context)?,

        TokenType::Asm => asm::build_asm_code_block(parser_context)?,

        TokenType::Alloc => {
            let alloc_tk: &Token = parser_context.advance()?;
            let span: Span = alloc_tk.get_span();

            let site_allocation: AllocationSite = match parser_context.peek().kind {
                TokenType::Heap => {
                    parser_context.only_advance()?;
                    AllocationSite::Heap
                }
                TokenType::Stack => {
                    parser_context.only_advance()?;
                    AllocationSite::Stack
                }
                TokenType::Static => {
                    parser_context.only_advance()?;
                    AllocationSite::Static
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

            let mut alloc_type: Type = typegen::build_type(parser_context)?;

            alloc_type = Type::Ptr(Some(alloc_type.into()));

            let attributes: Vec<LLVMAttribute> = if !parser_context.check(TokenType::RBrace) {
                attributes::build_attributes(
                    parser_context,
                    &[TokenType::RBrace, TokenType::SemiColon],
                )?
            } else {
                Vec::new()
            };

            parser_context.consume(
                TokenType::RBrace,
                "Syntax error".into(),
                "Expected '}'.".into(),
            )?;

            Ast::Alloc {
                type_to_alloc: alloc_type,
                site_allocation,
                attributes,
                span,
            }
        }

        TokenType::Load => {
            let load_tk: &Token = parser_context.advance()?;
            let span: Span = load_tk.get_span();

            let load_type: Type = typegen::build_type(parser_context)?;

            parser_context.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;

            if parser_context.check(TokenType::Identifier) {
                let identifier_tk: &Token = parser_context.consume(
                    TokenType::Identifier,
                    "Syntax error".into(),
                    "Expected 'identifier'.".into(),
                )?;

                let reference_name: &str = identifier_tk.get_lexeme();

                let reference: Ast =
                    reference::build_reference(parser_context, reference_name, span)?;

                return Ok(Ast::Load {
                    source: (Some((reference_name, reference.into())), None),
                    kind: load_type,
                    span,
                });
            }

            let expression: Ast = expr::build_expr(parser_context)?;

            Ast::Load {
                source: (None, Some(expression.into())),
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

                let reference: Ast = reference::build_reference(parser_context, name, span)?;

                parser_context.consume(
                    TokenType::Comma,
                    "Syntax error".into(),
                    "Expected ','.".into(),
                )?;

                let write_type: Type = typegen::build_type(parser_context)?;

                let value: Ast = expr::build_expr(parser_context)?;

                return Ok(Ast::Write {
                    source: (Some((name, reference.into())), None),
                    write_value: value.clone().into(),
                    write_type,
                    span,
                });
            }

            let expression: Ast = expr::build_expr(parser_context)?;

            parser_context.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;

            let write_type: Type = typegen::build_type(parser_context)?;
            let value: Ast = expr::build_expr(parser_context)?;

            Ast::Write {
                source: (None, Some(expression.into())),
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

                let reference: Ast = reference::build_reference(parser_context, name, span)?;

                let indexes: Vec<Ast> = address::build_address_indexes(parser_context, span)?;

                return Ok(Ast::Address {
                    source: (Some((name, reference.into())), None),
                    indexes,
                    kind: Type::Addr,
                    span: address_span,
                });
            }

            let expr: Ast = expr::build_expr(parser_context)?;
            let expr_span: Span = expr.get_span();

            let indexes: Vec<Ast> = address::build_address_indexes(parser_context, expr_span)?;

            return Ok(Ast::Address {
                source: (None, Some(expr.into())),
                indexes,
                kind: Type::Addr,
                span: address_span,
            });
        }

        TokenType::LParen => {
            let span: Span = parser_context.advance()?.get_span();

            let expression: Ast = expr::build_expr(parser_context)?;

            let expression_type: &Type = expression.get_value_type()?;

            parser_context.consume(
                TokenType::RParen,
                "Syntax error".into(),
                "Expected ')'.".into(),
            )?;

            return Ok(Ast::Group {
                expression: expression.clone().into(),
                kind: expression_type.clone(),
                span,
            });
        }

        TokenType::Str => {
            let str_tk: &Token = parser_context.advance()?;
            let span: Span = str_tk.get_span();

            let bytes: Vec<u8> = str_tk.fix_lexeme_scapes(span)?;

            Ast::new_str(bytes, Type::Str, span)
        }

        TokenType::Char => {
            let char_tk: &Token = parser_context.advance()?;
            let span: Span = char_tk.get_span();

            Ast::new_char(Type::Char, char_tk.get_lexeme_first_byte(), span)
        }

        TokenType::NullPtr => Ast::NullPtr {
            span: parser_context.advance()?.span,
        },

        TokenType::Integer => {
            let integer_tk: &Token = parser_context.advance()?;
            let integer: &str = integer_tk.get_lexeme();
            let span: Span = integer_tk.get_span();

            let parsed_integer: (Type, u64) = parse::integer(integer, span)?;

            let integer_type: Type = parsed_integer.0;
            let integer_value: u64 = parsed_integer.1;

            Ast::new_integer(integer_type, integer_value, false, span)
        }

        TokenType::Float => {
            let float_tk: &Token = parser_context.advance()?;

            let float: &str = float_tk.get_lexeme();
            let span: Span = float_tk.get_span();

            let parsed_float: (Type, f64) = parse::float(float, span)?;

            let float_type: Type = parsed_float.0;
            let float_value: f64 = parsed_float.1;

            Ast::new_float(float_type, float_value, false, span)
        }

        TokenType::Identifier => {
            let identifier_tk: &Token = parser_context.advance()?;

            let name: &str = identifier_tk.get_lexeme();
            let span: Span = identifier_tk.get_span();

            let symbol: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

            if parser_context.match_token(TokenType::LBracket)? {
                let reference: Ast = reference::build_reference(parser_context, name, span)?;

                let index: Ast = index::build_index(
                    parser_context,
                    (Some((name, reference.into())), None),
                    span,
                )?;

                return Ok(index);
            }

            if parser_context.match_token(TokenType::Arrow)? {
                return enumv::build_enum_value(parser_context, name, span);
            }

            if parser_context.match_token(TokenType::LParen)? {
                return call::build_call(parser_context, name, span);
            }

            if parser_context.match_token(TokenType::Dot)? {
                let reference: Ast = reference::build_reference(parser_context, name, span)?;

                let property: Ast = property::build_property(
                    parser_context,
                    (Some((name, reference.clone().into())), None),
                    span,
                )?;

                if parser_context.match_token(TokenType::LBracket)? {
                    return index::build_index(parser_context, (None, Some(property.into())), span);
                }

                return Ok(property);
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

            reference::build_reference(parser_context, name, span)?
        }

        TokenType::True => Ast::new_boolean(Type::Bool, 1, parser_context.advance()?.span),
        TokenType::False => Ast::new_boolean(Type::Bool, 0, parser_context.advance()?.span),

        TokenType::Pass => Ast::Pass {
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
