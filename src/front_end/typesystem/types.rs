use std::sync::Arc;

use crate::front_end::typesystem::modificators::FunctionReferenceTypeModificator;
use crate::front_end::typesystem::modificators::StructureTypeModificator;

#[derive(Debug, Clone, Default)]
pub enum Type {
    // Signed Integer Type
    S8,
    S16,
    S32,
    S64,

    // Unsigned Integer Type
    U8,
    U16,
    U32,
    U64,
    U128,

    // Floating Point Type
    F32,
    F64,
    F128,
    FX8680,
    FPPC128,

    // Boolean Type
    Bool,

    // Char Type
    Char,

    // Constant Type
    Const(Arc<Type>),

    // Ptr Type
    Ptr(Option<Arc<Type>>),

    // Struct Type
    Struct(String, Vec<Type>, StructureTypeModificator),

    // Fixed FixedArray
    FixedArray(Arc<Type>, u32),

    // Array Type
    Array(Arc<Type>),

    // Memory Address
    Addr,

    // Function Referece
    Fn(Vec<Type>, Arc<Type>, FunctionReferenceTypeModificator),

    // Void Type
    #[default]
    Void,

    // Internal Compiler Types

    // NullPtr
    NullPtr,
}
