use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, checks, expr, typegen},
        types::{
            ast::Ast,
            parser::stmts::{
                traits::TokenExtensions,
                types::{EnumFields, ThrushAttributes},
            },
        },
        typesystem::types::Type,
    },
};

pub fn build_enum<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

    ctx.consume(
        TokenType::Enum,
        String::from("Syntax error"),
        String::from("Expected 'enum'."),
    )?;

    let name: &Token = ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected enum name."),
    )?;

    let enum_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let enum_attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::LBrace])?;

    ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut enum_fields: EnumFields = Vec::with_capacity(10);

    let mut default_float_value: f64 = 0.0;
    let mut default_integer_value: u64 = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        if ctx.match_token(TokenType::Identifier)? {
            let field_tk: &Token = ctx.previous();

            let name: &str = field_tk.get_lexeme();
            let span: Span = field_tk.get_span();

            ctx.consume(
                TokenType::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            let field_type: Type = typegen::build_type(ctx)?;

            if !field_type.is_numeric() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected integer, boolean, char or floating-point types."),
                    None,
                    span,
                ));
            }

            if ctx.match_token(TokenType::SemiColon)? {
                let field_value: Ast = if field_type.is_integer_type() {
                    Ast::new_integer(field_type, default_integer_value, false, span)
                } else if field_type.is_float_type() {
                    Ast::new_float(field_type, default_float_value, false, span)
                } else if field_type.is_bool_type() {
                    Ast::new_boolean(field_type, default_integer_value, span)
                } else if field_type.is_char_type() {
                    if default_integer_value > char::MAX as u64 {
                        return Err(ThrushCompilerIssue::Error(
                            "Syntax error".into(),
                            "Char overflow.".into(),
                            None,
                            span,
                        ));
                    }

                    Ast::new_char(field_type, default_integer_value, span)
                } else {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected integer, boolean, char or floating-point types.".into(),
                        None,
                        span,
                    ));
                };

                enum_fields.push((name, field_value));

                default_float_value += 1.0;
                default_integer_value += 1;

                continue;
            }

            ctx.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

            let expression: Ast = expr::build_expr(ctx)?;
            let expression_type: &Type = expression.get_value_type()?;
            let expression_span: Span = expression.get_span();

            if !expression_type.is_integer_type()
                && !expression_type.is_float_type()
                && !expression_type.is_bool_type()
                && !expression_type.is_char_type()
            {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected integer, boolean, char or floating-point types.".into(),
                    None,
                    expression_span,
                ));
            }

            ctx.consume(
                TokenType::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            enum_fields.push((name, expression));

            continue;
        }

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Expected identifier in enum field.".into(),
            None,
            ctx.advance()?.get_span(),
        ));
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    if declare_forward {
        ctx.get_mut_symbols()
            .new_enum(enum_name, (enum_fields, enum_attributes), span)?;

        return Ok(Ast::new_nullptr(span));
    }

    Ok(Ast::Enum {
        name: enum_name,
        fields: enum_fields,
        attributes: enum_attributes,
        span,
    })
}
