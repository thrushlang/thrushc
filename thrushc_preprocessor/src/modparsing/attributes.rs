use thrushc_attributes::{ThrushAttribute, ThrushAttributes, linkage::ThrushLinkage};
use thrushc_span::Span;

use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::{TokenType, traits::TokenTypeAttributesExtensions};

use crate::parser::ModuleParser;

pub fn build_attributes<'parser>(
    parser: &mut ModuleParser<'parser>,
    limits: &[TokenType],
) -> Result<ThrushAttributes, ()> {
    let mut attributes: ThrushAttributes = Vec::with_capacity(10);

    while !limits.contains(&parser.peek().get_type()) {
        let current_tk: &Token = parser.peek();
        let span: Span = current_tk.get_span();

        match current_tk.get_type() {
            TokenType::Extern => {
                attributes.push(ThrushAttribute::Extern(
                    self::build_external_attribute(parser)?,
                    span,
                ));
            }

            TokenType::Convention => {
                attributes.push(ThrushAttribute::Convention(
                    self::build_call_convention_attribute(parser)?,
                    span,
                ));
            }

            TokenType::Linkage => {
                let result: (ThrushLinkage, String) = self::build_linkage_attribute(parser)?;

                let linkage: ThrushLinkage = result.0;
                let id: String = result.1;

                attributes.push(ThrushAttribute::Linkage(linkage, id, span));
            }

            TokenType::Public => {
                attributes.push(ThrushAttribute::Public(span));
                parser.only_advance()?;
            }

            TokenType::AsmSyntax => attributes.push(ThrushAttribute::AsmSyntax(
                self::build_assembler_syntax_attribute(parser)?,
                span,
            )),

            tk_type if tk_type.is_attribute() => {
                if let Some(compiler_attribute) = thrushc_attributes::as_attribute(tk_type, span) {
                    attributes.push(compiler_attribute);
                    parser.only_advance()?;
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
    parser: &mut ModuleParser<'parser>,
) -> Result<(ThrushLinkage, String), ()> {
    parser.only_advance()?;

    parser.consume(TokenType::LParen)?;

    let linkage_tk: &Token = parser.consume_these(&[TokenType::CString, TokenType::CNString])?;

    let id: String = linkage_tk.get_ascii_lexeme().to_string();
    let linkage: ThrushLinkage = ThrushLinkage::get_linkage(&id);

    parser.consume(TokenType::RParen)?;

    Ok((linkage, id))
}

fn build_external_attribute<'parser>(parser: &mut ModuleParser<'parser>) -> Result<String, ()> {
    parser.only_advance()?;

    parser.consume(TokenType::LParen)?;

    let name: &Token = parser.consume_these(&[TokenType::CString, TokenType::CNString])?;
    let name: String = name.get_lexeme().to_string();

    parser.consume(TokenType::RParen)?;

    Ok(name)
}

fn build_assembler_syntax_attribute<'parser>(
    parser: &mut ModuleParser<'parser>,
) -> Result<String, ()> {
    parser.only_advance()?;

    parser.consume(TokenType::LParen)?;

    let syntax_tk: &Token = parser.consume_these(&[TokenType::CString, TokenType::CNString])?;
    let syntax: String = syntax_tk.get_lexeme().to_string();

    parser.consume(TokenType::RParen)?;

    Ok(syntax)
}

fn build_call_convention_attribute(parser: &mut ModuleParser) -> Result<String, ()> {
    parser.only_advance()?;

    parser.consume(TokenType::LParen)?;

    let convention_tk: &Token = parser.consume_these(&[TokenType::CString, TokenType::CNString])?;
    let name: String = convention_tk.get_lexeme().to_string();

    parser.consume(TokenType::RParen)?;

    Ok(name)
}
