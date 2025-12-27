use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::typesystem::modificators::GCCStructureTypeModificator;
use crate::front_end::typesystem::modificators::LLVMStructureTypeModificator;
use crate::front_end::typesystem::modificators::StructureTypeModificator;

use crate::middle_end::mir::attributes::ThrushAttributes;

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
) -> Result<Option<crate::middle_end::mir::threadmode::ThrushThreadMode>, CompilationIssue> {
    if ctx.match_token(TokenType::ThreadDynamic)? {
        return Ok(Some(
            crate::middle_end::mir::threadmode::ThrushThreadMode::GeneralDynamicTLSModel,
        ));
    }

    if ctx.match_token(TokenType::ThreadExec)? {
        return Ok(Some(
            crate::middle_end::mir::threadmode::ThrushThreadMode::LocalExecTLSModel,
        ));
    }

    if ctx.match_token(TokenType::ThreadInit)? {
        return Ok(Some(
            crate::middle_end::mir::threadmode::ThrushThreadMode::InitialExecTLSModel,
        ));
    }

    if ctx.match_token(TokenType::ThreadLDynamic)? {
        return Ok(Some(
            crate::middle_end::mir::threadmode::ThrushThreadMode::LocalDynamicTLSModel,
        ));
    }

    Ok(None)
}

#[inline]
pub fn build_atomic_ord<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Option<crate::middle_end::mir::atomicord::ThrushAtomicOrdering>, CompilationIssue> {
    if ctx.match_token(TokenType::AtomNone)? {
        return Ok(Some(
            crate::middle_end::mir::atomicord::ThrushAtomicOrdering::AtomicNone,
        ));
    }

    if ctx.match_token(TokenType::AtomFree)? {
        return Ok(Some(
            crate::middle_end::mir::atomicord::ThrushAtomicOrdering::AtomicFree,
        ));
    }

    if ctx.match_token(TokenType::AtomRelax)? {
        return Ok(Some(
            crate::middle_end::mir::atomicord::ThrushAtomicOrdering::AtomicRelax,
        ));
    }

    if ctx.match_token(TokenType::AtomGrab)? {
        return Ok(Some(
            crate::middle_end::mir::atomicord::ThrushAtomicOrdering::AtomicGrab,
        ));
    }

    if ctx.match_token(TokenType::AtomDrop)? {
        return Ok(Some(
            crate::middle_end::mir::atomicord::ThrushAtomicOrdering::AtomicDrop,
        ));
    }

    if ctx.match_token(TokenType::AtomSync)? {
        return Ok(Some(
            crate::middle_end::mir::atomicord::ThrushAtomicOrdering::AtomicSync,
        ));
    }

    if ctx.match_token(TokenType::AtomStrict)? {
        return Ok(Some(
            crate::middle_end::mir::atomicord::ThrushAtomicOrdering::AtomicStrict,
        ));
    }

    Ok(None)
}
