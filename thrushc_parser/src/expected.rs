use std::path::PathBuf;

use thrushc_entities::parser::FoundSymbolId;
use thrushc_errors::{CompilationIssue, CompilationPosition};
use thrushc_span::Span;

use crate::traits::FoundSymbolEitherExtensions;

impl<'parser> FoundSymbolEitherExtensions<'parser> for FoundSymbolId<'parser> {
    fn expected_struct(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue> {
        if let Some(struct_id) = self.0 {
            return Ok(struct_id);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected struct reference"),
            String::from("Expected struct but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_function(&self, span: Span) -> Result<&'parser str, CompilationIssue> {
        if let Some(name) = self.1 {
            return Ok(name);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_enum(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue> {
        if let Some(name) = self.2 {
            return Ok(name);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_static(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue> {
        if let Some(static_id) = self.3 {
            return Ok(static_id);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected static reference"),
            String::from("Expected static but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_constant(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue> {
        if let Some(const_id) = self.4 {
            return Ok(const_id);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected constant reference"),
            String::from("Expected constant but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_custom_type(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue> {
        if let Some(type_id) = self.5 {
            return Ok(type_id);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected custom type reference"),
            String::from("Expected custom type but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_parameter(&self, span: Span) -> Result<&'parser str, CompilationIssue> {
        if let Some(name) = self.6 {
            return Ok(name);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected parameter reference"),
            String::from("Expected parameter but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_asm_function(&self, span: Span) -> Result<&'parser str, CompilationIssue> {
        if let Some(name) = self.7 {
            return Ok(name);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected assembler function reference"),
            String::from("Expected assembler function but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_lli(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue> {
        if let Some((name, scope_idx)) = self.8 {
            return Ok((name, scope_idx));
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected low level instruction reference"),
            String::from("Expected LLI but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_local(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue> {
        if let Some((name, scope_idx)) = self.9 {
            return Ok((name, scope_idx));
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }

    fn expected_intrinsic(&self, span: Span) -> Result<&'parser str, CompilationIssue> {
        if let Some(name) = self.10 {
            return Ok(name);
        }

        Err(CompilationIssue::FrontEndBug(
            String::from("Expected intrinsic reference"),
            String::from("Expected intrinsic but found something else."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ))
    }
}
