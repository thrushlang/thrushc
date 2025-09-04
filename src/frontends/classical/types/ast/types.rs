use std::rc::Rc;

use crate::frontends::classical::types::ast::Ast;

pub type AstEitherExpression<'ctx> = (Option<(&'ctx str, Rc<Ast<'ctx>>)>, Option<Rc<Ast<'ctx>>>);
