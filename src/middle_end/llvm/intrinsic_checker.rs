use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::types::ast::Ast;

use inkwell::intrinsics::Intrinsic;

#[derive(Debug)]
pub struct IntrinsicChecker<'llvm> {
    ast: &'llvm [Ast<'llvm>],
    errors: Vec<CompilationIssue>,
    diagnostician: Diagnostician,
}

impl<'llvm> IntrinsicChecker<'llvm> {
    #[inline]
    pub fn new(
        ast: &'llvm [Ast<'llvm>],
        file: &'llvm CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        Self {
            ast,
            errors: Vec::with_capacity(100),
            diagnostician: Diagnostician::new(file, options),
        }
    }
}

impl<'llvm> IntrinsicChecker<'llvm> {
    pub fn check(&mut self) -> Result<(), ()> {
        for node in self.ast {
            if let Ast::Intrinsic {
                external_name,
                span,
                ..
            } = node
            {
                let intrinsic: Option<Intrinsic> = Intrinsic::find(external_name);

                if intrinsic.is_none()
                    || (intrinsic.is_some_and(|intrinsic| intrinsic.is_overloaded())
                        && external_name.split(".").count() <= 2)
                {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0025,
                        "This intrinsic is not recognized by the compiler as existing. Try another name.".into(),
                        None,
                        *span,
                    ));
                }
            }
        }

        self.verify()?;

        Ok(())
    }
}

impl IntrinsicChecker<'_> {
    #[inline]
    fn verify(&mut self) -> Result<(), ()> {
        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            return Err(());
        }

        Ok(())
    }
}

impl<'llvm> IntrinsicChecker<'llvm> {
    #[inline]
    fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}
