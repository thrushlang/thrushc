use crate::core::compiler::options::CompilationUnit;
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::CompilationIssue;

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
    pub fn new(ast: &'llvm [Ast<'llvm>], file: &'llvm CompilationUnit) -> Self {
        Self {
            ast,
            errors: Vec::with_capacity(100),
            diagnostician: Diagnostician::new(file),
        }
    }
}

impl<'llvm> IntrinsicChecker<'llvm> {
    pub fn check(&mut self) -> Result<(), ()> {
        for ast in self.ast {
            if let Ast::Intrinsic {
                external_name,
                span,
                ..
            } = ast
            {
                let intrinsic: Option<Intrinsic> = Intrinsic::find(external_name);

                if intrinsic.is_none()
                    || (intrinsic.is_some_and(|intrinsic| intrinsic.is_overloaded())
                        && external_name.split(".").count() <= 2)
                {
                    self.add_error(CompilationIssue::Error(
                        "Intrinsic not found".into(),
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
    pub fn verify(&mut self) -> Result<(), ()> {
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
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}
