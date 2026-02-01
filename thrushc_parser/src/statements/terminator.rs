use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{ParserContext, expressions};

pub fn build_return<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let return_tk: &Token = ctx.consume(
        TokenType::Return,
        CompilationIssueCode::E0001,
        "Expected 'return' keyword.".into(),
    )?;

    let span: Span = return_tk.get_span();

    if ctx.match_token(TokenType::SemiColon)? {
        return Ok(Ast::Return {
            expression: None,
            kind: Type::Void(span),
            span,
        });
    }

    let value: Ast = expressions::build_expr(ctx)?;
    let kind: &Type = value.get_value_type()?;

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    Ok(Ast::Return {
        expression: Some(value.clone().into()),
        kind: kind.clone(),
        span,
    })
}
