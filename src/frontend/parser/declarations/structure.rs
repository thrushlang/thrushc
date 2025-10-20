use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, builder, checks, typegen},
        types::{
            ast::Ast,
            parser::stmts::{
                traits::{StructFieldsExtensions, TokenExtensions},
                types::{StructFields, ThrushAttributes},
            },
        },
        typesystem::{modificators::StructureTypeModificator, types::Type},
    },
};

pub fn build_structure<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

    ctx.consume(
        TokenType::Struct,
        String::from("Syntax error"),
        String::from("Expected 'struct' keyword."),
    )?;

    let name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected identifier."),
    )?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::LBrace])?;
    let modificator: StructureTypeModificator = builder::build_structure_modificator(&attributes);

    ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let name: &str = name_tk.get_lexeme();
    let span: Span = name_tk.get_span();

    let mut fields_types: StructFields = (name, Vec::with_capacity(10), modificator);
    let mut field_position: u32 = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        if ctx.check(TokenType::Identifier) {
            let field_tk: &Token = ctx.consume(
                TokenType::Identifier,
                String::from("Syntax error"),
                String::from("Expected identifier."),
            )?;

            let field_name: &str = field_tk.get_lexeme();
            let field_span: Span = field_tk.get_span();

            ctx.consume(
                TokenType::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            let field_type: Type = typegen::build_type(ctx)?;

            fields_types
                .1
                .push((field_name, field_type, field_position, field_span));

            field_position += 1;

            if ctx.check(TokenType::RBrace) {
                break;
            } else if ctx.match_token(TokenType::Comma)? {
                if ctx.check(TokenType::RBrace) {
                    break;
                }
            } else if ctx.check_to(TokenType::Identifier, 0) {
                ctx.consume(
                    TokenType::Comma,
                    String::from("Syntax error"),
                    String::from("Expected ','."),
                )?;
            } else {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected identifier."),
                    None,
                    ctx.previous().get_span(),
                ));
            }
        } else {
            ctx.only_advance()?;

            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected structure fields identifiers."),
                None,
                ctx.previous().get_span(),
            ));
        }
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    if declare_forward {
        ctx.get_mut_symbols().new_struct(
            name,
            (name, fields_types.1, attributes, modificator),
            span,
        )?;

        return Ok(Ast::new_nullptr(span));
    }

    Ok(Ast::Struct {
        name,
        fields: fields_types.clone(),
        kind: fields_types.get_type(),
        attributes,
        span,
    })
}
