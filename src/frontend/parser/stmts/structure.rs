use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, typegen},
        types::{
            ast::Ast,
            lexer::ThrushType,
            parser::stmts::{
                traits::{StructFieldsExtensions, TokenExtensions},
                types::{StructFields, ThrushAttributes},
            },
        },
    },
};

pub fn build_structure<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let struct_tk: &Token = parser_ctx.consume(
        TokenType::Struct,
        String::from("Syntax error"),
        String::from("Expected 'struct' keyword."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Structs are only defined globally."),
            None,
            struct_tk.get_span(),
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected structure name."),
    )?;

    let struct_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let attributes: ThrushAttributes =
        attributes::build_attributes(parser_ctx, &[TokenType::LBrace])?;

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut fields_types: StructFields = (struct_name, Vec::with_capacity(10));
    let mut field_position: u32 = 0;

    loop {
        if parser_ctx.check(TokenType::RBrace) {
            break;
        }

        if parser_ctx.check(TokenType::Identifier) {
            let field_tk: &Token = parser_ctx.consume(
                TokenType::Identifier,
                String::from("Syntax error"),
                String::from("Expected identifier."),
            )?;

            let field_name: &str = field_tk.get_lexeme();
            let field_span: Span = field_tk.get_span();

            parser_ctx.consume(
                TokenType::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            let field_type: ThrushType = typegen::build_type(parser_ctx)?;

            fields_types
                .1
                .push((field_name, field_type, field_position, field_span));

            field_position += 1;

            if parser_ctx.check(TokenType::RBrace) {
                break;
            } else if parser_ctx.match_token(TokenType::Comma)? {
                if parser_ctx.check(TokenType::RBrace) {
                    break;
                }
            } else if parser_ctx.check_to(TokenType::Identifier, 0) {
                parser_ctx.consume(
                    TokenType::Comma,
                    String::from("Syntax error"),
                    String::from("Expected ','."),
                )?;
            } else {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected identifier."),
                    None,
                    parser_ctx.previous().get_span(),
                ));
            }
        } else {
            parser_ctx.only_advance()?;

            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected structure fields identifiers."),
                None,
                parser_ctx.previous().get_span(),
            ));
        }
    }

    parser_ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if declare_forward {
        if let Err(error) = parser_ctx.get_mut_symbols().new_struct(
            struct_name,
            (struct_name, fields_types.1, attributes),
            span,
        ) {
            parser_ctx.add_error(error);
        }

        return Ok(Ast::Null { span });
    }

    Ok(Ast::Struct {
        name: struct_name,
        fields: fields_types.clone(),
        kind: fields_types.get_type(),
        attributes,
        span,
    })
}
