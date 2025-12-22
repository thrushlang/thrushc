use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

pub fn abort_front_end(
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
