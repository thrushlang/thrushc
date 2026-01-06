use thrushc_attributes::{ThrushAttribute, ThrushAttributes, linkage::ThrushLinkage};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{
    Token,
    tokentype::TokenType,
    traits::{TokenExtensions, TokenTypeAttributesExtensions},
};

use crate::ParserContext;

pub fn build_attributes<'parser>(
    ctx: &mut ParserContext<'parser>,
    limits: &[TokenType],
) -> Result<ThrushAttributes, CompilationIssue> {
    let mut attributes: ThrushAttributes = Vec::with_capacity(10);

    while !limits.contains(&ctx.peek().get_type()) {
        let current_tk: &Token = ctx.peek();
        let span: Span = current_tk.get_span();

        match current_tk.get_type() {
            TokenType::Extern => {
                attributes.push(ThrushAttribute::Extern(
                    self::build_external_attribute(ctx)?,
                    span,
                ));
            }

            TokenType::Convention => {
                attributes.push(ThrushAttribute::Convention(
                    self::build_call_convention_attribute(ctx)?,
                    span,
                ));
            }

            TokenType::Linkage => {
                let result: (ThrushLinkage, String) = self::build_linkage_attribute(ctx)?;

                let linkage: ThrushLinkage = result.0;
                let id: String = result.1;

                attributes.push(ThrushAttribute::Linkage(linkage, id, span));
            }

            TokenType::Public => {
                attributes.push(ThrushAttribute::Public(span));
                ctx.only_advance()?;
            }

            TokenType::AsmSyntax => attributes.push(ThrushAttribute::AsmSyntax(
                self::build_assembler_syntax_attribute(ctx)?,
                span,
            )),

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
