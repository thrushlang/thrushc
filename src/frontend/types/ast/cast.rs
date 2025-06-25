use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        types::ast::Ast,
    },
};

impl Ast<'_> {
    pub fn cast_signess(&mut self, operator: TokenType) {
        if let Ast::Integer { kind, signed, .. } = self {
            if operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
                *signed = true;
            }
        }

        if let Ast::Reference { kind, .. } = self {
            if kind.is_integer_type() && operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
            }
        }

        if let Ast::Float { signed, .. } = self {
            if operator.is_minus_operator() {
                *signed = true;
            }
        }

        if let Ast::Group { kind, .. } = self {
            if kind.is_integer_type() && operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
            }
        }

        if let Ast::UnaryOp { kind, .. } = self {
            if kind.is_integer_type() && operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
            }
        }

        if let Ast::BinaryOp { kind, .. } = self {
            if kind.is_integer_type() && operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
            }
        }
    }

    pub fn throw_attemping_use_jit(&self, span: Span) -> Result<(), ThrushCompilerIssue> {
        if !self.is_integer() && !self.is_float() && !self.is_bool() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Attemping use JIT"),
                String::from("This expression cannot be compiled correctly."),
                Some(String::from(
                    "The compiler does not accept runtime-only expressions until the Just-in-Time (JIT) compiler development is complete.",
                )),
                span,
            ));
        }

        Ok(())
    }
}
