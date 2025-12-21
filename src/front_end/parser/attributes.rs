use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::types::lexer::traits::TokenTypeExtensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

pub fn build_attributes<'parser>(
    ctx: &mut ParserContext<'parser>,
    limits: &[TokenType],
) -> Result<crate::middle_end::mir::attributes::ThrushAttributes, CompilationIssue> {
    let mut attributes: crate::middle_end::mir::attributes::ThrushAttributes =
        Vec::with_capacity(10);

    while !limits.contains(&ctx.peek().kind) {
        let current_tk: &Token = ctx.peek();
        let span: Span = current_tk.get_span();

        match current_tk.kind {
            TokenType::Extern => {
                attributes.push(crate::middle_end::mir::attributes::ThrushAttribute::Extern(
                    self::build_external_attribute(ctx)?,
                    span,
                ));
            }

            TokenType::Convention => {
                attributes.push(
                    crate::middle_end::mir::attributes::ThrushAttribute::Convention(
                        self::build_call_convention_attribute(ctx)?,
                        span,
                    ),
                );
            }

            TokenType::Linkage => {
                let result: (
                    crate::middle_end::mir::attributes::linkage::ThrushLinkage,
                    String,
                ) = self::build_linkage_attribute(ctx)?;

                let linkage: crate::middle_end::mir::attributes::linkage::ThrushLinkage = result.0;
                let id: String = result.1;

                attributes.push(
                    crate::middle_end::mir::attributes::ThrushAttribute::Linkage(linkage, id, span),
                );
            }

            TokenType::Public => {
                attributes.push(crate::middle_end::mir::attributes::ThrushAttribute::Public(
                    span,
                ));
                ctx.only_advance()?;
            }

            TokenType::AsmSyntax => attributes.push(
                crate::middle_end::mir::attributes::ThrushAttribute::AsmSyntax(
                    self::build_assembler_syntax_attribute(ctx)?,
                    span,
                ),
            ),

            tk_type if tk_type.is_attribute() => {
                if let Some(compiler_attribute) =
                    crate::middle_end::mir::attributes::as_attribute(tk_type, span)
                {
                    attributes.push(compiler_attribute);
                    ctx.only_advance()?;
                }
            }

            _ => {
                break;
            }
        }
    }

    Ok(attributes)
}

fn build_linkage_attribute<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<
    (
        crate::middle_end::mir::attributes::linkage::ThrushLinkage,
        String,
    ),
    CompilationIssue,
> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let linkage_tk: &Token = ctx.consume(
        TokenType::Str,
        CompilationIssueCode::E0001,
        "Expected a string literal.".into(),
    )?;

    let id: String = linkage_tk.get_ascii_lexeme().to_string();
    let linkage: crate::middle_end::mir::attributes::linkage::ThrushLinkage =
        crate::middle_end::mir::attributes::linkage::ThrushLinkage::get_linkage(&id);

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok((linkage, id))
}

fn build_external_attribute<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<String, CompilationIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let name: &Token = ctx.consume(
        TokenType::Str,
        CompilationIssueCode::E0001,
        "Expected a string literal.".into(),
    )?;

    let name: String = name.get_lexeme().to_string();

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(name)
}

fn build_assembler_syntax_attribute<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<String, CompilationIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let syntax_tk: &Token = ctx.consume(
        TokenType::Str,
        CompilationIssueCode::E0001,
        "Expected a string literal.".into(),
    )?;

    let syntax: String = syntax_tk.get_lexeme().to_string();

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(syntax)
}

fn build_call_convention_attribute(ctx: &mut ParserContext) -> Result<String, CompilationIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let convention_tk: &Token = ctx.consume(
        TokenType::Str,
        CompilationIssueCode::E0001,
        "Expected a string literal.".into(),
    )?;

    let name: String = convention_tk.get_lexeme().to_string();

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(name)
}
