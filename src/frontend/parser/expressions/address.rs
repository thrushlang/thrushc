use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::parser::stmts::stmt::ThrushStatement,
    },
};

pub fn build_address_indexes<'instr>(
    parser_context: &mut ParserContext<'instr>,
    span: Span,
) -> Result<Vec<ThrushStatement<'instr>>, ThrushCompilerIssue> {
    parser_context.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut indexes: Vec<ThrushStatement> = Vec::with_capacity(10);

    loop {
        if parser_context.check(TokenType::RBrace) {
            break;
        }

        let index: ThrushStatement = expression::build_expr(parser_context)?;

        indexes.push(index);

        if parser_context.check(TokenType::RBrace) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if indexes.is_empty() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            "At least one index was expected.".into(),
            None,
            span,
        ));
    }

    Ok(indexes)
}
