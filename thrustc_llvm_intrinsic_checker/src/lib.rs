/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/


use inkwell::intrinsics::Intrinsic;

use thrustc_ast::Ast;
use thrustc_diagnostician::Diagnostician;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_options::{CompilationUnit, CompilerOptions};

#[derive(Debug)]
pub struct LLVMIntrinsicChecker<'llvm> {
    ast: &'llvm [Ast<'llvm>],
    errors: Vec<CompilationIssue>,
    diagnostician: Diagnostician,
}

impl<'llvm> LLVMIntrinsicChecker<'llvm> {
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

impl<'llvm> LLVMIntrinsicChecker<'llvm> {
    pub fn analyze(&mut self) -> bool {
        {
            for node in self.ast.iter() {
                if let Ast::Intrinsic {
                    external_name,
                    span,
                    ..
                } = node
                {
                    let ffi_name: &str = external_name.trim();
                    let intrinsic: Option<Intrinsic> = Intrinsic::find(ffi_name);
                    let is_bad_overloaded: bool = intrinsic
                        .is_some_and(|intrinsic| intrinsic.is_overloaded())
                        && ffi_name.split(".").count() <= 2;

                    if is_bad_overloaded {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0034,
                            "This overloaded compiler intrinsic has an invalid syntax. Try to fix it using the correct points.".into(),
                            None,
                            *span,
                        ));
                    }

                    if intrinsic.is_none() {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0025,
                            "This compiler intrinsic is not recognized. Try another name.".into(),
                            None,
                            *span,
                        ));
                    }
                }
            }
        }

        self.verify()
    }
}

impl LLVMIntrinsicChecker<'_> {
    #[inline]
    fn verify(&mut self) -> bool {
        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, thrustc_logging::LoggingType::Error);
            });

            true
        } else {
            false
        }
    }
}

impl<'llvm> LLVMIntrinsicChecker<'llvm> {
    #[inline]
    fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}
