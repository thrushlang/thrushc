use std::{path::PathBuf, process};

use crate::{
    back_end::llvm::compiler::context::LLVMCodeGenContext,
    core::{
        console::logging::LoggingType,
        diagnostic::diagnostician::Diagnostician,
        errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    },
    front_end::lexer::span::Span,
};

pub fn abort_codegen<'ctx>(
    context: &mut LLVMCodeGenContext<'ctx, '_>,
    message: &str,
    span: Span,
    file: PathBuf,
    line: u32,
) -> ! {
    let diagnostician: &mut Diagnostician = context.get_mut_diagnostician();

    diagnostician.dispatch_diagnostic(
        &ThrushCompilerIssue::BackenEndBug(
            "Failed to Compile".into(),
            message.into(),
            span,
            CompilationPosition::LLVMBackend,
            file,
            line,
        ),
        LoggingType::BackendBug,
    );

    process::exit(1);
}
