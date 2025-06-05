use attrchecker::AttributeChecker;
use linter::{Linter, attributes::AttributesLinter};
use typechecker::TypeChecker;

use crate::{
    core::compiler::options::CompilerFile, frontend::types::parser::stmts::stmt::ThrushStatement,
};

pub mod attrchecker;
pub mod linter;
pub mod typechecker;

pub struct SemanticAnalyzer<'semantic_analyzer> {
    type_checker: TypeChecker<'semantic_analyzer>,
    attr_checker: AttributeChecker<'semantic_analyzer>,
    linter: Linter<'semantic_analyzer>,
    attr_linter: AttributesLinter<'semantic_analyzer>,
}

impl<'semantic_analyzer> SemanticAnalyzer<'semantic_analyzer> {
    pub fn new(
        stmts: &'semantic_analyzer [ThrushStatement<'semantic_analyzer>],
        file: &'semantic_analyzer CompilerFile,
    ) -> Self {
        let type_checker: TypeChecker = TypeChecker::new(stmts, file);
        let attr_checker: AttributeChecker = AttributeChecker::new(stmts, file);
        let linter: Linter = Linter::new(stmts, file);
        let attr_linter: AttributesLinter = AttributesLinter::new(stmts, file);

        Self {
            type_checker,
            attr_checker,
            linter,
            attr_linter,
        }
    }

    pub fn check(&mut self, parser_throwed_errors: bool) -> bool {
        if !parser_throwed_errors {
            let type_checker_throw_errors: bool = self.type_checker.check();
            let attr_checker_throw_errors: bool = self.attr_checker.check();

            if !type_checker_throw_errors && !attr_checker_throw_errors && !parser_throwed_errors {
                self.linter.check();
                self.attr_linter.check();
            }

            return type_checker_throw_errors || attr_checker_throw_errors || parser_throwed_errors;
        }

        true
    }
}
