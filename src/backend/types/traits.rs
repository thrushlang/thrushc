use inkwell::InlineAsmDialect;

pub trait AssemblerFunctionExtensions {
    fn assembler_syntax_attr_to_inline_assembler_dialect(syntax: &str) -> InlineAsmDialect;
}
