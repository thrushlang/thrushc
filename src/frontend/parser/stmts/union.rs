use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, expression, typegen},
        types::{
            ast::Ast,
            lexer::ThrushType,
            parser::stmts::{
                traits::TokenExtensions,
                types::{EnumFields, ThrushAttributes},
            },
        },
    },
};

pub fn build_enum<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let enum_tk: &Token = parser_ctx.consume(
        TokenType::Enum,
        String::from("Syntax error"),
        String::from("Expected 'enum'."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Enums are only defined globally."),
            None,
            enum_tk.get_span(),
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected enum name."),
    )?;

    let enum_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let enum_attributes: ThrushAttributes =
        attributes::build_attributes(parser_ctx, &[TokenType::LBrace])?;

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut enum_fields: EnumFields = Vec::with_capacity(10);

    let mut default_float_value: f64 = 0.0;
    let mut default_integer_value: u64 = 0;

    loop {
        if parser_ctx.check(TokenType::RBrace) {
            break;
        }

        if parser_ctx.match_token(TokenType::Identifier)? {
            let field_tk: &Token = parser_ctx.previous();

            let name: &str = field_tk.get_lexeme();
            let span: Span = field_tk.get_span();

            parser_ctx.consume(
                TokenType::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            let field_type: ThrushType = typegen::build_type(parser_ctx)?;

            if !field_type.is_numeric() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected integer, boolean, char or floating-point types."),
                    None,
                    span,
                ));
            }

            if parser_ctx.match_token(TokenType::SemiColon)? {
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

            parser_ctx.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

            let expression: Ast = expression::build_expr(parser_ctx)?;
            let expression_type: &ThrushType = expression.get_value_type()?;
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

            parser_ctx.consume(
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
            parser_ctx.advance()?.get_span(),
        ));
    }

    parser_ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    parser_ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    if declare_forward {
        if let Err(error) =
            parser_ctx
                .get_mut_symbols()
                .new_enum(enum_name, (enum_fields, enum_attributes), span)
        {
            parser_ctx.add_error(error);
        }

        return Ok(Ast::Null { span });
    }

    Ok(Ast::Enum {
        name: enum_name,
        fields: enum_fields,
        span,
    })
}
