use ahash::AHashMap as HashMap;
use thrushc_attributes::ThrushAttributes;
use thrushc_span::Span;
use thrushc_typesystem::Type;

pub type TypeCheckerLocal<'symbol> = (&'symbol Type, Span);
pub type TypeCheckerLocals<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLocal<'symbol>>>;

pub type TypeCheckerIntrinsic<'symbol> =
    (&'symbol Type, &'symbol [Type], &'symbol ThrushAttributes);
pub type TypeCheckerIntrinsics<'symbol> = HashMap<&'symbol str, TypeCheckerIntrinsic<'symbol>>;

pub type TypeCheckerAssemblerFunction<'symbol> =
    (&'symbol Type, &'symbol [Type], &'symbol ThrushAttributes);
pub type TypeCheckerAssemblerFunctions<'symbol> =
    HashMap<&'symbol str, TypeCheckerAssemblerFunction<'symbol>>;

pub type TypeCheckerFunction<'symbol> = (&'symbol Type, &'symbol [Type], &'symbol ThrushAttributes);
pub type TypeCheckerFunctions<'symbol> = HashMap<&'symbol str, TypeCheckerFunction<'symbol>>;
