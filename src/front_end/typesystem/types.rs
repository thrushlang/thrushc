use crate::front_end::typesystem::modificators::FunctionReferenceTypeModificator;
use crate::front_end::typesystem::modificators::StructureTypeModificator;

#[derive(Debug, Clone, Default)]
pub enum Type {
    // Signed Integer Type
    S8,
    S16,
    S32,
    S64,
    SSize,

    // Unsigned Integer Type
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,

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
    Const(std::boxed::Box<Type>),

    // Ptr Type
    Ptr(Option<std::boxed::Box<Type>>),

    // Struct Type
    Struct(String, std::vec::Vec<Type>, StructureTypeModificator),

    // Fixed FixedArray
    FixedArray(std::boxed::Box<Type>, u32),

    // Array Type
    Array(std::boxed::Box<Type>),

    // Memory Address
    Addr,

    // Function Referece
    Fn(
        std::vec::Vec<Type>,
        std::boxed::Box<Type>,
        FunctionReferenceTypeModificator,
    ),

    // Void Type
    #[default]
    Void,

    // Internal Compiler Types

    // NullPtr
    NullPtr,
}
