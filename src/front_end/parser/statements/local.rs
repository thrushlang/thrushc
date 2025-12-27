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
use crate::front_end::types::ast::metadata::local::LocalMetadata;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;

pub fn build_local<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Local,
        CompilationIssueCode::E0001,
        "Expected 'local' keyword.".into(),
    )?;

    let is_mutable: bool = ctx.match_token(TokenType::Mut)?;
    let is_volatile: bool = ctx.match_token(TokenType::Volatile)?;

    let atom_ord: Option<crate::middle_end::mir::atomicord::ThrushAtomicOrdering> =
        builder::build_atomic_ord(ctx)?;

    let local_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
    )?;

    let name: &str = local_tk.get_lexeme();
    let ascii_name: &str = local_tk.get_ascii_lexeme();
    let span: Span = local_tk.get_span();

    ctx.consume(
        TokenType::Colon,
        CompilationIssueCode::E0001,
        String::from("Expected ':'."),
    )?;

    let local_type: Type = typegen::build_type(ctx, false)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::SemiColon, TokenType::Eq])?;

    if ctx.match_token(TokenType::SemiColon)? {
        let metadata: LocalMetadata = LocalMetadata::new(true, is_mutable, is_volatile, atom_ord);

        ctx.get_mut_symbols()
            .new_local(name, (local_type.clone(), metadata, span), span)?;

        return Ok(Ast::Local {
            name,
            ascii_name,
            kind: local_type,
            value: None,
            attributes,
            metadata,
            span,
        });
    }

    let metadata: LocalMetadata = LocalMetadata::new(false, is_mutable, is_volatile, atom_ord);

    ctx.get_mut_symbols()
        .new_local(name, (local_type.clone(), metadata, span), span)?;

    ctx.consume(
        TokenType::Eq,
        CompilationIssueCode::E0001,
        String::from("Expected '='."),
    )?;

    ctx.get_mut_type_ctx().add_infered_type(local_type.clone());

    let value: Ast = expressions::build_expression(ctx)?;

    ctx.get_mut_type_ctx().pop_infered_type();

    let local: Ast = Ast::Local {
        name,
        ascii_name,
        kind: local_type,
        value: Some(value.into()),
        attributes,
        metadata,
        span,
    };

    Ok(local)
}
