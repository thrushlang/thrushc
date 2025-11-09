use crate::{
    back_end::llvm::compiler::attributes::LLVMAttribute,
    front_end::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        preprocessor::parser::ModuleParser,
        types::parser::stmts::types::ThrushAttributes,
    },
};

pub fn build_attributes<'module_parser>(
    ctx: &mut ModuleParser<'module_parser>,
    limits: &[TokenType],
) -> Result<ThrushAttributes<'module_parser>, ()> {
    let mut attributes: ThrushAttributes = Vec::with_capacity(10);

    while !limits.contains(&ctx.peek().kind) {
        let current_tk: &Token = ctx.peek();
        let span: Span = current_tk.span;

        match current_tk.kind {
            TokenType::Extern => {
                self::build_external_attribute(ctx)?;
            }
            TokenType::Convention => {
                self::build_call_convention_attribute(ctx)?;
            }
            TokenType::AsmSyntax => {
                self::build_assembler_syntax_attribute(ctx)?;
            }

            TokenType::Public => {
                attributes.push(LLVMAttribute::Public(span));
                ctx.only_advance()?;
            }

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

fn build_external_attribute(ctx: &mut ModuleParser) -> Result<(), ()> {
    ctx.only_advance()?;

    ctx.consume(TokenType::LParen)?;
    ctx.consume(TokenType::Str)?;
    ctx.consume(TokenType::RParen)?;

    Ok(())
}

fn build_assembler_syntax_attribute(ctx: &mut ModuleParser) -> Result<(), ()> {
    ctx.only_advance()?;

    ctx.consume(TokenType::LParen)?;
    ctx.consume(TokenType::Str)?;
    ctx.consume(TokenType::RParen)?;

    Ok(())
}

fn build_call_convention_attribute(ctx: &mut ModuleParser) -> Result<(), ()> {
    ctx.only_advance()?;

    ctx.consume(TokenType::LParen)?;
    ctx.consume(TokenType::Str)?;
    ctx.consume(TokenType::RParen)?;

    Ok(())
}
