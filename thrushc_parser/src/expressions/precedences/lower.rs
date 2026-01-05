use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{
    Token,
    tokentype::TokenType,
    traits::{TokenExtensions, TokenTypeBuiltinExtensions},
};
use thrushc_typesystem::{Type, traits::TypeExtensions};

use crate::{
    ParserContext, builtins,
    expressions::{self, array, asm, call, constructor, deref, enumv, farray, indirect, reference},
    reinterpret,
};

pub fn lower_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let primary: Ast = match &ctx.peek().kind {
        TokenType::New => constructor::build_constructor(ctx)?,

        TokenType::Fixed => farray::build_fixed_array(ctx)?,
        TokenType::LBracket => array::build_array(ctx)?,
        TokenType::Deref => deref::build_dereference(ctx)?,

        tk_type if tk_type.is_builtin() => builtins::build_builtin(ctx, *tk_type)?,

        TokenType::Asm => asm::build_asm_code_block(ctx)?,
        TokenType::Indirect => indirect::build_indirect(ctx)?,

        TokenType::LParen => {
            let span: Span = ctx.advance()?.get_span();

            let expr: Ast = expressions::build_expr(ctx)?;
            let expr_type: &Type = expr.get_value_type()?;

            ctx.consume(
                TokenType::RParen,
                CompilationIssueCode::E0001,
                "Expected ')'.".into(),
            )?;

            return Ok(Ast::Group {
                expression: expr.clone().into(),
                kind: expr_type.clone(),
                span,
            });
        }

        TokenType::Str => {
            let tk: &Token = ctx.advance()?;

            let content: &str = tk.get_lexeme();
            let span: Span = tk.get_span();

            let source: &[u8] = content.as_bytes();

            let mut processed: Vec<u8> = Vec::with_capacity(source.len());
            let mut idx: usize = 0;

            while idx < source.len() {
                if let Some(byte) = source.get(idx) {
                    if *byte == b'\\' {
                        idx += 1;

                        match source.get(idx) {
                            Some(b'n') => processed.push(b'\n'),
                            Some(b't') => processed.push(b'\t'),
                            Some(b'r') => processed.push(b'\r'),
                            Some(b'\\') => processed.push(b'\\'),
                            Some(b'0') => processed.push(b'\0'),
                            Some(b'\'') => processed.push(b'\''),
                            Some(b'"') => processed.push(b'"'),

                            _ => (),
                        }

                        idx += 1;
                        continue;
                    }

                    processed.push(source[idx]);

                    idx += 1;
                }
            }

            Ast::new_str(
                processed,
                Type::Const(
                    Type::Ptr(
                        Some(Type::Array(Type::Char(span).into(), span).into()),
                        span,
                    )
                    .into(),
                    span,
                ),
                span,
            )
        }

        TokenType::Char => {
            let tk: &Token = ctx.advance()?;
            let span: Span = tk.get_span();

            Ast::new_char(Type::Char(span), tk.get_lexeme_first_byte(), span)
        }

        TokenType::NullPtr => Ast::new_nullptr(ctx.advance()?.span),

        TokenType::Integer => {
            let tk: &Token = ctx.advance()?;

            let integer: &str = tk.get_lexeme();
            let span: Span = tk.get_span();

            let parsed_integer: (Type, u64) = reinterpret::integer(integer, span)?;

            let integer_type: Type = parsed_integer.0;
            let integer_value: u64 = parsed_integer.1;

            Ast::new_integer(integer_type, integer_value, false, span)
        }

        TokenType::Float => {
            let tk: &Token = ctx.advance()?;

            let float: &str = tk.get_lexeme();
            let span: Span = tk.get_span();

            let parsed_float: (Type, f64) = reinterpret::floating_point(float, span)?;

            let float_type: Type = parsed_float.0;
            let float_value: f64 = parsed_float.1;

            Ast::new_float(float_type, float_value, false, span)
        }

        TokenType::Identifier => {
            let tk: &Token = ctx.advance()?;

            let name: &str = tk.get_lexeme();
            let span: Span = tk.get_span();

            if ctx.match_token(TokenType::Arrow)? {
                return enumv::build_enum_value(ctx, name, span);
            }

            if ctx.match_token(TokenType::LParen)? {
                return call::build_call(ctx, name, span);
            }

            reference::build_reference(ctx, name, span)?
        }

        TokenType::DirectRef => {
            let span: Span = ctx.advance()?.get_span();

            let expr: Ast = expressions::build_expr(ctx)?;
            let expr_type: &Type = expr.get_value_type()?;

            Ast::DirectRef {
                expr: expr.clone().into(),
                kind: expr_type.get_type_ref(),
                span,
            }
        }

        TokenType::True => {
            let span: Span = ctx.advance()?.get_span();
            Ast::new_boolean(Type::Bool(span), 1, span)
        }
        TokenType::False => {
            let span: Span = ctx.advance()?.get_span();
            Ast::new_boolean(Type::Bool(span), 0, span)
        }
        TokenType::Unreachable => {
            let span: Span = ctx.advance()?.get_span();
            Ast::Unreachable {
                span,
                kind: Type::Void(span),
            }
        }

        _ => {
            let previous: &Token = ctx.advance()?;
            let span: Span = previous.get_span();

            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                format!(
                    "Expression or statement '{}' don't allowed in this point.",
                    previous.get_lexeme()
                ),
                None,
                span,
            ));

            return Ok(Ast::invalid_ast(span));
        }
    };

    Ok(primary)
}
