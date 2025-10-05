use attrchecker::AttributeChecker;
use linter::{Linter, attributes::AttributesLinter};
use typechecker::TypeChecker;

use crate::{
    core::compiler::options::CompilationUnit,
    frontends::classical::{semantic::analyzer::Analyzer, types::ast::Ast},
};

pub mod analyzer;
pub mod attrchecker;
pub mod linter;
pub mod typechecker;

#[derive(Debug)]
pub struct SemanticAnalyzer<'semantic_analyzer> {
    type_checker: TypeChecker<'semantic_analyzer>,
    analyzer: Analyzer<'semantic_analyzer>,
    attr_checker: AttributeChecker<'semantic_analyzer>,
    attr_linter: AttributesLinter<'semantic_analyzer>,

    linter: Linter<'semantic_analyzer>,
}

impl<'semantic_analyzer> SemanticAnalyzer<'semantic_analyzer> {
    pub fn new(
        ast: &'semantic_analyzer [Ast<'semantic_analyzer>],
        file: &'semantic_analyzer CompilationUnit,
    ) -> Self {
        let type_checker: TypeChecker = TypeChecker::new(ast, file);
        let analyzer: Analyzer = Analyzer::new(ast, file);
        let attr_checker: AttributeChecker = AttributeChecker::new(ast, file);
        let linter: Linter = Linter::new(ast, file);
        let attr_linter: AttributesLinter = AttributesLinter::new(ast, file);

        Self {
            type_checker,
            analyzer,
            attr_checker,
            linter,
            attr_linter,
        }
    }
}

impl<'semantic_analyzer> SemanticAnalyzer<'semantic_analyzer> {
    pub fn check(&mut self, parser_throwed_errors: bool) -> bool {
        if parser_throwed_errors {
            return true;
        }

        let type_checker_errors: bool = self.type_checker.check();
        let analyzer_errors: bool = self.analyzer.check();
        let attr_checker_errors: bool = self.attr_checker.check();

        if !type_checker_errors && !analyzer_errors && !attr_checker_errors {
            self.linter.check();
            self.attr_linter.check();
        }

        type_checker_errors || analyzer_errors || attr_checker_errors
    }
}
