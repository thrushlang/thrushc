use thrushc_ast::Ast;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{ParserContext, expressions, statements};

pub fn build_defer_executation<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let defer_tk: &Token = ctx.consume(
        TokenType::Defer,
        CompilationIssueCode::E0001,
        "Expected 'defer'.".into(),
    )?;

    let span: Span = defer_tk.get_span();

    let node: Ast<'_> = if ctx.check(TokenType::LBrace) {
        statements::block::build_block(ctx)?
    } else {
        expressions::build_expression(ctx)?
    };

    Ok(Ast::Defer {
        node: node.into(),
        kind: Type::Void(span),
        span,
    })
}
