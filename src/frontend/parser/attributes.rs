use crate::backend::llvm::compiler::attributes::LLVMAttribute;
use crate::backend::llvm::compiler::conventions::CallConvention;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;

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
) -> Result<ThrushAttributes<'parser>, ThrushCompilerIssue> {
    let mut attributes: ThrushAttributes = Vec::with_capacity(10);

    while !limits.contains(&ctx.peek().kind) {
        let current_tk: &Token = ctx.peek();
        let span: Span = current_tk.span;

        match current_tk.kind {
            TokenType::Extern => {
                attributes.push(LLVMAttribute::Extern(
                    self::build_external_attribute(ctx)?,
                    span,
                ));
            }

            TokenType::Convention => {
                attributes.push(LLVMAttribute::Convention(
                    self::build_call_convention_attribute(ctx)?,
                    span,
                ));
            }

            TokenType::Public => {
                attributes.push(self::LLVMAttribute::Public(span));
                ctx.only_advance()?;
            }

            TokenType::AsmSyntax => attributes.push(LLVMAttribute::AsmSyntax(
                self::build_assembler_syntax_attribute(ctx)?,
                span,
            )),

            attribute if attribute.is_attribute() => {
                if let Some(compiler_attribute) = attribute.as_attribute(span) {
                    attributes.push(compiler_attribute);
                    ctx.only_advance()?;
                }
            }

            _ => break,
        }
    }

    Ok(attributes)
}

fn build_external_attribute<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<&'parser str, ThrushCompilerIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let name: &Token = ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a string literal for @extern(\"FFI NAME\").".into(),
    )?;

    let ffi_name: &str = name.get_lexeme();

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(ffi_name)
}

fn build_assembler_syntax_attribute<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<&'parser str, ThrushCompilerIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let syntax_tk: &Token = ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a string literal for @asmsyntax(\"Intel\").".into(),
    )?;

    let specified_syntax: &str = syntax_tk.get_lexeme();
    let syntax_span: Span = syntax_tk.get_span();

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    if !INLINE_ASSEMBLER_SYNTAXES.contains(&specified_syntax) {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            format!(
                "Unknown assembler syntax, valid are '{}'.",
                INLINE_ASSEMBLER_SYNTAXES.join(", ")
            ),
            None,
            syntax_span,
        ));
    }

    Ok(specified_syntax)
}

fn build_call_convention_attribute(
    ctx: &mut ParserContext,
) -> Result<CallConvention, ThrushCompilerIssue> {
    ctx.only_advance()?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let convention_tk: &Token = ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a literal 'str' for @convention(\"CONVENTION NAME\").".into(),
    )?;

    let span: Span = convention_tk.span;
    let name: &[u8] = convention_tk.lexeme.as_bytes();

    if let Some(call_convention) = CALL_CONVENTIONS.get(name) {
        ctx.consume(
            TokenType::RParen,
            "Syntax error".into(),
            "Expected ')'.".into(),
        )?;

        return Ok(*call_convention);
    }

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Err(ThrushCompilerIssue::Error(
        "Syntax error".into(),
        "Unknown call convention.".into(),
        None,
        span,
    ))
}
