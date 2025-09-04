use crate::frontends::classical::types::ast::{Ast, traits::AstExtensions};

impl AstExtensions for Ast<'_> {
    fn is_lli(&self) -> bool {
        matches!(
            self,
            Ast::Write { .. } | Ast::Load { .. } | Ast::Address { .. } | Ast::Alloc { .. }
        )
    }
}
