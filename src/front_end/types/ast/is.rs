use crate::core::errors::standard::CompilationIssue;

use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{
    AstCodeBlockEntensions, AstConstantExtensions, AstGetType, AstMemoryExtensions,
    AstMutabilityExtensions, AstScopeExtensions, AstStandardExtensions, AstStatementExtentions,
};

impl AstStandardExtensions for Ast<'_> {
    #[inline]
    fn is_literal_value(&self) -> bool {
        matches!(self, Ast::Integer { .. } | Ast::Float { .. })
    }

    #[inline]
    fn is_reference(&self) -> bool {
        matches!(self, Ast::Reference { .. })
    }

    #[inline]
    fn is_before_unary(&self) -> bool {
        matches!(self, Ast::UnaryOp { is_pre: true, .. })
    }

    #[inline]
    fn is_function(&self) -> bool {
        matches!(self, Ast::Function { .. })
    }

    #[inline]
    fn is_intrinsic(&self) -> bool {
        matches!(self, Ast::Intrinsic { .. })
    }

    #[inline]
    fn is_asm_function(&self) -> bool {
        matches!(self, Ast::AssemblerFunction { .. })
    }

    #[inline]
    fn is_struct(&self) -> bool {
        matches!(self, Ast::Struct { .. })
    }

    #[inline]
    fn is_enum(&self) -> bool {
        matches!(self, Ast::Enum { .. })
    }

    #[inline]
    fn is_str(&self) -> bool {
        matches!(self, Ast::Str { .. })
    }

    #[inline]
    fn is_constant(&self) -> bool {
        matches!(self, Ast::Const { .. })
    }

    #[inline]
    fn is_static(&self) -> bool {
        matches!(self, Ast::Static { .. })
    }

    #[inline]
    fn is_integer(&self) -> bool {
        matches!(self, Ast::Integer { .. })
    }

    #[inline]
    fn is_terminator(&self) -> bool {
        matches!(self, Ast::Return { .. })
    }

    #[inline]
    fn is_unreacheable(&self) -> bool {
        matches!(self, Ast::Unreachable { .. })
    }

    #[inline]
    fn is_break(&self) -> bool {
        matches!(self, Ast::Break { .. })
    }

    #[inline]
    fn is_continue(&self) -> bool {
        matches!(self, Ast::Continue { .. })
    }

    #[inline]
    fn is_custom_type(&self) -> bool {
        matches!(self, Ast::CustomType { .. })
    }

    #[inline]
    fn is_global_asm(&self) -> bool {
        matches!(self, Ast::GlobalAssembler { .. })
    }

    #[inline]
    fn is_import(&self) -> bool {
        matches!(self, Ast::Import { .. })
    }

    #[inline]
    fn is_lli(&self) -> bool {
        matches!(
            self,
            Ast::Write { .. } | Ast::Load { .. } | Ast::Address { .. } | Ast::Alloc { .. }
        )
    }
}

impl AstStatementExtentions for Ast<'_> {
    fn is_statement(&self) -> bool {
        matches!(
            self,
            Ast::Block { .. }
                | Ast::If { .. }
                | Ast::Else { .. }
                | Ast::Elif { .. }
                | Ast::While { .. }
                | Ast::For { .. }
                | Ast::Loop { .. }
                | Ast::Return { .. }
                | Ast::Break { .. }
                | Ast::Continue { .. }
                | Ast::Local { .. }
                | Ast::Struct { .. }
                | Ast::Const { .. }
                | Ast::Static { .. }
        )
    }
}

impl AstCodeBlockEntensions for Ast<'_> {
    #[inline]
    fn is_empty_block(&self) -> bool {
        let Ast::Block { nodes, .. } = self else {
            return false;
        };

        nodes.is_empty()
    }

    #[inline]
    fn has_terminator(&self) -> bool {
        let Ast::Block { nodes, .. } = self else {
            return false;
        };

        nodes.iter().any(|node| node.is_terminator())
    }
}

impl AstMutabilityExtensions for Ast<'_> {
    #[inline]
    fn is_mutable(&self) -> bool {
        match self {
            Ast::Local { metadata, .. } => metadata.is_mutable(),
            Ast::FunctionParameter { metadata, .. } => metadata.is_mutable(),
            Ast::Index { metadata, .. } => metadata.is_mutable(),
            Ast::Reference { metadata, .. } => metadata.is_mutable(),
            Ast::Property { source, .. } => source.is_reference(),

            _ => false,
        }
    }
}

impl AstMemoryExtensions for Ast<'_> {
    #[inline]
    fn is_allocated(&self) -> bool {
        match self {
            Ast::Reference { metadata, .. } => metadata.is_allocated(),
            Ast::Property { metadata, .. } => metadata.is_allocated(),

            _ => false,
        }
    }

    #[inline]
    fn is_allocated_value(&self) -> Result<bool, CompilationIssue> {
        match self {
            Ast::Reference { metadata, .. } => Ok(metadata.is_allocated()),
            Ast::Property { metadata, .. } => Ok(metadata.is_allocated()),

            _ => Ok(self.get_value_type()?.is_ptr_like_type()),
        }
    }
}

impl AstConstantExtensions for Ast<'_> {
    fn is_constant_value(&self) -> bool {
        match self {
            Ast::Integer { .. }
            | Ast::Float { .. }
            | Ast::Boolean { .. }
            | Ast::Char { .. }
            | Ast::Str { .. }
            | Ast::NullPtr { .. }
            | Self::Builtin {
                builtin:
                    crate::middle_end::mir::builtins::ThrushBuiltin::AlignOf { .. }
                    | crate::middle_end::mir::builtins::ThrushBuiltin::SizeOf { .. }
                    | crate::middle_end::mir::builtins::ThrushBuiltin::AbiSizeOf { .. }
                    | crate::middle_end::mir::builtins::ThrushBuiltin::AbiAlignOf { .. }
                    | crate::middle_end::mir::builtins::ThrushBuiltin::BitSizeOf { .. },
                ..
            } => true,
            Ast::EnumValue { value, .. } => value.is_constant_value(),
            Ast::DirectRef { expr, .. } => expr.is_constant_value(),
            Ast::Group { expression, .. } => expression.is_constant_value(),
            Ast::BinaryOp { left, right, .. } => {
                left.is_constant_value() && right.is_constant_value()
            }
            Ast::UnaryOp { expression, .. } => expression.is_constant_value(),
            Ast::Reference { metadata, .. } => metadata.is_constant(),
            Ast::As { metadata, .. } => metadata.is_constant(),
            Ast::FixedArray { items, .. } => items.iter().all(|item| item.is_constant_value()),
            Ast::Constructor { args, .. } => args.iter().all(|arg| arg.1.is_constant_value()),

            _ => false,
        }
    }
}

impl AstScopeExtensions for Ast<'_> {
    #[inline]
    fn is_compatible_with_main_scope(&self) -> bool {
        matches!(
            self,
            Ast::CustomType { .. }
                | Ast::Struct { .. }
                | Ast::Enum { .. }
                | Ast::Intrinsic { .. }
                | Ast::Function { .. }
                | Ast::AssemblerFunction { .. }
                | Ast::GlobalAssembler { .. }
                | Ast::Const { .. }
                | Ast::Static { .. }
                | Ast::Import { .. }
        )
    }
}
