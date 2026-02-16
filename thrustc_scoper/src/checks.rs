use thrustc_ast::{
    Ast,
    traits::{AstCodeLocation, AstStandardExtensions},
};
use thrustc_errors::{CompilationIssue, CompilationIssueCode};

use crate::Scoper;

pub fn check_for_multiple_terminators(scoper: &mut Scoper, node: &Ast) {
    let Ast::Block { nodes, .. } = node else {
        return;
    };

    if nodes.is_empty() {
        return;
    }

    let unreacheable_positions: Vec<(usize, &Ast)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_unreacheable())
        .collect();

    if unreacheable_positions.len() > 1 {
        for (_, node) in &unreacheable_positions[1..] {
            scoper.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0015,
                "Only one unreacheable instruction is allowed per block. Additional unreacheable instructions are redundant and disallowed. Remove it.".into(),
                None,
                node.get_span(),
            ));
        }
    }

    let return_positions: Vec<(usize, &Ast)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_terminator())
        .collect();

    if return_positions.len() > 1 {
        for (_, node) in &return_positions[1..] {
            scoper.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0015,
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
                CompilationIssueCode::E0015,
                "Only one break loop control terminator is allowed per loop block. Additional break loop control terminators are redundant and disallowed. Remove it.".into(),
                None,
                node.get_span(),
            ));
        }
    }

    let breakall_positions: Vec<(usize, &Ast)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_breakall())
        .collect();

    if breakall_positions.len() > 1 {
        for (_, node) in &breakall_positions[1..] {
            scoper.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0015,
                "Only one breaker all loop control terminator is allowed per loop block. Additional break loop control terminators are redundant and disallowed. Remove it.".into(),
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
                CompilationIssueCode::E0015,
                "Only one continue loop control terminator is allowed per loop block. Additional continue loop control terminators are redundant and disallowed. Remove it.".into(),
                None,
                node.get_span()
            ));
        }
    }

    let continueall_positions: Vec<(usize, &Ast)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, stmt)| stmt.is_continueall())
        .collect();

    if continueall_positions.len() > 1 {
        for (_, node) in &continueall_positions[1..] {
            scoper.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0015,
                "Only one continue all loop control terminator is allowed per loop block. Additional continue loop control terminators are redundant and disallowed. Remove it.".into(),
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

    let Some((terminator_idx, _)) = nodes.iter().enumerate().find(|(_, stmt)| {
        stmt.is_terminator() || stmt.is_unreacheable() || stmt.is_break() || stmt.is_continue()
    }) else {
        return;
    };

    let unreachable_range: std::ops::Range<usize> = (terminator_idx + 1)..total_nodes;

    for idx in unreachable_range {
        if let Some(unreachable_node) = nodes.get(idx) {
            scoper.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0014,
                "This instruction will never be executed because of a previous terminator (return/unreacheable/break/continue). Remove it.".to_string(),
                None,
                unreachable_node.get_span(),
            ));
        }
    }

    let Some((unreacheable_condition_idx, _)) =
        nodes.iter().enumerate().find_map(|(idx, stmt)| match stmt {
            Ast::If {
                block,
                elseif,
                anyway,
                ..
            } => {
                let mut is_if_unreacheable: bool = false;
                let mut is_else_if_unreacheable: bool = false;
                let mut is_else_unreacheable: bool = false;

                {
                    if let Ast::Block { nodes, .. } = block.as_ref() {
                        is_if_unreacheable = nodes.iter().any(|stmt| {
                            stmt.is_terminator()
                                || stmt.is_unreacheable()
                                || stmt.is_break()
                                || stmt.is_breakall()
                                || stmt.is_continue()
                                || stmt.is_continueall()
                        });
                    }
                }

                {
                    for node in elseif {
                        if let Ast::Elif { block, .. } = node {
                            if let Ast::Block { nodes, .. } = &**block {
                                is_else_if_unreacheable = nodes.iter().any(|stmt| {
                                    stmt.is_terminator()
                                        || stmt.is_unreacheable()
                                        || stmt.is_break()
                                        || stmt.is_breakall()
                                        || stmt.is_continue()
                                        || stmt.is_continueall()
                                });
                            }
                        }
                    }
                }

                {
                    if let Some(otherwise) = anyway {
                        if let Ast::Else { block, .. } = &**otherwise {
                            if let Ast::Block { nodes, .. } = block.as_ref() {
                                is_else_unreacheable = nodes.iter().any(|stmt| {
                                    stmt.is_terminator()
                                        || stmt.is_unreacheable()
                                        || stmt.is_break()
                                        || stmt.is_breakall()
                                        || stmt.is_continue()
                                        || stmt.is_continueall()
                                });
                            }
                        }
                    }
                }

                let is_unreacheable_if_else: bool =
                    is_if_unreacheable && is_else_unreacheable && elseif.is_empty();

                let is_full_unreacheable: bool =
                    is_if_unreacheable && is_else_if_unreacheable && is_else_unreacheable;

                if is_unreacheable_if_else || is_full_unreacheable {
                    Some((idx, stmt))
                } else {
                    None
                }
            }
            _ => None,
        })
    else {
        return;
    };

    let unreachable_range: std::ops::Range<usize> = (unreacheable_condition_idx + 1)..total_nodes;

    for idx in unreachable_range {
        if let Some(unreachable_node) = nodes.get(idx) {
            scoper.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0014,
                "This instruction will never be executed due to a conditional pattern with terminators (return/unreacheable/break/continue). Remove it.".to_string(),
                None,
                unreachable_node.get_span(),
            ));
        }
    }
}
