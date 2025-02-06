#[derive(Debug)]
pub enum ThrushError {
    Parse(ThrushErrorKind, String, String, usize, String),
    Lex(ThrushErrorKind, String, String, usize),
    Scope(ThrushErrorKind, String, String, usize),
}

#[derive(Debug)]
pub enum ThrushErrorKind {
    SyntaxError,
    UnreachableNumber,
    ParsedNumber,
    UnknownChar,
    UnreachableVariable,
    ObjectNotDefined,
    VariableNotDefined,
    VariableNotDeclared,
    MissingEntryPoint,
}
