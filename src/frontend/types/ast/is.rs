use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::types::{ast::Ast, lexer::ThrushType},
};

impl Ast<'_> {
    pub fn is_mutable(&self) -> bool {
        if let Ast::Local { is_mutable, .. } = self {
            return *is_mutable;
        }

        if let Ast::Index { is_mutable, .. } = self {
            return *is_mutable;
        }

        if let Ast::Reference {
            is_mutable, kind, ..
        } = self
        {
            return *is_mutable || kind.is_ptr_type() || kind.is_address_type();
        }

        if let Ast::Property { reference, .. } = self {
            return reference.is_mutable();
        }

        false
    }

    pub fn has_block(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return !stmts.is_empty();
        }

        false
    }

    pub fn has_return(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_return());
        }

        false
    }

    pub fn has_break(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_break());
        }

        false
    }

    pub fn has_continue(&self) -> bool {
        if let Ast::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_continue());
        }

        false
    }

    #[inline]
    pub fn is_block(&self) -> bool {
        matches!(self, Ast::Block { .. })
    }

    #[inline]
    pub fn is_unsigned_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            ThrushType::U8 | ThrushType::U16 | ThrushType::U32 | ThrushType::U64
        ))
    }

    #[inline]
    pub fn is_moreu32bit_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            ThrushType::U32 | ThrushType::U64
        ))
    }

    #[inline]
    pub fn is_lessu32bit_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            ThrushType::U8 | ThrushType::U16 | ThrushType::U32
        ))
    }

    #[inline]
    pub fn is_reference(&self) -> bool {
        matches!(self, Ast::Reference { .. })
    }

    #[inline]
    pub fn is_allocated_reference(&self) -> bool {
        matches!(
            self,
            Ast::Reference {
                is_allocated: true,
                ..
            }
        )
    }

    #[inline]
    pub fn is_pre_unaryop(&self) -> bool {
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
    pub fn is_constructor(&self) -> bool {
        matches!(self, Ast::Constructor { .. })
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
    pub fn is_bool(&self) -> bool {
        matches!(self, Ast::Boolean { .. })
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Ast::Float { .. })
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
