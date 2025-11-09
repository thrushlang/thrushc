pub trait FoundModuleSymbolEntension {
    fn is_structure(&self) -> bool;
    fn is_custom_type(&self) -> bool;
}

pub trait FoundModuleSymbolEither<'symbols> {
    fn expected_structure(&self) -> Result<String, ()>;
    fn expected_custom_type(&self) -> Result<String, ()>;
}
