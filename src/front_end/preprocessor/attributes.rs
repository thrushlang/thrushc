use crate::{
    core::diagnostic::span::Span,
    front_end::{
        lexer::{token::Token, tokentype::TokenType},
        preprocessor::parser::ModuleParser,
    },
};

pub fn build_attributes(
    ctx: &mut ModuleParser,
    limits: &[TokenType],
) -> Result<crate::middle_end::mir::attributes::ThrushAttributes, ()> {
    let mut attributes: crate::middle_end::mir::attributes::ThrushAttributes =
        Vec::with_capacity(10);

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
                attributes.push(crate::middle_end::mir::attributes::ThrushAttribute::Public(
                    span,
                ));

                ctx.only_advance()?;
            }

            tk_type if tk_type.is_attribute() => {
                if let Some(compiler_attribute) =
                    crate::middle_end::mir::attributes::as_attribute(tk_type, span)
                {
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
