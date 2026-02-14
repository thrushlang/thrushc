use thrushc_attributes::{ThrushAttribute, ThrushAttributes, linkage::ThrushLinkage};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;

use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::{TokenType, traits::TokenTypeAttributesExtensions};

use crate::ParserContext;

pub fn build_compiler_attributes<'parser>(
    ctx: &mut ParserContext<'parser>,
    limits: &[TokenType],
) -> Result<ThrushAttributes, CompilationIssue> {
    let mut attributes: ThrushAttributes = Vec::with_capacity(u8::MAX as usize);

    while !limits.contains(&ctx.peek().get_type()) {
        let current_tk: &Token = ctx.peek();
        let span: Span = current_tk.get_span();

        match current_tk.get_type() {
            TokenType::Extern => {
                ctx.consume(
                    TokenType::Extern,
                    CompilationIssueCode::E0001,
                    "Expected '@extern' prologue for an attribute.".into(),
                )?;

                attributes.push(ThrushAttribute::Extern(
                    self::build_external_attribute(ctx)?,
                    span,
                ));
            }

            TokenType::Convention => {
                ctx.consume(
                    TokenType::Convention,
                    CompilationIssueCode::E0001,
                    "Expected '@convention' prologue for an attribute.".into(),
                )?;

                attributes.push(ThrushAttribute::Convention(
                    self::build_call_convention_attribute(ctx)?,
                    span,
                ));
            }

            TokenType::Linkage => {
                ctx.consume(
                    TokenType::Linkage,
                    CompilationIssueCode::E0001,
                    "Expected '@linkage' prologue for an attribute.".into(),
                )?;

                let result: (ThrushLinkage, String) = self::build_linkage_attribute(ctx)?;

                let linkage: ThrushLinkage = result.0;
                let id: String = result.1;

                attributes.push(ThrushAttribute::Linkage(linkage, id, span));
            }

            TokenType::Public => {
                ctx.consume(
                    TokenType::Public,
                    CompilationIssueCode::E0001,
                    "Expected '@public' as attribute.".into(),
                )?;

                attributes.push(ThrushAttribute::Public(span));
            }

            TokenType::AsmSyntax => {
                ctx.consume(
                    TokenType::AsmSyntax,
                    CompilationIssueCode::E0001,
                    "Expected '@asmSyntax' prologue for an attribute.".into(),
                )?;

                attributes.push(ThrushAttribute::AsmSyntax(
                    self::build_assembler_syntax_attribute(ctx)?,
                    span,
                ))
            }

            tk_type if tk_type.is_attribute() => {
                if let Some(compiler_attribute) = thrushc_attributes::as_attribute(tk_type, span) {
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
) -> Result<(ThrushLinkage, String), CompilationIssue> {
    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let linkage_tk: &Token = ctx.consume_these(
        &[TokenType::CString, TokenType::CNString],
        CompilationIssueCode::E0001,
        "Expected a string literal.".into(),
    )?;

    let id: String = linkage_tk.get_ascii_lexeme().to_string();
    let linkage: ThrushLinkage = ThrushLinkage::get_linkage(&id);

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
    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let name: &Token = ctx.consume_these(
        &[TokenType::CString, TokenType::CNString],
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
    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let syntax_tk: &Token = ctx.consume_these(
        &[TokenType::CString, TokenType::CNString],
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
    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let convention_tk: &Token = ctx.consume_these(
        &[TokenType::CString, TokenType::CNString],
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
