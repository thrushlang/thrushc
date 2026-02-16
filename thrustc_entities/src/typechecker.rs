use ahash::AHashMap as HashMap;
use thrustc_attributes::ThrustAttributes;
use thrustc_span::Span;
use thrustc_typesystem::Type;

pub type TypeCheckerLocal<'symbol> = (&'symbol Type, Span);
pub type TypeCheckerLocals<'symbol> = Vec<HashMap<&'symbol str, TypeCheckerLocal<'symbol>>>;

pub type TypeCheckerIntrinsic<'symbol> =
    (&'symbol Type, &'symbol [Type], &'symbol ThrustAttributes);
pub type TypeCheckerIntrinsics<'symbol> = HashMap<&'symbol str, TypeCheckerIntrinsic<'symbol>>;

pub type TypeCheckerAssemblerFunction<'symbol> =
    (&'symbol Type, &'symbol [Type], &'symbol ThrustAttributes);
pub type TypeCheckerAssemblerFunctions<'symbol> =
    HashMap<&'symbol str, TypeCheckerAssemblerFunction<'symbol>>;

pub type TypeCheckerFunction<'symbol> = (&'symbol Type, &'symbol [Type], &'symbol ThrustAttributes);
pub type TypeCheckerFunctions<'symbol> = HashMap<&'symbol str, TypeCheckerFunction<'symbol>>;
