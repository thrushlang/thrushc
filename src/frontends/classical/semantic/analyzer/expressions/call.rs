use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{semantic::analyzer::Analyzer, types::ast::Ast},
};

pub fn validate<'analyzer>(
    analyzer: &mut Analyzer<'analyzer>,
    args: &'analyzer [Ast],
) -> Result<(), ThrushCompilerIssue> {
    args.iter().try_for_each(|arg| analyzer.analyze_stmt(arg))?;

    Ok(())
}
