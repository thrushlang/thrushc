use thrustc_attributes::{ThrustAttribute, ThrustAttributes, linkage::ThrustLinkage};
use thrustc_span::Span;

use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::{TokenType, traits::TokenTypeAttributesExtensions};

use crate::parser::ModuleParser;

pub fn build_attributes<'parser>(
    parser: &mut ModuleParser<'parser>,
    limits: &[TokenType],
) -> Result<ThrustAttributes, ()> {
    let mut attributes: ThrustAttributes = Vec::with_capacity(10);

    while !limits.contains(&parser.peek().get_type()) {
        let current_tk: &Token = parser.peek();
        let span: Span = current_tk.get_span();

        match current_tk.get_type() {
            TokenType::Extern => {
                attributes.push(ThrustAttribute::Extern(
                    self::build_external_attribute(parser)?,
                    span,
                ));
            }

            TokenType::Convention => {
                attributes.push(ThrustAttribute::Convention(
                    self::build_call_convention_attribute(parser)?,
                    span,
                ));
            }

            TokenType::Linkage => {
                let result: (ThrustLinkage, String) = self::build_linkage_attribute(parser)?;

                let linkage: ThrustLinkage = result.0;
                let id: String = result.1;

                attributes.push(ThrustAttribute::Linkage(linkage, id, span));
            }

            TokenType::Public => {
                attributes.push(ThrustAttribute::Public(span));
                parser.only_advance()?;
            }

            TokenType::AsmSyntax => attributes.push(ThrustAttribute::AsmSyntax(
                self::build_assembler_syntax_attribute(parser)?,
                span,
            )),

            tk_type if tk_type.is_attribute() => {
                if let Some(compiler_attribute) = thrustc_attributes::as_attribute(tk_type, span) {
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
) -> Result<(ThrustLinkage, String), ()> {
    parser.only_advance()?;

    parser.consume(TokenType::LParen)?;

    let linkage_tk: &Token = parser.consume_these(&[TokenType::CString, TokenType::CNString])?;

    let id: String = linkage_tk.get_ascii_lexeme().to_string();
    let linkage: ThrustLinkage = ThrustLinkage::get_linkage(&id);

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
