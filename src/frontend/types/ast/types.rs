use std::rc::Rc;

use crate::frontend::types::ast::Ast;

pub type AstEitherExpression<'ctx> = (Option<(&'ctx str, Rc<Ast<'ctx>>)>, Option<Rc<Ast<'ctx>>>);
