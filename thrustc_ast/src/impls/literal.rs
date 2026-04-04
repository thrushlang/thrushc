use crate::{Ast, traits::AstLiteralExtensions};

impl AstLiteralExtensions for Ast<'_> {
    #[inline]
    fn is_literal_value(&self) -> bool {
        match self {
            Ast::Integer { .. }
            | Ast::Float { .. }
            | Ast::Boolean { .. }
            | Ast::Char { .. }
            | Ast::CString { .. }
            | Ast::CNString { .. }
            | Ast::NullPtr { .. } => true,

            Ast::FixedArray { items, .. } => items.iter().all(|item| item.is_literal_value()),
            Ast::Array { items, .. } => items.iter().all(|item| item.is_literal_value()),

            Ast::EnumValue { value, .. } => value.is_literal_value(),

            Ast::Group { node, .. } => node.is_literal_value(),
            Ast::BinaryOp { left, right, .. } => {
                left.is_literal_value() && right.is_literal_value()
            }
            Ast::UnaryOp { node, .. } => node.is_literal_value(),

            _ => false,
        }
    }
}
