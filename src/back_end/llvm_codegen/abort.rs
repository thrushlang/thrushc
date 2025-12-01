use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;

use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use std::path::PathBuf;
use std::process;

pub fn abort_codegen<'ctx>(
    context: &mut LLVMCodeGenContext<'ctx, '_>,
    message: &str,
    span: Span,
    file: PathBuf,
    line: u32,
) -> ! {
    let diagnostician: &mut Diagnostician = context.get_mut_diagnostician();

    diagnostician.dispatch_diagnostic(
        &CompilationIssue::BackenEndBug(
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
