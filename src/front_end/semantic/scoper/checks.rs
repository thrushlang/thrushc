use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::scoper::Scoper;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstStandardExtensions;

pub fn check_for_multiple_terminators(scoper: &mut Scoper, node: &Ast) {
    let Ast::Block { nodes, .. } = node else {
        return;
    };

    if nodes.is_empty() {
        return;
    }

    let return_positions: Vec<(usize, &Ast)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_terminator())
        .collect();

    if return_positions.len() > 1 {
        for (_, node) in &return_positions[1..] {
            scoper.add_error(CompilationIssue::Error(
                "Syntax Error".into(),
                "Only one function terminator is allowed per block. The previous function terminator at an earlier position makes this block unreachable and invalid. Remove it.".into(),
                None,
                node.get_span(),
            ));
        }
    }

    let break_positions: Vec<(usize, &Ast)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_break())
        .collect();

    if break_positions.len() > 1 {
        for (_, node) in &break_positions[1..] {
            scoper.add_error(CompilationIssue::Error(
                "Syntax Error".into(),
                "Only one break loop control terminator is allowed per loop block. Additional break loop control terminators are redundant and disallowed. Remove it.".into(),
                None,
                node.get_span(),
            ));
        }
    }

    let continue_positions: Vec<(usize, &Ast)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_continue())
        .collect();

    if continue_positions.len() > 1 {
        for (_, node) in &continue_positions[1..] {
            scoper.add_error(CompilationIssue::Error(
                "Syntax Error".into(),
                "Only one continue loop control terminator is allowed per loop block. Additional continue loop control terminators are redundant and disallowed. Remove it.".into(),
                None,
                node.get_span()
            ));
        }
    }
}

pub fn check_for_unreachable_code_instructions(scoper: &mut Scoper, node: &Ast) {
    let Ast::Block { nodes, .. } = node else {
        return;
    };

    let total_nodes: usize = nodes.len();

    if total_nodes == 0 {
        return;
    }

    let Some((terminator_idx, _)) = nodes
        .iter()
        .enumerate()
        .find(|(_, stmt)| stmt.is_terminator() || stmt.is_break() || stmt.is_continue())
    else {
        return;
    };

    let unreachable_range: std::ops::Range<usize> = (terminator_idx + 1)..total_nodes;

    for idx in unreachable_range {
        if let Some(unreachable_node) = nodes.get(idx) {
            scoper.add_error(CompilationIssue::Error(
                "Unreachable code".to_string(),
                "This instruction will never be executed because of a previous terminator (return/break/continue). Remove it.".to_string(),
                None,
                unreachable_node.get_span(),
            ));
        }
    }
}
