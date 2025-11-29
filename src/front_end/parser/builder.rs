use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::typesystem::modificators::GCCStructureTypeModificator;
use crate::front_end::typesystem::modificators::LLVMStructureTypeModificator;
use crate::front_end::typesystem::modificators::StructureTypeModificator;
use crate::middle_end::mir::attributes::ThrushAttributes;

use inkwell::AtomicOrdering;
use inkwell::ThreadLocalMode;

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
) -> Result<Option<ThreadLocalMode>, CompilationIssue> {
    if ctx.match_token(TokenType::ThreadDynamic)? {
        return Ok(Some(ThreadLocalMode::GeneralDynamicTLSModel));
    }

    if ctx.match_token(TokenType::ThreadExec)? {
        return Ok(Some(ThreadLocalMode::LocalExecTLSModel));
    }

    if ctx.match_token(TokenType::ThreadInit)? {
        return Ok(Some(ThreadLocalMode::InitialExecTLSModel));
    }

    Ok(None)
}

#[inline]
pub fn build_atomic_ord<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Option<AtomicOrdering>, CompilationIssue> {
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
