use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationPosition};
use thrushc_logging::LoggingType;
use thrushc_span::Span;

use crate::{context::LLVMCodeGenContext, debug::LLVMDebugContext};

pub fn abort_codegen<'ctx>(
    context: &mut LLVMCodeGenContext<'ctx, '_>,
    message: &str,
    span: Span,
    file: std::path::PathBuf,
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

    std::process::exit(1);
}

pub fn abort_codegen_dbg<'ctx>(
    context: &mut LLVMDebugContext<'ctx, '_>,
    message: &str,
    span: Span,
    file: std::path::PathBuf,
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

    std::process::exit(1);
}
