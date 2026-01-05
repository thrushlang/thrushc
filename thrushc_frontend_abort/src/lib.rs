use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationPosition};
use thrushc_logging::LoggingType;
use thrushc_span::Span;

pub fn abort_compilation(
    diagnostician: &mut Diagnostician,
    position: CompilationPosition,
    message: &str,
    span: Span,
    file: std::path::PathBuf,
    line: u32,
) -> ! {
    diagnostician.dispatch_diagnostic(
        &CompilationIssue::FrontEndBug(
            "Failed to Compile".into(),
            message.into(),
            span,
            position,
            file,
            line,
        ),
        LoggingType::FronteEndBug,
    );

    std::process::exit(1);
}
