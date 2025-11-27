use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstStandardExtensions;

pub fn check_for_multiple_terminators(analyzer: &mut Analyzer, node: &Ast) {
    let Ast::Block { stmts, .. } = node else {
        return;
    };

    if stmts.is_empty() {
        return;
    }

    let return_positions: Vec<(usize, &Ast)> = stmts
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_terminator())
        .collect();

    if return_positions.len() > 1 {
        for (_, node) in &return_positions[1..] {
            analyzer.add_error(CompilationIssue::Error(
                "Syntax Error".into(),
                "Only one 'return' terminator is allowed per block. Previous 'return' at earlier position makes this unreachable and invalid.".into(),
                None,
                node.get_span(),
            ));
        }
    }

    let break_positions: Vec<(usize, &Ast)> = stmts
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_break())
        .collect();

    if break_positions.len() > 1 {
        for (_, node) in &break_positions[1..] {
            analyzer.add_error(CompilationIssue::Error(
                "Syntax Error".into(),
                "Only one 'break' terminator is allowed per loop block. Additional 'break' terminators are redundant and disallowed.".into(),
                None,
                node.get_span(),
            ));
        }
    }

    let continue_positions: Vec<(usize, &Ast)> = stmts
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_continue())
        .collect();

    if continue_positions.len() > 1 {
        for (_, node) in &continue_positions[1..] {
            analyzer.add_error(CompilationIssue::Error(
                "Syntax Error".into(),
                "Only one 'continue' terminator is allowed per loop block. Additional 'continue' terminators are redundant and disallowed.".into(),
                None,
                node.get_span()
            ));
        }
    }
}

pub fn check_for_unreachable_code_instructions(analyzer: &mut Analyzer, node: &Ast) {
    let Ast::Block { stmts, .. } = node else {
        return;
    };

    let total_nodes: usize = stmts.len();

    if total_nodes <= 1 {
        return;
    }

    let Some((terminator_idx, _)) = stmts
        .iter()
        .enumerate()
        .find(|(_, stmt)| stmt.is_terminator() || stmt.is_break() || stmt.is_continue())
    else {
        return;
    };

    let unreachable_range: std::ops::Range<usize> = (terminator_idx + 1)..total_nodes;

    for idx in unreachable_range {
        if let Some(unreachable_node) = stmts.get(idx) {
            analyzer.add_error(CompilationIssue::Error(
                "Unreachable code".to_string(),
                "This instruction will never be executed because of a previous terminator (return/break/continue).".to_string(),
                None,
                unreachable_node.get_span(),
            ));
        }
    }
}
