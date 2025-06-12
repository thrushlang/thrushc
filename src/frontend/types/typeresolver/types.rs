use ahash::AHashMap as HashMap;

use crate::frontend::types::lexer::ThrushType;

pub type TypeResolverFunction<'typer> = &'typer [ThrushType];
pub type TypeResolverFunctions<'typer> = HashMap<&'typer str, &'typer [ThrushType]>;
