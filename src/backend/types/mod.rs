use std::rc::Rc;

use crate::frontend::types::ast::Ast;

pub mod impls;
pub mod repr;
pub mod traits;

pub type LLVMEitherExpression<'ctx> = (Option<(&'ctx str, Rc<Ast<'ctx>>)>, Option<Rc<Ast<'ctx>>>);
