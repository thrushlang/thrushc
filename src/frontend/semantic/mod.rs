use linter::Linter;
use typechecker::TypeChecker;

use crate::{
    middle::types::frontend::parser::stmts::stmt::ThrushStatement, standard::misc::CompilerFile,
};

pub mod linter;
pub mod typechecker;

pub struct SemanticAnalyzer<'semantic_analysis> {
    type_checker: TypeChecker<'semantic_analysis>,
    linter: Linter<'semantic_analysis>,
}

impl<'semantic_analysis> SemanticAnalyzer<'semantic_analysis> {
    pub fn new(
        stmts: &'semantic_analysis [ThrushStatement],
        file: &'semantic_analysis CompilerFile,
    ) -> Self {
        let type_checker: TypeChecker = TypeChecker::new(stmts, file);
        let linter: Linter = Linter::new(stmts, file);

        Self {
            type_checker,
            linter,
        }
    }

    pub fn check(&mut self) -> bool {
        let throw_errors: bool = self.type_checker.check();
        self.linter.check();

        throw_errors
    }
}
