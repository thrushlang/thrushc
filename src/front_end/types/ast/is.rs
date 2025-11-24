use crate::back_end::llvm::compiler::builtins::Builtin;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstStatementExtentions;

impl Ast<'_> {
    #[inline]
    pub fn is_literal_value(&self) -> bool {
        matches!(self, Ast::Integer { .. } | Ast::Float { .. })
    }

    #[inline]
    pub fn is_reference(&self) -> bool {
        matches!(self, Ast::Reference { .. })
    }

    #[inline]
    pub fn is_before_unary(&self) -> bool {
        matches!(self, Ast::UnaryOp { is_pre: true, .. })
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        matches!(self, Ast::Function { .. })
    }

    #[inline]
    pub fn is_intrinsic(&self) -> bool {
        matches!(self, Ast::Intrinsic { .. })
    }

    #[inline]
    pub fn is_asm_function(&self) -> bool {
        matches!(self, Ast::AssemblerFunction { .. })
    }

    #[inline]
    pub fn is_struct(&self) -> bool {
        matches!(self, Ast::Struct { .. })
    }

    #[inline]
    pub fn is_enum(&self) -> bool {
        matches!(self, Ast::Enum { .. })
    }

    #[inline]
    pub fn is_str(&self) -> bool {
        matches!(self, Ast::Str { .. })
    }

    #[inline]
    pub fn is_constant(&self) -> bool {
        matches!(self, Ast::Const { .. })
    }

    #[inline]
    pub fn is_static(&self) -> bool {
        matches!(self, Ast::Static { .. })
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        matches!(self, Ast::Integer { .. })
    }

    #[inline]
    pub fn is_terminator(&self) -> bool {
        matches!(self, Ast::Return { .. })
    }

    #[inline]
    pub fn is_break(&self) -> bool {
        matches!(self, Ast::Break { .. })
    }

    #[inline]
    pub fn is_continue(&self) -> bool {
        matches!(self, Ast::Continue { .. })
    }
}

impl AstStatementExtentions for Ast<'_> {
    fn is_statement(&self) -> bool {
        matches!(
            self,
            Ast::Block { .. }
                | Ast::If { .. }
                | Ast::While { .. }
                | Ast::For { .. }
                | Ast::Return { .. }
                | Ast::Break { .. }
                | Ast::Continue { .. }
                | Ast::Local { .. }
                | Ast::FunctionParameter { .. }
                | Ast::Index { .. }
                | Ast::Reference { .. }
                | Ast::Property { .. }
                | Ast::Struct { .. }
                | Ast::Enum { .. }
                | Ast::Const { .. }
                | Ast::Static { .. }
                | Ast::Integer { .. }
                | Ast::Function { .. }
                | Ast::Intrinsic { .. }
                | Ast::AssemblerFunction { .. }
                | Ast::Builtin { .. }
        ) || self.is_lli()
    }
}

impl Ast<'_> {
    #[inline]
    pub fn is_lli(&self) -> bool {
        matches!(
            self,
            Ast::Write { .. } | Ast::Load { .. } | Ast::Address { .. } | Ast::Alloc { .. }
        )
    }
}

impl Ast<'_> {
    #[inline]
    pub fn is_empty_block(&self) -> bool {
        let Ast::Block { stmts, .. } = self else {
            return false;
        };

        stmts.is_empty()
    }
}

impl Ast<'_> {
    #[must_use]
    pub fn has_terminator(&self) -> bool {
        let Ast::Block { stmts, .. } = self else {
            return false;
        };

        stmts.iter().any(|stmt| stmt.is_terminator())
    }
}

impl Ast<'_> {
    #[inline]
    pub fn is_mutable(&self) -> bool {
        if let Ast::Local { metadata, .. } = self {
            return metadata.is_mutable();
        }

        if let Ast::FunctionParameter { metadata, .. } = self {
            return metadata.is_mutable();
        }

        if let Ast::Index { metadata, .. } = self {
            return metadata.is_mutable();
        }

        if let Ast::Reference { metadata, .. } = self {
            return metadata.is_mutable();
        }

        if let Ast::Property { source, .. } = self {
            return source.is_reference();
        }

        false
    }
}

impl Ast<'_> {
    #[inline]
    pub fn is_allocated(&self) -> bool {
        if let Ast::Reference { metadata, .. } = self {
            return metadata.is_allocated();
        }

        if let Ast::Property { metadata, .. } = self {
            return metadata.is_allocated();
        }

        false
    }

    #[inline]
    pub fn is_allocated_value(&self) -> Result<bool, ThrushCompilerIssue> {
        if let Ast::Reference { metadata, .. } = self {
            return Ok(metadata.is_allocated());
        }

        if let Ast::Property { metadata, .. } = self {
            return Ok(metadata.is_allocated());
        }

        Ok(self.get_value_type()?.is_ptr_like_type())
    }
}

impl Ast<'_> {
    pub fn is_constant_value(&self) -> bool {
        if matches!(
            self,
            Ast::Integer { .. }
                | Ast::Float { .. }
                | Ast::Boolean { .. }
                | Ast::Char { .. }
                | Ast::Str { .. }
                | Ast::NullPtr { .. }
        ) || self.is_constant_builtin()
        {
            return true;
        }

        if let Ast::EnumValue { value, .. } = self {
            return value.is_constant_value();
        }

        if let Ast::DirectRef { expr, .. } = self {
            return expr.is_constant_value();
        }

        if let Ast::Group { expression, .. } = self {
            return expression.is_constant_value();
        }

        if let Ast::BinaryOp { left, right, .. } = self {
            return left.is_constant_value() && right.is_constant_value();
        }

        if let Ast::UnaryOp { expression, .. } = self {
            return expression.is_constant_value();
        }

        if let Ast::Reference { metadata, .. } = self {
            return metadata.is_constant();
        }

        if let Ast::As { metadata, .. } = self {
            return metadata.is_constant();
        }

        if let Ast::FixedArray { items, .. } = self {
            return items.iter().all(|item| item.is_constant_value());
        }

        if let Ast::Constructor { args, .. } = self {
            return args.iter().all(|arg| arg.1.is_constant_value());
        }

        false
    }

    #[inline]
    pub fn is_constant_builtin(&self) -> bool {
        matches!(
            self,
            Self::Builtin {
                builtin: Builtin::AlignOf { .. } | Builtin::SizeOf { .. },
                ..
            }
        )
    }
}
