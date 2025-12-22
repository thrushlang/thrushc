use crate::core::{
    console::logging::LoggingType,
    diagnostic::{diagnostician::Diagnostician, span::Span},
    errors::{position::CompilationPosition, standard::CompilationIssue},
};

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
