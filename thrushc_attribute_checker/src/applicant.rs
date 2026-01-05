use thrushc_typesystem::Type;

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
