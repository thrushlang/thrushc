use inkwell::InlineAsmDialect;

pub trait AssemblerFunctionExtensions {
    fn to_inline_assembler_dialect(syntax: &str) -> InlineAsmDialect;
}
