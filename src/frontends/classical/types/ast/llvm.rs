use crate::frontends::classical::types::ast::{Ast, traits::LLVMAstExtensions};

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
