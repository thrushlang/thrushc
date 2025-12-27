use crate::front_end::typesystem::types::Type;

#[derive(Debug, Clone, Copy)]
pub enum AttributeCheckerAttributeApplicant<'attr_checker> {
    AssemblerFunction,
    Intrinsic,
    Function { return_type: &'attr_checker Type },
    Constant,
    Static,
    Struct,
    Enum,
    Local,
}
