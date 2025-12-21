use inkwell::AtomicOrdering;

use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::core::errors::standard::CompilationIssueCode;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::attributes;
use crate::front_end::parser::builder;
use crate::front_end::parser::expressions;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::constant::ConstantMetadata;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;

pub fn build_global_const<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Const,
        CompilationIssueCode::E0001,
        "Expected 'const' keyword.".into(),
    )?;

    let is_lazy: bool = ctx.match_token(TokenType::LazyThread)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atom_ord: Option<AtomicOrdering> = builder::build_atomic_ord(ctx)?;

    let const_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected name.".into(),
    )?;

    let name: &str = const_tk.get_lexeme();
    let ascii_name: &str = const_tk.get_ascii_lexeme();

    let span: Span = const_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        CompilationIssueCode::E0001,
        "Expected ':'.".into(),
    )?;

    let const_type: Type = typegen::build_type(ctx, false)?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::Eq])?;

    ctx.consume(
        TokenType::Eq,
        CompilationIssueCode::E0001,
        "Expected '='.".into(),
    )?;

    let value: Ast = expressions::build_expression(ctx)?;

    if declare_forward {
        ctx.get_mut_symbols().new_global_constant(
            name,
            (const_type.clone(), attributes.clone()),
            span,
        )?;
    }

    Ok(Ast::Const {
        name,
        ascii_name,
        kind: const_type,
        value: value.into(),
        attributes,
        metadata: ConstantMetadata::new(true, is_lazy, is_volatile, atom_ord),
        span,
    })
}
