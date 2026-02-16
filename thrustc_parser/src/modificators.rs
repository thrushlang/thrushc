use thrustc_attributes::ThrustAttributes;

use thrustc_typesystem::modificators::{
    GCCStructureTypeModificator, LLVMStructureTypeModificator, StructureTypeModificator,
};

use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_mir::{atomicord::ThrustAtomicOrdering, threadmode::ThrustThreadMode};
use thrustc_modificators::{Modificator, Modificators};
use thrustc_token::traits::TokenExtensions;
use thrustc_token_type::TokenType;

use crate::ParserContext;

#[inline]
pub fn build_structure_modificator(attributes: &ThrustAttributes) -> StructureTypeModificator {
    let llvm_packed_modificator: bool = attributes.iter().any(|attr| attr.is_packed());

    StructureTypeModificator::new(
        LLVMStructureTypeModificator::new(llvm_packed_modificator),
        GCCStructureTypeModificator::new(),
    )
}

pub fn build_stmt_modificator(
    ctx: &mut ParserContext,
    limits: &[TokenType],
) -> Result<Modificators, CompilationIssue> {
    let mut modificators: Modificators = Vec::with_capacity(u8::MAX as usize);

    const VALID_MODIFICATORS: &[TokenType] = &[
        TokenType::ThreadInit,
        TokenType::ThreadDynamic,
        TokenType::ThreadExec,
        TokenType::ThreadLDynamic,
        TokenType::AtomNone,
        TokenType::AtomFree,
        TokenType::AtomRelax,
        TokenType::AtomGrab,
        TokenType::AtomDrop,
        TokenType::Volatile,
        TokenType::LazyThread,
    ];

    while !limits.contains(&ctx.peek().get_type())
        && VALID_MODIFICATORS.contains(&ctx.peek().get_type())
    {
        let tk_type: TokenType = ctx.peek().get_type();

        match tk_type {
            TokenType::ThreadInit => {
                ctx.consume(
                    TokenType::ThreadInit,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::ThreadMode(
                    ThrustThreadMode::InitialExecTLSModel,
                ));
            }
            TokenType::ThreadDynamic => {
                ctx.consume(
                    TokenType::ThreadDynamic,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::ThreadMode(
                    ThrustThreadMode::GeneralDynamicTLSModel,
                ));
            }
            TokenType::ThreadExec => {
                ctx.consume(
                    TokenType::ThreadExec,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::ThreadMode(ThrustThreadMode::LocalExecTLSModel))
            }
            TokenType::ThreadLDynamic => {
                ctx.consume(
                    TokenType::ThreadLDynamic,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::ThreadMode(
                    ThrustThreadMode::LocalDynamicTLSModel,
                ))
            }
            TokenType::AtomNone => {
                ctx.consume(
                    TokenType::AtomNone,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::AtomicOrdering(
                    ThrustAtomicOrdering::AtomicNone,
                ))
            }
            TokenType::AtomFree => {
                ctx.consume(
                    TokenType::AtomFree,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::AtomicOrdering(
                    ThrustAtomicOrdering::AtomicFree,
                ))
            }
            TokenType::AtomRelax => {
                ctx.consume(
                    TokenType::AtomRelax,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::AtomicOrdering(
                    ThrustAtomicOrdering::AtomicRelax,
                ))
            }
            TokenType::AtomGrab => {
                ctx.consume(
                    TokenType::AtomGrab,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::AtomicOrdering(
                    ThrustAtomicOrdering::AtomicGrab,
                ))
            }
            TokenType::AtomDrop => {
                ctx.consume(
                    TokenType::AtomDrop,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::AtomicOrdering(
                    ThrustAtomicOrdering::AtomicDrop,
                ))
            }
            TokenType::AtomSync => {
                ctx.consume(
                    TokenType::AtomSync,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::AtomicOrdering(
                    ThrustAtomicOrdering::AtomicSync,
                ))
            }
            TokenType::AtomStrict => {
                ctx.consume(
                    TokenType::AtomStrict,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::AtomicOrdering(
                    ThrustAtomicOrdering::AtomicStrict,
                ))
            }
            TokenType::LazyThread => {
                ctx.consume(
                    TokenType::LazyThread,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::LazyThread);
            }
            TokenType::Volatile => {
                ctx.consume(
                    TokenType::Volatile,
                    CompilationIssueCode::E0001,
                    format!("Expected '{}' keyword.", tk_type),
                )?;

                modificators.push(Modificator::Volatile);
            }

            _ => break,
        }
    }

    Ok(modificators)
}
