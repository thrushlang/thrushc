use thrushc_span::Span;
use thrushc_typesystem::{Type, modificators::StructureTypeModificator};

use crate::Ast;

pub type StructureData<'ctx> = (
    &'ctx str,
    Vec<(&'ctx str, Type, u32, Span)>,
    StructureTypeModificator,
    Span,
);

pub type StructDataField<'ctx> = (usize, &'ctx (&'ctx str, Type, u32, Span));

pub type EnumData<'ctx> = Vec<(&'ctx str, Type, Ast<'ctx>)>;
pub type EnumDataField<'ctx> = (&'ctx str, Type, Ast<'ctx>);

pub type ConstructorData<'ctx> = Vec<(&'ctx str, Ast<'ctx>, Type, u32)>;
