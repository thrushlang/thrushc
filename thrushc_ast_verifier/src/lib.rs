use thrushc_ast::Ast;
use thrushc_diagnostician::Diagnostician;
use thrushc_errors::CompilationIssue;
use thrushc_options::{CompilationUnit, CompilerOptions};

pub struct AstVerifier<'ast_verifier> {
    ast: &'ast_verifier [Ast<'ast_verifier>],
    errors: Vec<CompilationIssue>,
    diagnostician: Diagnostician,
}

impl<'ast_verifier> AstVerifier<'ast_verifier> {
    #[inline]
    pub fn new(
        ast: &'ast_verifier [Ast<'ast_verifier>],
        file: &CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        Self {
            ast,
            errors: Vec::with_capacity(u8::MAX as usize),
            diagnostician: Diagnostician::new(file, options),
        }
    }
}
