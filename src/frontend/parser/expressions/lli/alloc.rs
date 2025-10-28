use crate::backend::llvm::compiler::attributes::LLVMAttribute;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::{span::Span, token::Token, tokentype::TokenType};
use crate::frontend::parser::{ParserContext, attributes, typegen};
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::{sites::AllocationSite, traits::TokenExtensions};
use crate::frontend::typesystem::types::Type;

pub fn build_alloc<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let alloc_tk: &Token = ctx.consume(
        TokenType::Alloc,
        "Syntax error".into(),
        "Expected 'alloc' keyword.".into(),
    )?;

    let span: Span = alloc_tk.get_span();

    let site_allocation: AllocationSite = match ctx.peek().kind {
        TokenType::Heap => {
            ctx.only_advance()?;
            AllocationSite::Heap
        }

        TokenType::Stack => {
            ctx.only_advance()?;
            AllocationSite::Stack
        }

        TokenType::Static => {
            ctx.only_advance()?;
            AllocationSite::Static
        }

        _ => {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected site allocation attribute.".into(),
                None,
                span,
            ));
        }
    };

    ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let mut alloc_type: Type = typegen::build_type(ctx)?;

    alloc_type = Type::Ptr(Some(alloc_type.into()));

    let attributes: Vec<LLVMAttribute> = if !ctx.check(TokenType::RBrace) {
        attributes::build_attributes(ctx, &[TokenType::RBrace, TokenType::SemiColon])?
    } else {
        Vec::new()
    };

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    Ok(Ast::Alloc {
        alloc: alloc_type,
        site_allocation,
        attributes,
        span,
    })
}
