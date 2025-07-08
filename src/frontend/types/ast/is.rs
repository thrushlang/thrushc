use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        types::ast::{Ast, traits::LLVMAstExtensions},
        typesystem::types::Type,
    },
};

impl Ast<'_> {
    #[inline]
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Ast::Integer { .. } | Ast::Float { .. } | Ast::Boolean { .. } | Ast::Char { .. }
        )
    }

    #[inline]
    pub fn is_block(&self) -> bool {
        matches!(self, Ast::Block { .. })
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
    pub fn is_null(&self) -> bool {
        matches!(self, Ast::Null { .. })
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
    pub fn has_block(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return !stmts.is_empty();
        }

        false
    }

    pub fn has_return(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| {
                if let Ast::If { block, .. } = stmt {
                    block.has_return()
                } else {
                    stmt.is_return()
                }
            });
        }

        false
    }

    pub fn has_break(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| {
                if let Ast::If { block, .. } = stmt {
                    block.has_break()
                } else {
                    stmt.is_break()
                }
            });
        }

        false
    }

    pub fn has_continue(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| {
                if let Ast::If { block, .. } = stmt {
                    block.has_continue()
                } else {
                    stmt.is_continue()
                }
            });
        }

        false
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
            if let Some(any_reference) = &source.0 {
                let reference: &Ast = &any_reference.1;
                return reference.is_mutable();
            }

            if let Some(expr) = &source.1 {
                return expr.is_mutable();
            }
        }

        false
    }
}

impl Ast<'_> {
    #[inline]
    pub fn is_unsigned_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            Type::U8 | Type::U16 | Type::U32 | Type::U64
        ))
    }

    #[inline]
    pub fn is_lessu32bit_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            Type::U8 | Type::U16 | Type::U32
        ))
    }

    #[inline]
    pub fn is_allocated_ref(&self) -> bool {
        if let Ast::Reference { metadata, .. } = self {
            return metadata.is_allocated();
        }

        false
    }
}

impl LLVMAstExtensions for Ast<'_> {
    fn is_llvm_constant_value(&self) -> bool {
        if matches!(
            self,
            Ast::Integer { .. }
                | Ast::Float { .. }
                | Ast::Boolean { .. }
                | Ast::Char { .. }
                | Ast::Str { .. }
        ) {
            return true;
        }

        if let Ast::Group { expression, .. } = self {
            return expression.is_llvm_constant_value();
        }

        if let Ast::BinaryOp { left, right, .. } = self {
            return left.is_llvm_constant_value() && right.is_llvm_constant_value();
        }

        if let Ast::UnaryOp { expression, .. } = self {
            return expression.is_llvm_constant_value();
        }

        if let Ast::Reference { metadata, .. } = self {
            return metadata.is_constant();
        }

        if let Ast::As { metadata, .. } = self {
            return metadata.is_constant();
        }

        if let Ast::FixedArray { items, .. } = self {
            return items.iter().all(|item| item.is_llvm_constant_value());
        }

        if let Ast::Constructor { args, .. } = self {
            return args.iter().all(|arg| arg.1.is_llvm_constant_value());
        }

        false
    }
}
