use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::expressions::precedences::mutation;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::types::Type;

pub fn factor<'parser>(ctx: &mut ParserContext<'parser>) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expression: Ast = mutation::equal_precedence(ctx)?;

    while ctx.match_token(TokenType::Slash)? || ctx.match_token(TokenType::Star)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = mutation::equal_precedence(ctx)?;

        let left_type: &Type = expression.get_value_type()?;

        expression = Ast::BinaryOp {
            left: expression.clone().into(),
            operator,
            right: right.into(),
            kind: left_type.clone(),
            span,
        };
    }

    Ok(expression)
}
