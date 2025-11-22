use crate::back_end::llvm::compiler::conventions::CallConvention;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::types::attributes::ThrushAttribute;
use crate::front_end::types::attributes::linkage::ThrushLinkage;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::stmts::types::ThrushAttributes;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

pub const INLINE_ASSEMBLER_SYNTAXES: [&str; 2] = ["Intel", "AT&T"];

lazy_static! {
    pub static ref CALL_CONVENTIONS: HashMap<&'static [u8], CallConvention> = {
        let mut call_conventions: HashMap<&'static [u8], CallConvention> =
            HashMap::with_capacity(10);

        call_conventions.insert(b"C", CallConvention::Standard);
        call_conventions.insert(b"fast", CallConvention::Fast);
        call_conventions.insert(b"tail", CallConvention::Tail);
        call_conventions.insert(b"cold", CallConvention::Cold);
        call_conventions.insert(b"weakReg", CallConvention::PreserveMost);
        call_conventions.insert(b"strongReg", CallConvention::PreserveAll);
        call_conventions.insert(b"Swift", CallConvention::Swift);
        call_conventions.insert(b"Haskell", CallConvention::GHC);
        call_conventions.insert(b"Erlang", CallConvention::HiPE);

        call_conventions
    };
}

pub fn build_attributes<'parser>(
    ctx: &mut ParserContext<'parser>,
    limits: &[TokenType],
) -> Result<ThrushAttributes, ThrushCompilerIssue> {
    let mut attributes: ThrushAttributes = Vec::with_capacity(10);

    while !limits.contains(&ctx.peek().kind) {
        let current_tk: &Token = ctx.peek();
        let span: Span = current_tk.span;

        match current_tk.kind {
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

            attribute if attribute.is_attribute() => {
                if let Some(compiler_attribute) = attribute.as_attribute(span) {
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
) -> Result<(ThrushLinkage, String), ThrushCompilerIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let linkage_tk: &Token = ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a string literal.".into(),
    )?;

    let id: String = linkage_tk.get_ascii_lexeme().to_string();
    let linkage: ThrushLinkage = ThrushLinkage::get_linkage(&id);

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok((linkage, id))
}

fn build_external_attribute<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<String, ThrushCompilerIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let name: &Token = ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a string literal.".into(),
    )?;

    let name: String = name.get_lexeme().to_string();

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(name)
}

fn build_assembler_syntax_attribute<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<String, ThrushCompilerIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let syntax_tk: &Token = ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a string literal.".into(),
    )?;

    let syntax: String = syntax_tk.get_lexeme().to_string();

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(syntax)
}

fn build_call_convention_attribute(ctx: &mut ParserContext) -> Result<String, ThrushCompilerIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let convention_tk: &Token = ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a string literal.".into(),
    )?;

    let name: String = convention_tk.get_lexeme().to_string();

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(name)
}
