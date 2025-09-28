use inkwell::AtomicOrdering;

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::tokentype::TokenType,
        parser::ParserContext,
        types::parser::stmts::types::ThrushAttributes,
        typesystem::modificators::{
            GCCStructureTypeModificator, LLVMStructureTypeModificator, StructureTypeModificator,
        },
    },
};

#[inline]
pub fn build_structure_modificator(attributes: &ThrushAttributes) -> StructureTypeModificator {
    let llvm_packed_modificator: bool = attributes.iter().any(|attr| attr.is_packed());

    StructureTypeModificator::new(
        LLVMStructureTypeModificator::new(llvm_packed_modificator),
        GCCStructureTypeModificator::new(),
    )
}

#[inline]
pub fn build_atomic_ord<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Option<AtomicOrdering>, ThrushCompilerIssue> {
    if ctx.match_token(TokenType::AtomNone)? {
        return Ok(Some(AtomicOrdering::NotAtomic));
    }

    if ctx.match_token(TokenType::AtomFree)? {
        return Ok(Some(AtomicOrdering::Unordered));
    }

    if ctx.match_token(TokenType::AtomRelax)? {
        return Ok(Some(AtomicOrdering::Monotonic));
    }

    if ctx.match_token(TokenType::AtomGrab)? {
        return Ok(Some(AtomicOrdering::Acquire));
    }

    if ctx.match_token(TokenType::AtomDrop)? {
        return Ok(Some(AtomicOrdering::Release));
    }

    if ctx.match_token(TokenType::AtomSync)? {
        return Ok(Some(AtomicOrdering::AcquireRelease));
    }

    if ctx.match_token(TokenType::AtomStrict)? {
        return Ok(Some(AtomicOrdering::SequentiallyConsistent));
    }

    Ok(None)
}
