use thrustc_attributes::ThrustAttributes;
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::{
    modparsing::{attributes, typegen},
    parser::ModuleParser,
    signatures::{Signature, Symbol, Variant},
};

pub fn parse_type<'module_parser>(ctx: &mut ModuleParser<'module_parser>) -> Result<Symbol, ()> {
    ctx.consume(TokenType::Type)?;

    let identifier_tk: &Token = ctx.consume(TokenType::Identifier)?;
    let name: String = identifier_tk.get_lexeme().to_string();
    let span: Span = identifier_tk.get_span();

    let attributes: ThrustAttributes = attributes::build_attributes(ctx, &[TokenType::Eq])?;

    ctx.consume(TokenType::Eq)?;

    let r#type: Type = typegen::build_type(ctx)?;

    ctx.consume(TokenType::SemiColon)?;

    Ok(Symbol {
        name,
        signature: Signature::CustomType {
            kind: r#type,
            attributes,
            span,
        },
        variant: Variant::CustomType,
    })
}
