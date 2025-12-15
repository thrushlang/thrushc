use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::builtins;
use crate::front_end::parser::expr;
use crate::front_end::parser::expressions::array;
use crate::front_end::parser::expressions::asm;
use crate::front_end::parser::expressions::call;
use crate::front_end::parser::expressions::constructor;
use crate::front_end::parser::expressions::deref;
use crate::front_end::parser::expressions::enumv;
use crate::front_end::parser::expressions::farray;
use crate::front_end::parser::expressions::indirect;
use crate::front_end::parser::expressions::lli;
use crate::front_end::parser::expressions::reference;
use crate::front_end::parser::interpret;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::lexer::traits::TokenTypeBuiltinExtensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::traits::TypeExtensions;
use crate::front_end::typesystem::types::Type;

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

        TokenType::Alloc => lli::alloc::build_alloc(ctx)?,
        TokenType::Load => lli::load::build_load(ctx)?,
        TokenType::Write => lli::write::build_write(ctx)?,
        TokenType::Address => lli::address::build_address(ctx)?,

        TokenType::Indirect => indirect::build_indirect(ctx)?,

        TokenType::LParen => {
            let span: Span = ctx.advance()?.get_span();

            let expr: Ast = expr::build_expr(ctx)?;
            let expr_type: &Type = expr.get_value_type()?;

            ctx.consume(
                TokenType::RParen,
                "Syntax error".into(),
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

            let bytes: Vec<u8> = lexer::scapes::parse_scapes(content, span)?;

            Ast::new_str(
                bytes,
                Type::Const(Type::Ptr(Some(Type::Array(Type::Char.into()).into())).into()),
                span,
            )
        }

        TokenType::Char => {
            let tk: &Token = ctx.advance()?;

            let span: Span = tk.get_span();

            Ast::new_char(Type::Char, tk.get_lexeme_first_byte(), span)
        }

        TokenType::NullPtr => Ast::new_nullptr(ctx.advance()?.span),

        TokenType::Integer => {
            let tk: &Token = ctx.advance()?;

            let integer: &str = tk.get_lexeme();
            let span: Span = tk.get_span();

            let parsed_integer: (Type, u64) = interpret::integer(integer, span)?;

            let integer_type: Type = parsed_integer.0;
            let integer_value: u64 = parsed_integer.1;

            Ast::new_integer(integer_type, integer_value, false, span)
        }

        TokenType::Float => {
            let tk: &Token = ctx.advance()?;

            let float: &str = tk.get_lexeme();
            let span: Span = tk.get_span();

            let parsed_float: (Type, f64) = interpret::float(float, span)?;

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

            let expr: Ast = expr::build_expr(ctx)?;
            let expr_type: &Type = expr.get_value_type()?;

            Ast::DirectRef {
                expr: expr.clone().into(),
                kind: expr_type.get_type_ref(),
                span,
            }
        }

        TokenType::True => Ast::new_boolean(Type::Bool, 1, ctx.advance()?.get_span()),
        TokenType::False => Ast::new_boolean(Type::Bool, 0, ctx.advance()?.get_span()),
        TokenType::Pass => Ast::Pass {
            span: ctx.advance()?.get_span(),
        },
        TokenType::Unreachable => Ast::Unreachable {
            span: ctx.advance()?.get_span(),
        },

        _ => {
            ctx.set_force_abort();

            let previous: &Token = ctx.advance()?;

            return Err(CompilationIssue::Error(
                "Syntax error".into(),
                format!("Expression '{}' don't allowed.", previous.lexeme),
                None,
                previous.span,
            ));
        }
    };

    Ok(primary)
}
