use inkwell::values::FunctionValue;
use thrushc_ast::Ast;
use thrushc_span::Span;
use thrushc_typesystem::Type;

use crate::{
    traits::{AstLLVMGetType, LLVMDBGFunctionExtensions, LLVMFunctionExtensions},
    types::{LLVMDBGFunction, LLVMFunction},
};

impl AstLLVMGetType for Ast<'_> {
    fn llvm_get_type(&self) -> &Type {
        match self {
            // Primitive values
            Ast::Integer { kind, .. } => kind,
            Ast::Float { kind, .. } => kind,
            Ast::Boolean { kind, .. } => kind,
            Ast::Char { kind, .. } => kind,
            Ast::Str { kind, .. } => kind,
            Ast::NullPtr { kind, .. } => kind,

            // Custom Type
            Ast::CustomType { kind, .. } => kind,

            // Static
            Ast::Static { kind, .. } => kind,

            // Variables and references
            Ast::Local { kind, .. } => kind,
            Ast::Mut { kind, .. } => kind,
            Ast::Reference { kind, .. } => kind,
            Ast::DirectRef { kind, .. } => kind,
            Ast::FunctionParameter { kind, .. } => kind,
            Ast::AssemblerFunctionParameter { kind, .. } => kind,

            // Memory operations
            Ast::Load { kind, .. } => kind,
            Ast::Address { kind, .. } => kind,

            // Memory operations
            Ast::Deref { kind, .. } => kind,

            // Composite types
            Ast::FixedArray { kind, .. } => kind,
            Ast::Array { kind, .. } => kind,
            Ast::Constructor { kind, .. } => kind,
            Ast::Property { kind, .. } => kind,
            Ast::EnumValue { kind, .. } => kind,

            // Expressions
            Ast::Call { kind, .. } => kind,
            Ast::BinaryOp { kind, .. } => kind,
            Ast::UnaryOp { kind, .. } => kind,
            Ast::Group { kind, .. } => kind,
            Ast::Index { kind, .. } => kind,

            // Type operations
            Ast::As { cast: kind, .. } => kind,

            // Builtins
            Ast::Builtin { kind, .. } => kind,

            // ASM Code Block
            Ast::AsmValue { kind, .. } => kind,

            // Indirect Call
            Ast::Indirect { kind, .. } => kind,

            // Intrinsic
            Ast::Intrinsic {
                return_type: kind, ..
            } => kind,
            Ast::IntrinsicParameter { kind, .. } => kind,

            // Invalid
            Ast::Invalid { kind, .. } => kind,

            // Control flow
            Ast::If { kind, .. } => kind,
            Ast::Elif { kind, .. } => kind,
            Ast::Else { kind, .. } => kind,
            Ast::For { kind, .. } => kind,
            Ast::Loop { kind, .. } => kind,
            Ast::While { kind, .. } => kind,
            Ast::Break { kind, .. } => kind,
            Ast::BreakAll { kind, .. } => kind,
            Ast::Continue { kind, .. } => kind,
            Ast::ContinueAll { kind, .. } => kind,
            Ast::Block { kind, .. } => kind,

            // Functions
            Ast::Function { return_type, .. } => return_type,
            Ast::AssemblerFunction { return_type, .. } => return_type,
            Ast::Return { kind, .. } => kind,

            // Composite type definitions
            Ast::Struct { kind, .. } => kind,
            Ast::Enum { kind, .. } => kind,

            // Constants
            Ast::Const { kind, .. } => kind,

            // LLI
            Ast::Write { write_type, .. } => write_type,

            // Module imports
            Ast::Import { kind, .. } => kind,
            Ast::ImportC { kind, .. } => kind,

            // Others
            Ast::Unreachable { kind, .. } => kind,
            Ast::GlobalAssembler { kind, .. } => kind,
        }
    }
}

impl<'ctx> LLVMFunctionExtensions<'ctx> for LLVMFunction<'ctx> {
    #[inline]
    fn get_value(&self) -> FunctionValue<'ctx> {
        self.0
    }

    #[inline]
    fn get_return_type(&self) -> &'ctx Type {
        self.1
    }

    #[inline]
    fn get_call_convention(&self) -> u32 {
        self.3
    }

    #[inline]
    fn get_param_count(&self) -> usize {
        self.2.len()
    }

    #[inline]
    fn get_parameters_types(&self) -> &[Type] {
        self.2
    }

    #[inline]
    fn get_span(&self) -> Span {
        self.4
    }
}

impl<'ctx> LLVMDBGFunctionExtensions<'ctx> for LLVMDBGFunction<'ctx> {
    #[inline]
    fn get_name(&self) -> &str {
        &self.0
    }

    #[inline]
    fn get_value(&self) -> FunctionValue<'ctx> {
        self.1
    }

    #[inline]
    fn get_return_type(&self) -> &'ctx Type {
        self.2
    }

    #[inline]
    fn get_parameters_types(&self) -> Vec<Type> {
        self.3.clone()
    }

    #[inline]
    fn is_definition(&self) -> bool {
        self.4
    }

    #[inline]
    fn is_local(&self) -> bool {
        self.5
    }

    #[inline]
    fn get_span(&self) -> Span {
        self.6
    }
}
