use thrushc_attributes::ThrushAttributes;
use thrushc_errors::CompilationIssue;
use thrushc_mir::{atomicord::ThrushAtomicOrdering, threadmode::ThrushThreadMode};
use thrushc_token::tokentype::TokenType;
use thrushc_typesystem::modificators::{
    GCCStructureTypeModificator, LLVMStructureTypeModificator, StructureTypeModificator,
};

use crate::ParserContext;

#[inline]
pub fn build_structure_modificator(attributes: &ThrushAttributes) -> StructureTypeModificator {
    let llvm_packed_modificator: bool = attributes.iter().any(|attr| attr.is_packed());

    StructureTypeModificator::new(
        LLVMStructureTypeModificator::new(llvm_packed_modificator),
        GCCStructureTypeModificator::new(),
    )
}

#[inline]
pub fn build_thread_local_mode<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Option<ThrushThreadMode>, CompilationIssue> {
    if ctx.match_token(TokenType::ThreadDynamic)? {
        return Ok(Some(ThrushThreadMode::GeneralDynamicTLSModel));
    }

    if ctx.match_token(TokenType::ThreadExec)? {
        return Ok(Some(ThrushThreadMode::LocalExecTLSModel));
    }

    if ctx.match_token(TokenType::ThreadInit)? {
        return Ok(Some(ThrushThreadMode::InitialExecTLSModel));
    }

    if ctx.match_token(TokenType::ThreadLDynamic)? {
        return Ok(Some(ThrushThreadMode::LocalDynamicTLSModel));
    }

    Ok(None)
}

#[inline]
pub fn build_atomic_ord<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Option<ThrushAtomicOrdering>, CompilationIssue> {
    if ctx.match_token(TokenType::AtomNone)? {
        return Ok(Some(ThrushAtomicOrdering::AtomicNone));
    }

    if ctx.match_token(TokenType::AtomFree)? {
        return Ok(Some(ThrushAtomicOrdering::AtomicFree));
    }

    if ctx.match_token(TokenType::AtomRelax)? {
        return Ok(Some(ThrushAtomicOrdering::AtomicRelax));
    }

    if ctx.match_token(TokenType::AtomGrab)? {
        return Ok(Some(ThrushAtomicOrdering::AtomicGrab));
    }

    if ctx.match_token(TokenType::AtomDrop)? {
        return Ok(Some(ThrushAtomicOrdering::AtomicDrop));
    }

    if ctx.match_token(TokenType::AtomSync)? {
        return Ok(Some(ThrushAtomicOrdering::AtomicSync));
    }

    if ctx.match_token(TokenType::AtomStrict)? {
        return Ok(Some(ThrushAtomicOrdering::AtomicStrict));
    }

    Ok(None)
}
