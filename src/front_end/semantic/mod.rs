use attrchecker::AttributeChecker;
use linter::Linter;
use typechecker::TypeChecker;

use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::front_end::semantic::analyzer::Analyzer;
use crate::front_end::semantic::scoper::Scoper;
use crate::front_end::types::ast::Ast;

pub mod analyzer;
pub mod attrchecker;
pub mod linter;
pub mod scoper;
pub mod typechecker;

#[derive(Debug)]
pub struct SemanticAnalyzer<'semantic_analyzer> {
    type_checker: TypeChecker<'semantic_analyzer>,
    analyzer: Analyzer<'semantic_analyzer>,
    attr_checker: AttributeChecker<'semantic_analyzer>,
    scoper: Scoper<'semantic_analyzer>,

    linter: Linter<'semantic_analyzer>,
}

impl<'semantic_analyzer> SemanticAnalyzer<'semantic_analyzer> {
    #[inline]
    pub fn new(
        ast: &'semantic_analyzer [Ast<'semantic_analyzer>],
        file: &'semantic_analyzer CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        let type_checker: TypeChecker = TypeChecker::new(ast, file, options);
        let analyzer: Analyzer = Analyzer::new(ast, file, options);
        let attr_checker: AttributeChecker = AttributeChecker::new(ast, file, options);
        let scoper: Scoper = Scoper::new(ast, file, options);
        let linter: Linter = Linter::new(ast, file, options);

        Self {
            type_checker,
            analyzer,
            attr_checker,
            scoper,
            linter,
        }
    }
}

impl<'semantic_analyzer> SemanticAnalyzer<'semantic_analyzer> {
    pub fn check(&mut self, parser_throwed_errors: bool) -> bool {
        if parser_throwed_errors {
            return true;
        }

        let type_checker_errors: bool = self.type_checker.start();
        let analyzer_errors: bool = self.analyzer.start();
        let attr_checker_errors: bool = self.attr_checker.start();
        let scoper_errors: bool = self.scoper.start();

        if !type_checker_errors && !analyzer_errors && !attr_checker_errors && !scoper_errors {
            self.linter.check();
        }

        type_checker_errors || analyzer_errors || attr_checker_errors || scoper_errors
    }
}
