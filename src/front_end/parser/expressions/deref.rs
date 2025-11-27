use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::builder;
use crate::front_end::parser::expr;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::dereference::DereferenceMetadata;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::typesystem::traits::DereferenceExtensions;
use crate::front_end::typesystem::types::Type;

use inkwell::AtomicOrdering;

pub fn build_dereference<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let initial_deref_tk: &Token = ctx.advance()?;
    let span: Span = initial_deref_tk.span;

    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;
    let atomic_ord: Option<AtomicOrdering> = builder::build_atomic_ord(ctx)?;

    let mut deref_count: u64 = 1;

    let mut current_expr: Ast = {
        while ctx.check(TokenType::Deref) {
            ctx.consume(
                TokenType::Deref,
                "Syntax error".into(),
                "Expected 'defer' keyword.".into(),
            )?;

            deref_count += 1;
        }

        let expr: Ast = expr::build_expr(ctx)?;

        expr
    };

    let mut current_type: Type = current_expr.get_value_type()?.clone();

    (0..deref_count).for_each(|_| {
        current_expr = Ast::Deref {
            value: current_expr.clone().into(),
            kind: current_type.dereference(),
            metadata: DereferenceMetadata::new(is_volatile, atomic_ord),
            span,
        };

        current_type = current_type.dereference();
    });

    Ok(current_expr)
}
