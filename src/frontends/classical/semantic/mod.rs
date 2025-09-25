use attrchecker::AttributeChecker;
use linter::{Linter, attributes::AttributesLinter};
use typechecker::TypeChecker;

use crate::{core::compiler::options::CompilationUnit, frontends::classical::types::ast::Ast};

pub mod attrchecker;
pub mod linter;
pub mod typechecker;

#[derive(Debug)]
pub struct SemanticAnalyzer<'semantic_analyzer> {
    type_checker: TypeChecker<'semantic_analyzer>,
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
        let attr_checker: AttributeChecker = AttributeChecker::new(ast, file);
        let linter: Linter = Linter::new(ast, file);
        let attr_linter: AttributesLinter = AttributesLinter::new(ast, file);

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
