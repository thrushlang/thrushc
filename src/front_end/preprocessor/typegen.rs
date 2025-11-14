use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::preprocessor::attributes;
use crate::front_end::preprocessor::parser::ModuleParser;

use crate::front_end::types::parser::stmts::traits::{ThrushAttributesExtensions, TokenExtensions};
use crate::front_end::types::parser::stmts::types::ThrushAttributes;
use crate::front_end::types::preprocessor::symbols::traits::FoundModuleSymbolEither;
use crate::front_end::types::preprocessor::symbols::traits::FoundModuleSymbolEntension;
use crate::front_end::typesystem::modificators::{
    FunctionReferenceTypeModificator, GCCFunctionReferenceTypeModificator,
    LLVMFunctionReferenceTypeModificator,
};
use crate::front_end::typesystem::types::Type;

pub fn build_type(parser: &mut ModuleParser) -> Result<Type, ()> {
    match parser.peek().kind {
        tk_kind if tk_kind.is_type() => {
            parser.only_advance()?;

            if tk_kind.is_array() {
                return self::build_array_type(parser);
            }
            if tk_kind.is_const() {
                return self::build_const_type(parser);
            }
            if tk_kind.is_fn_ref() {
                return self::build_fn_ref_type(parser);
            }

            match tk_kind.as_type_preprocessor()? {
                ty if ty.is_ptr_type() && parser.check(TokenType::LBracket) => {
                    self::build_recursive_type(parser, Type::Ptr(None))
                }
                ty => Ok(ty),
            }
        }

        TokenType::Identifier => {
            let identifier_tk: &Token = parser.advance()?;

            let name: String = identifier_tk.get_lexeme().to_string();

            if let Ok(object) = parser.get_table().get_symbols_id(name) {
                if object.is_structure() {
                    let id: String = object.expected_structure()?;

                    let kind: Type = parser.get_table().get_struct_by_id(id)?;

                    Ok(kind)
                } else if object.is_custom_type() {
                    let id: String = object.expected_custom_type()?;

                    let kind: Type = parser.get_table().get_custom_type_by_id(id)?;

                    Ok(kind)
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
        }

        _ => Err(()),
    }
}

fn build_fn_ref_type(parser: &mut ModuleParser) -> Result<Type, ()> {
    parser.consume(TokenType::LBracket)?;

    let mut parameter_types: Vec<Type> = Vec::with_capacity(10);

    loop {
        if parser.check(TokenType::RBracket) {
            break;
        }

        parameter_types.push(self::build_type(parser)?);

        if parser.check(TokenType::RBracket) {
            break;
        }

        parser.consume(TokenType::Comma)?;
    }

    parser.consume(TokenType::RBracket)?;

    let attributes: ThrushAttributes = attributes::build_attributes(parser, &[TokenType::Arrow])?;
    let has_ignore: bool = attributes.has_ignore_attribute();

    parser.consume(TokenType::Arrow)?;

    let return_type: Type = build_type(parser)?;

    Ok(Type::Fn(
        parameter_types,
        return_type.into(),
        FunctionReferenceTypeModificator::new(
            LLVMFunctionReferenceTypeModificator::new(has_ignore),
            GCCFunctionReferenceTypeModificator::default(),
        ),
    ))
}

fn build_const_type(parser: &mut ModuleParser) -> Result<Type, ()> {
    Ok(Type::Const(self::build_type(parser)?.into()))
}

fn build_array_type(parser: &mut ModuleParser) -> Result<Type, ()> {
    parser.consume(TokenType::LBracket)?;

    let array_type: Type = self::build_type(parser)?;

    if parser.check(TokenType::SemiColon) {
        parser.consume(TokenType::SemiColon)?;

        let size: Option<u32> = match parser.peek().kind {
            TokenType::Integer => {
                let tk: &Token = parser.advance()?;

                tk.get_lexeme().parse::<u32>().ok()
            }
            _ => None,
        };

        if let Some(array_size) = size {
            parser.consume(TokenType::RBracket)?;

            return Ok(Type::FixedArray(array_type.into(), array_size));
        }

        return Err(());
    }

    parser.consume(TokenType::RBracket)?;

    Ok(Type::Array(array_type.into()))
}

fn build_recursive_type(parser: &mut ModuleParser, mut before_type: Type) -> Result<Type, ()> {
    parser.consume(TokenType::LBracket)?;

    if let Type::Ptr(_) = &mut before_type {
        let mut inner_type: Type = self::build_type(parser)?;

        while parser.check(TokenType::LBracket) {
            inner_type = self::build_recursive_type(parser, inner_type)?;
        }

        parser.consume(TokenType::RBracket)?;

        Ok(Type::Ptr(Some(inner_type.into())))
    } else {
        Err(())
    }
}
