use crate::front_end::{
    preprocessor::signatures,
    types::{ast::Ast, parser::stmts::traits::ThrushAttributesExtensions},
};

#[derive(Debug)]
pub struct LinkageCheckerSignature<'signature> {
    pub name: &'signature [u8],
    pub variant: LinkageCheckerSignatureVariant,
}

impl<'signature> LinkageCheckerSignature<'signature> {
    #[inline]
    pub fn new(name: &'signature [u8], variant: LinkageCheckerSignatureVariant) -> Self {
        Self { name, variant }
    }
}

#[derive(Debug)]
pub enum LinkageCheckerSignatureVariant {
    Function,
    Global,
}

impl LinkageCheckerSignatureVariant {
    #[inline]
    pub fn is_function(&self) -> bool {
        matches!(self, Self::Function)
    }

    #[inline]
    pub fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}

pub fn transform<'signature>(
    ast: &'signature [Ast<'signature>],
) -> Vec<LinkageCheckerSignature<'signature>> {
    ast.iter()
        .filter_map(|ast| match ast {
            Ast::Function {
                name, attributes, ..
            } if attributes.has_extern_attribute() => Some(LinkageCheckerSignature::new(
                name.as_bytes(),
                LinkageCheckerSignatureVariant::Function,
            )),

            Ast::Static {
                name,
                attributes,
                value,
                ..
            } if attributes.has_public_attribute() && value.is_none() => {
                Some(LinkageCheckerSignature::new(
                    name.as_bytes(),
                    LinkageCheckerSignatureVariant::Global,
                ))
            }

            _ => None,
        })
        .collect()
}
