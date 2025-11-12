use crate::back_end::llvm::compiler::builtins::Builtin;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

impl Ast<'_> {
    #[inline]
    pub fn is_literal(&self) -> bool {
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
    pub fn is_return(&self) -> bool {
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
        if let Ast::Block { stmts, .. } = self {
            return stmts.is_empty();
        }

        false
    }
}

impl Ast<'_> {
    #[must_use]
    pub fn has_return(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.has_return());
        }

        self.is_return()
    }

    #[must_use]
    pub fn has_return_for_function(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.has_return());
        }

        self.is_return()
    }

    #[must_use]
    pub fn has_break(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.has_break());
        }

        self.is_break()
    }

    #[must_use]
    pub fn has_continue(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.has_continue());
        }

        if let Ast::If {
            block,
            elseif,
            anyway,
            ..
        } = self
        {
            return block.has_continue()
                || elseif.iter().any(|elif| elif.has_continue())
                || anyway.as_ref().is_some_and(|anyway| anyway.has_continue());
        }

        if let Ast::Elif { block, .. } = self {
            return block.has_continue();
        }

        if let Ast::Else { block, .. } = self {
            return block.has_continue();
        }

        self.is_continue()
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
    pub fn is_unsigned_integer_for_index(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            Type::U8 | Type::U16 | Type::U32 | Type::U64
        ))
    }

    #[inline]
    pub fn is_unsigned_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128,
        ))
    }

    #[inline]
    pub fn is_lessu32bit_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            Type::U8 | Type::U16 | Type::U32
        ))
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
