use thrushc_span::Span;
use thrushc_typesystem::{Type, modificators::StructureTypeModificator};

use crate::Ast;

pub type StructFields<'ctx> = (
    &'ctx str,
    Vec<(&'ctx str, Type, u32, Span)>,
    StructureTypeModificator,
    Span,
);
pub type StructField<'ctx> = (usize, &'ctx (&'ctx str, Type, u32, Span));

pub type EnumFields<'ctx> = Vec<(&'ctx str, Type, Ast<'ctx>)>;
pub type EnumField<'ctx> = (&'ctx str, Type, Ast<'ctx>);

pub type Constructor<'ctx> = Vec<(&'ctx str, Ast<'ctx>, Type, u32)>;
