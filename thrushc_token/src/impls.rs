use crate::Token;

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TOKEN TYPE '{:?}' literal '{}', ascii '{}' at '{}'.",
            self.kind, self.lexeme, self.ascii, self.span
        )
    }
}
