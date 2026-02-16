use thrustc_token_type::TokenType;

use crate::{
    Type,
    traits::{PrecedenceTypeExtensions, TypeCodeLocation, TypeIsExtensions},
};

impl PrecedenceTypeExtensions for Type {
    fn get_term_precedence_type(&self, other: &Type, operator: TokenType) -> Type {
        if self.is_ptr_type() && other.is_ptr_type() && operator == TokenType::Minus {
            return Type::SSize(self.get_span());
        }

        self.clone()
    }
}
