use crate::frontends::classical::lexer::tokentype::TokenType;

impl TokenType {
    #[must_use]
    pub fn is_logical_operator(&self) -> bool {
        matches!(
            self,
            TokenType::BangEq
                | TokenType::EqEq
                | TokenType::LessEq
                | TokenType::Less
                | TokenType::Greater
                | TokenType::GreaterEq
        )
    }

    #[must_use]
    pub fn is_logical_gate(&self) -> bool {
        matches!(self, TokenType::And | TokenType::Or)
    }

    #[must_use]
    pub fn is_minus_minus_operator(&self) -> bool {
        matches!(self, TokenType::MinusMinus)
    }

    #[must_use]
    pub fn is_plus_plus_operator(&self) -> bool {
        matches!(self, TokenType::PlusPlus)
    }

    #[must_use]
    pub fn is_address(&self) -> bool {
        matches!(self, TokenType::Addr)
    }

    #[must_use]
    pub fn is_mut(&self) -> bool {
        matches!(self, TokenType::Mut)
    }

    #[must_use]
    pub fn is_void(&self) -> bool {
        matches!(self, TokenType::Void)
    }

    #[must_use]
    pub fn is_bool(&self) -> bool {
        matches!(self, TokenType::Bool)
    }

    #[must_use]
    pub fn is_str(&self) -> bool {
        matches!(self, TokenType::Str)
    }

    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, TokenType::Array)
    }

    #[must_use]
    pub fn is_ptr(&self) -> bool {
        matches!(self, TokenType::Ptr)
    }

    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, TokenType::F32 | TokenType::F64 | TokenType::FX8680)
    }

    #[must_use]
    pub fn is_const(&self) -> bool {
        matches!(self, TokenType::Const)
    }

    #[must_use]
    pub fn is_fn_ref(&self) -> bool {
        matches!(self, TokenType::FnRef)
    }

    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            TokenType::S8
                | TokenType::S16
                | TokenType::S32
                | TokenType::S64
                | TokenType::U8
                | TokenType::U16
                | TokenType::U32
                | TokenType::U64
                | TokenType::U128
                | TokenType::Char
        )
    }

    #[must_use]
    pub fn is_type(&self) -> bool {
        self.is_integer()
            || self.is_float()
            || self.is_bool()
            || self.is_array()
            || self.is_ptr()
            || self.is_str()
            || self.is_void()
            || self.is_mut()
            || self.is_address()
            || self.is_const()
            || self.is_fn_ref()
    }

    #[must_use]
    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenType::Identifier)
    }
}
