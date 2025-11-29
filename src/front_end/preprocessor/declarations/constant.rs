use crate::core::diagnostic::span::Span;
use crate::middle_end::mir::attributes::ThrushAttributes;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;

use crate::front_end::preprocessor::attributes;
use crate::front_end::preprocessor::parser::ModuleParser;
use crate::front_end::preprocessor::signatures::{ExternalSymbol, Signature, Variant};
use crate::front_end::preprocessor::typegen;

use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;

pub fn build_constant(parser: &mut ModuleParser) -> Result<Option<ExternalSymbol>, ()> {
    parser.consume(TokenType::Const)?;
    parser.advance_until_check(TokenType::Identifier)?;

    let name_tk: &Token = parser.consume(TokenType::Identifier)?;

    let name: String = name_tk.get_lexeme().to_string();
    let span: Span = name_tk.get_span();

    parser.consume(TokenType::Colon)?;

    let kind: Type = typegen::build_type(parser)?;

    let attributes: ThrushAttributes = attributes::build_attributes(parser, &[TokenType::Eq])?;
    let is_public: bool = attributes.has_public_attribute();

    if is_public {
        parser.advance_until(TokenType::SemiColon)?;

        return Ok(Some(ExternalSymbol {
            name,
            signature: Signature::Constant {
                kind,
                span,
                attributes,
            },
            variant: Variant::Constant,
        }));
    }

    parser.advance_until(TokenType::SemiColon)?;

    Ok(None)
}
