use thrushc_ast::{
    Ast,
    traits::{AstCodeLocation, AstScopeExtensions, AstStandardExtensions},
};
use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_options::{CompilationUnit, CompilerOptions};

use crate::context::ScoperContext;

mod checks;
mod context;

#[derive(Debug)]
pub struct Scoper<'scoper> {
    ast: &'scoper [Ast<'scoper>],
    context: ScoperContext,
    errors: Vec<CompilationIssue>,
    diagnostician: Diagnostician,
}

impl<'scoper> Scoper<'scoper> {
    #[inline]
    pub fn new(
        ast: &'scoper [Ast<'scoper>],
        file: &CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        Self {
            ast,
            context: ScoperContext::new(),
            errors: Vec::with_capacity(100),
            diagnostician: Diagnostician::new(file, options),
        }
    }
}

impl<'scoper> Scoper<'scoper> {
    pub fn start(&mut self) -> bool {
        for node in self.ast.iter() {
            self.analyze_global_node(node);
        }

        self.check()
    }
}

impl<'scoper> Scoper<'scoper> {
    fn check(&mut self) -> bool {
        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, thrushc_logging::LoggingType::Error);
            });

            true
        } else {
            false
        }
    }
}

impl<'scoper> Scoper<'scoper> {
    fn analyze_global_node(&mut self, node: &Ast) {
        if !node.is_compatible_with_main_scope() {
            self.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0016,
                "This expression, statement, or declaration should not be in the main scope. It should be in a local scope. Reposition it.".into(),
                None,
                node.get_span(),
            ));
        }

        if let Ast::Function { body, .. } = node {
            let Some(body) = body else {
                return;
            };

            self.get_mut_context().enter_function();
            self.analyze_local_node(body);
            self.get_mut_context().leave_function();
        }
    }

    fn analyze_local_node(&mut self, node: &Ast) {
        if node.is_function() {
            self.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0016,
                "This function should not be in a local scope. It should be in the main scope. Reposition it.".into(),
                None,
                node.get_span(),
            ));
        }

        if node.is_asm_function() {
            self.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0016,
                "This assembler function should not be in a local scope. It should be in the main scope. Reposition it.".into(),
                None,
                node.get_span(),
            ));
        }

        if node.is_custom_type() {
            self.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0016,
                "This type should not be in a local scope. It should be in the main scope. Reposition it.".into(),
                None,
                node.get_span(),
            ));
        }

        if node.is_global_asm() {
            self.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0016,
                "This global module assembler should not be in a local scope. It should be in the main scope. Reposition it.".into(),
                None,
                node.get_span(),
            ));
        }

        if node.is_enum() {
            self.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0016,
                "This enumeration should not be in a local scope. It should be in the main scope. Reposition it.".into(),
                None,
                node.get_span(),
            ));
        }

        if node.is_import() {
            self.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0016,
                "This module import should not be in a local scope. It should be in the main scope. Reposition it.".into(),
                None,
                node.get_span(),
            ));
        }

        if node.is_intrinsic() {
            self.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0016,
                "This compiler intrinsic should not be in a local scope. It should be in the main scope. Reposition it.".into(),
                None,
                node.get_span(),
            ));
        }

        if let Ast::Block { nodes, post, .. } = node {
            checks::check_for_multiple_terminators(self, node);
            checks::check_for_unreachable_code_instructions(self, node);

            for node in nodes.iter() {
                self.analyze_local_node(node);
            }

            for postnode in post.iter() {
                self.analyze_local_node(postnode);
            }
        }

        match node {
            Ast::If {
                block,
                elseif,
                anyway,
                ..
            } => {
                self.analyze_local_node(block);

                for elseif in elseif.iter() {
                    self.analyze_local_node(elseif);
                }

                if let Some(otherwise) = anyway {
                    self.analyze_local_node(otherwise);
                }
            }
            Ast::Elif { block, .. } => {
                self.analyze_local_node(block);
            }
            Ast::Else { block, .. } => {
                self.analyze_local_node(block);
            }

            Ast::While { block, .. } => {
                self.get_mut_context().enter_loop();
                self.analyze_local_node(block);
                self.get_mut_context().leave_loop();
            }
            Ast::Loop { block, .. } => {
                self.get_mut_context().enter_loop();
                self.analyze_local_node(block);
                self.get_mut_context().leave_loop();
            }
            Ast::For { block, .. } => {
                self.get_mut_context().enter_loop();
                self.analyze_local_node(block);
                self.get_mut_context().leave_loop();
            }

            Ast::Continue { .. }
            | Ast::ContinueAll { .. }
            | Ast::Break { .. }
            | Ast::BreakAll { .. } => {
                if !self.get_context().is_inside_loop() {
                    self.add_error(
                        CompilationIssue::Error(
                            CompilationIssueCode::E0017,
                            "Only loop controlers can be inside a loop. The instruction inside a loop was expected. Reposition it inside a loop.".into(),
                            None,
                            node.get_span(),
                        )
                    );
                }
            }
            Ast::Return { span, .. } => {
                if !self.get_context().is_inside_function() {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0018,
                        "Expected function terminator inside of a function. Reposition it.".into(),
                        None,
                        *span,
                    ));
                }
            }

            Ast::Defer { node, .. } => {
                self.analyze_local_node(node);
            }

            _ => (),
        }
    }
}

impl<'scoper> Scoper<'scoper> {
    #[inline]
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}

impl<'scoper> Scoper<'scoper> {
    #[inline]
    fn get_context(&self) -> &ScoperContext {
        &self.context
    }
}

impl<'scoper> Scoper<'scoper> {
    #[inline]
    fn get_mut_context(&mut self) -> &mut ScoperContext {
        &mut self.context
    }
}
