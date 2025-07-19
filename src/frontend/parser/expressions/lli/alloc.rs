use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, typegen},
        types::{
            ast::Ast,
            parser::stmts::{sites::AllocationSite, traits::TokenExtensions},
        },
        typesystem::types::Type,
    },
};

pub fn build_alloc<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let alloc_tk: &Token = parser_context.consume(
        TokenType::Alloc,
        "Syntax error".into(),
        "Expected 'alloc' keyword.".into(),
    )?;

    let span: Span = alloc_tk.get_span();

    let site_allocation: AllocationSite = match parser_context.peek().kind {
        TokenType::Heap => {
            parser_context.only_advance()?;
            AllocationSite::Heap
        }

        TokenType::Stack => {
            parser_context.only_advance()?;
            AllocationSite::Stack
        }

        TokenType::Static => {
            parser_context.only_advance()?;
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

    parser_context.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let mut alloc_type: Type = typegen::build_type(parser_context)?;

    alloc_type = Type::Ptr(Some(alloc_type.into()));

    let attributes: Vec<LLVMAttribute> = if !parser_context.check(TokenType::RBrace) {
        attributes::build_attributes(parser_context, &[TokenType::RBrace, TokenType::SemiColon])?
    } else {
        Vec::new()
    };

    parser_context.consume(
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
