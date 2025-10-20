use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr},
        types::{
            ast::Ast,
            parser::{
                stmts::{
                    traits::{ConstructorExtensions, StructExtensions, TokenExtensions},
                    types::Constructor,
                },
                symbols::types::Struct,
            },
        },
        typesystem::modificators::StructureTypeModificator,
    },
};

pub fn build_constructor<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::New,
        "Syntax error".into(),
        "Expected 'new' keyword.".into(),
    )?;

    let identifier_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected 'identifier' keyword.".into(),
    )?;

    ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let name: &str = identifier_tk.get_lexeme();
    let span: Span = identifier_tk.get_span();

    let structure: Struct = ctx.get_symbols().get_struct(name, span)?;
    let modificator: StructureTypeModificator = structure.get_modificator();

    let required: usize = structure.get_fields().1.len();

    let mut args: Constructor = Vec::with_capacity(10);
    let mut amount: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        if ctx.match_token(TokenType::Identifier)? {
            let field_tk: &Token = ctx.previous();
            let field_span: Span = field_tk.span;
            let field_name: &str = field_tk.get_lexeme();

            ctx.consume(
                TokenType::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            if !structure.contains_field(field_name) {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected existing structure field name."),
                    None,
                    field_span,
                ));
            }

            if amount >= required {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Too many fields in structure"),
                    format!("Expected '{}' fields, not '{}'.", required, amount),
                    None,
                    span,
                ));
            }

            let expression: Ast = expr::build_expr(ctx)?;

            if let Some(target_type) = structure.get_field_type(field_name) {
                args.push((field_name, expression, target_type, amount as u32));
            }

            amount += 1;

            if ctx.check(TokenType::RBrace) {
                break;
            }

            if ctx.match_token(TokenType::Comma)? {
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
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected field name."),
                None,
                span,
            ));
        }
    }

    let provided: usize = args.len();

    if provided != required {
        return Err(ThrushCompilerIssue::Error(
            String::from("Missing fields in structure"),
            format!(
                "Expected '{}' arguments, but '{}' was gived.",
                required, provided
            ),
            None,
            span,
        ));
    }

    ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    Ok(Ast::Constructor {
        name,
        args: args.clone(),
        kind: args.get_type(name, modificator),
        span,
    })
}
