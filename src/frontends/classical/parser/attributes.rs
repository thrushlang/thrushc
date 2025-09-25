use {
    crate::{
        backends::classical::llvm::compiler::{
            attributes::LLVMAttribute, conventions::CallConvention,
        },
        core::errors::standard::ThrushCompilerIssue,
        frontends::classical::{
            lexer::{span::Span, token::Token, tokentype::TokenType},
            parser::ParserContext,
            types::parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
    },
    ahash::AHashMap as HashMap,
    lazy_static::lazy_static,
};

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
        call_conventions.insert(b"swift", CallConvention::Swift);
        call_conventions.insert(b"haskell", CallConvention::GHC);
        call_conventions.insert(b"erlang", CallConvention::HiPE);

        call_conventions
    };
}

pub fn build_attributes<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
    limits: &[TokenType],
) -> Result<ThrushAttributes<'parser>, ThrushCompilerIssue> {
    let mut attributes: ThrushAttributes = Vec::with_capacity(10);

    while !limits.contains(&parser_ctx.peek().kind) {
        let current_tk: &Token = parser_ctx.peek();
        let span: Span = current_tk.span;

        match current_tk.kind {
            TokenType::Extern => {
                attributes.push(LLVMAttribute::Extern(
                    self::build_external_attribute(parser_ctx)?,
                    span,
                ));
            }

            TokenType::Convention => {
                attributes.push(LLVMAttribute::Convention(
                    self::build_call_convention_attribute(parser_ctx)?,
                    span,
                ));
            }

            TokenType::Public => {
                attributes.push(self::LLVMAttribute::Public(span));
                parser_ctx.only_advance()?;
            }

            TokenType::AsmSyntax => attributes.push(LLVMAttribute::AsmSyntax(
                self::build_assembler_syntax_attribute(parser_ctx)?,
                span,
            )),

            attribute if attribute.is_attribute() => {
                if let Some(compiler_attribute) = attribute.as_attribute(span) {
                    attributes.push(compiler_attribute);
                    parser_ctx.only_advance()?;
                }
            }

            _ => break,
        }
    }

    Ok(attributes)
}

fn build_external_attribute<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<&'parser str, ThrushCompilerIssue> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let name: &Token = parser_ctx.consume(
        TokenType::Str,
        String::from("Syntax error"),
        String::from("Expected a string literal for @extern(\"FFI NAME\")."),
    )?;

    let ffi_name: &str = name.get_lexeme();

    parser_ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ffi_name)
}

fn build_assembler_syntax_attribute<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<&'parser str, ThrushCompilerIssue> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let syntax_tk: &Token = parser_ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a string literal for @asmsyntax(\"Intel\").".into(),
    )?;

    let specified_syntax: &str = syntax_tk.get_lexeme();
    let syntax_span: Span = syntax_tk.get_span();

    let syntaxes: [&'static str; 2] = ["Intel", "AT&T"];

    if !syntaxes.contains(&specified_syntax) {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            format!(
                "Unknown assembler syntax, valid are '{}'.",
                syntaxes.join(", ")
            ),
            None,
            syntax_span,
        ));
    }

    parser_ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(specified_syntax)
}

fn build_call_convention_attribute(
    parser_ctx: &mut ParserContext,
) -> Result<CallConvention, ThrushCompilerIssue> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let convention_tk: &Token = parser_ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected a literal 'str' for @convention(\"CONVENTION NAME\").".into(),
    )?;

    let span: Span = convention_tk.span;
    let name: &[u8] = convention_tk.lexeme.as_bytes();

    if let Some(call_convention) = CALL_CONVENTIONS.get(name) {
        parser_ctx.consume(
            TokenType::RParen,
            "Syntax error".into(),
            "Expected ')'.".into(),
        )?;

        return Ok(*call_convention);
    }

    parser_ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Err(ThrushCompilerIssue::Error(
        String::from("Syntax error"),
        String::from("Unknown call convention."),
        None,
        span,
    ))
}
