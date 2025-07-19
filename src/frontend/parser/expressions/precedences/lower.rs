use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{
            ParserContext, builtins, expr,
            expressions::{
                array, asm, call, constructor, deref, enumv, farray, index, lli, property,
                reference, sizeof,
            },
            parse,
        },
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn lower_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let primary: Ast = match &parser_context.peek().kind {
        TokenType::New => constructor::build_constructor(parser_context)?,

        TokenType::Fixed => farray::build_fixed_array(parser_context)?,
        TokenType::LBracket => array::build_array(parser_context)?,
        TokenType::Deref => deref::build_dereference(parser_context)?,

        TokenType::SizeOf => sizeof::build_sizeof(parser_context)?,

        TokenType::Halloc => builtins::build_halloc(parser_context)?,
        TokenType::MemSet => builtins::build_memset(parser_context)?,
        TokenType::MemMove => builtins::build_memmove(parser_context)?,
        TokenType::MemCpy => builtins::build_memcpy(parser_context)?,

        TokenType::AlignOf => builtins::build_alignof(parser_context)?,

        TokenType::Asm => asm::build_asm_code_block(parser_context)?,

        TokenType::Alloc => lli::alloc::build_alloc(parser_context)?,
        TokenType::Load => lli::load::build_load(parser_context)?,
        TokenType::Write => lli::write::build_write(parser_context)?,
        TokenType::Address => lli::address::build_address(parser_context)?,

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
                    (Some((name, reference.into())), None),
                    span,
                )?;

                if parser_context.match_token(TokenType::LBracket)? {
                    return index::build_index(parser_context, (None, Some(property.into())), span);
                }

                return Ok(property);
            }

            reference::build_reference(parser_context, name, span)?
        }

        TokenType::True => Ast::new_boolean(Type::Bool, 1, parser_context.advance()?.span),
        TokenType::False => Ast::new_boolean(Type::Bool, 0, parser_context.advance()?.span),

        TokenType::Pass => Ast::Pass {
            span: parser_context.advance()?.get_span(),
        },
        TokenType::Unreachable => Ast::Unreachable {
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
