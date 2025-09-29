use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{
            ParserContext, builtins, expr,
            expressions::{
                array, asm, call, constructor, deref, enumv, farray, index, indirect, lli,
                property, reference, sizeof,
            },
            parse,
        },
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn lower_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let primary: Ast = match &ctx.peek().kind {
        TokenType::New => constructor::build_constructor(ctx)?,

        TokenType::Fixed => farray::build_fixed_array(ctx)?,
        TokenType::LBracket => array::build_array(ctx)?,
        TokenType::Deref => deref::build_dereference(ctx)?,

        TokenType::SizeOf => sizeof::build_sizeof(ctx)?,

        TokenType::Halloc => builtins::build_halloc(ctx)?,
        TokenType::MemSet => builtins::build_memset(ctx)?,
        TokenType::MemMove => builtins::build_memmove(ctx)?,
        TokenType::MemCpy => builtins::build_memcpy(ctx)?,

        TokenType::AlignOf => builtins::build_alignof(ctx)?,

        TokenType::Asm => asm::build_asm_code_block(ctx)?,

        TokenType::Alloc => lli::alloc::build_alloc(ctx)?,
        TokenType::Load => lli::load::build_load(ctx)?,
        TokenType::Write => lli::write::build_write(ctx)?,
        TokenType::Address => lli::address::build_address(ctx)?,

        TokenType::Indirect => indirect::build_indirect(ctx)?,

        TokenType::LParen => {
            let span: Span = ctx.advance()?.get_span();

            let expression: Ast = expr::build_expr(ctx)?;

            let expression_type: &Type = expression.get_value_type()?;

            ctx.consume(
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
            let str_tk: &Token = ctx.advance()?;
            let span: Span = str_tk.get_span();

            let bytes: Vec<u8> = str_tk.scape(span)?;

            Ast::new_str(bytes, Type::Str, span)
        }

        TokenType::Char => {
            let char_tk: &Token = ctx.advance()?;
            let span: Span = char_tk.get_span();

            Ast::new_char(Type::Char, char_tk.get_lexeme_first_byte(), span)
        }

        TokenType::NullPtr => Ast::NullPtr {
            span: ctx.advance()?.span,
        },

        TokenType::Integer => {
            let integer_tk: &Token = ctx.advance()?;
            let integer: &str = integer_tk.get_lexeme();
            let span: Span = integer_tk.get_span();

            let parsed_integer: (Type, u64) = parse::integer(integer, span)?;

            let integer_type: Type = parsed_integer.0;
            let integer_value: u64 = parsed_integer.1;

            Ast::new_integer(integer_type, integer_value, false, span)
        }

        TokenType::Float => {
            let float_tk: &Token = ctx.advance()?;

            let float: &str = float_tk.get_lexeme();
            let span: Span = float_tk.get_span();

            let parsed_float: (Type, f64) = parse::float(float, span)?;

            let float_type: Type = parsed_float.0;
            let float_value: f64 = parsed_float.1;

            Ast::new_float(float_type, float_value, false, span)
        }

        TokenType::Identifier => {
            let identifier_tk: &Token = ctx.advance()?;

            let name: &str = identifier_tk.get_lexeme();
            let span: Span = identifier_tk.get_span();

            if ctx.match_token(TokenType::LBracket)? {
                let reference: Ast = reference::build_reference(ctx, name, span)?;

                let index: Ast =
                    index::build_index(ctx, (Some((name, reference.into())), None), span)?;

                return Ok(index);
            }

            if ctx.match_token(TokenType::Arrow)? {
                return enumv::build_enum_value(ctx, name, span);
            }

            if ctx.match_token(TokenType::LParen)? {
                return call::build_call(ctx, name, span);
            }

            if ctx.match_token(TokenType::Dot)? {
                let reference: Ast = reference::build_reference(ctx, name, span)?;

                let property: Ast =
                    property::build_property(ctx, (Some((name, reference.into())), None), span)?;

                if ctx.match_token(TokenType::LBracket)? {
                    return index::build_index(ctx, (None, Some(property.into())), span);
                }

                return Ok(property);
            }

            reference::build_reference(ctx, name, span)?
        }

        TokenType::True => Ast::new_boolean(Type::Bool, 1, ctx.advance()?.span),
        TokenType::False => Ast::new_boolean(Type::Bool, 0, ctx.advance()?.span),

        TokenType::Pass => Ast::Pass {
            span: ctx.advance()?.get_span(),
        },
        TokenType::Unreachable => Ast::Unreachable {
            span: ctx.advance()?.get_span(),
        },

        _ => {
            let previous: &Token = ctx.advance()?;

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
