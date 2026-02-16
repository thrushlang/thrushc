use thrustc_span::Span;
use thrustc_typesystem::{Type, modificators::StructureTypeModificator};

use crate::Ast;

pub type StructureData<'ctx> = (
    &'ctx str,
    Vec<(&'ctx str, Type, u32, Span)>,
    StructureTypeModificator,
    Span,
);

pub type StructureDataFields<'ctx> = Vec<(&'ctx str, Type, u32, Span)>;
pub type StructDataField<'ctx> = (usize, &'ctx (&'ctx str, Type, u32, Span));

pub type EnumData<'ctx> = Vec<(&'ctx str, Type, Ast<'ctx>)>;
pub type EnumDataField<'ctx> = (&'ctx str, Type, Ast<'ctx>);

pub type ConstructorData<'ctx> = Vec<(&'ctx str, Ast<'ctx>, Type, u32)>;

pub type PropertyData = Vec<(Type, (Type, u32))>;
pub type PropertyDataField = (Type, (Type, u32));
pub type PropertyDataBaseField = (Type, u32);
