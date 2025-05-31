use attrchecker::AttributeChecker;
use linter::{Linter, attributes::AttributesLinter};
use typechecker::TypeChecker;

use crate::{standard::misc::CompilerFile, types::frontend::parser::stmts::stmt::ThrushStatement};

pub mod attrchecker;
pub mod linter;
pub mod typechecker;

pub struct SemanticAnalyzer<'semantic_analysis> {
    type_checker: TypeChecker<'semantic_analysis>,
    attr_checker: AttributeChecker<'semantic_analysis>,
    linter: Linter<'semantic_analysis>,
    attr_linter: AttributesLinter<'semantic_analysis>,
}

impl<'semantic_analysis> SemanticAnalyzer<'semantic_analysis> {
    pub fn new(
        stmts: &'semantic_analysis [ThrushStatement],
        file: &'semantic_analysis CompilerFile,
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

    pub fn check(&mut self) -> bool {
        let type_checker_throw_errors: bool = self.type_checker.check();
        let attr_checker_throw_errors: bool = self.attr_checker.check();

        self.linter.check();
        self.attr_linter.check();

        type_checker_throw_errors || attr_checker_throw_errors
    }
}
