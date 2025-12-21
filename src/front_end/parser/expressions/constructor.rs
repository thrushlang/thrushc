use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::FoundSymbolEither;
use crate::front_end::types::parser::stmts::{
    traits::{ConstructorExtensions, StructExtensions, TokenExtensions},
    types::Constructor,
};
use crate::front_end::types::parser::symbols::types::{FoundSymbolId, Struct};
use crate::front_end::typesystem::modificators::StructureTypeModificator;

pub fn build_constructor<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::New,
        CompilationIssueCode::E0001,
        "Expected 'new' keyword.".into(),
    )?;

    let identifier_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected 'identifier' keyword.".into(),
    )?;

    ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
        "Expected '{'.".into(),
    )?;

    let name: &str = identifier_tk.get_lexeme();
    let span: Span = identifier_tk.get_span();

    let object: FoundSymbolId = ctx.get_symbols().get_symbols_id(name, span)?;
    let structure_id: (&str, usize) = object.expected_struct(span)?;
    let id: &str = structure_id.0;
    let scope_idx: usize = structure_id.1;

    let structure: Struct = ctx.get_symbols().get_struct_by_id(id, scope_idx, span)?;
    let modificator: StructureTypeModificator = structure.get_modificator();

    let required: usize = structure.get_fields().1.len();

    let mut args: Constructor = Vec::with_capacity(10);
    let mut amount: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        if ctx.match_token(TokenType::Identifier)? {
            let field_tk: &Token = ctx.previous();
            let field_span: Span = field_tk.span;
            let field_name: &str = field_tk.get_lexeme();

            ctx.consume(
                TokenType::Colon,
                CompilationIssueCode::E0001,
                "Expected ':'.".into(),
            )?;

            if !structure.contains_field(field_name) {
                return Err(CompilationIssue::Error(
                    CompilationIssueCode::E0001,
                    "Expected existing field name.".into(),
                    None,
                    field_span,
                ));
            }

            if amount >= required {
                return Err(CompilationIssue::Error(
                    CompilationIssueCode::E0026,
                    format!("Expected '{}' fields, not '{}' fields.", required, amount),
                    None,
                    span,
                ));
            }

            let expression: Ast = expressions::build_expr(ctx)?;

            if let Some(target_type) = structure.get_field_type(field_name) {
                args.push((field_name, expression, target_type, amount as u32));
            }

            amount += 1;

            if ctx.check(TokenType::RBrace) {
                break;
            }

            if ctx.match_token(TokenType::Comma)? {
                if ctx.check(TokenType::RBrace) {
                    break;
                }
            } else if ctx.check_to(TokenType::Identifier, 0) {
                ctx.consume(
                    TokenType::Comma,
                    CompilationIssueCode::E0001,
                    "Expected ','.".into(),
                )?;
            } else {
                return Err(CompilationIssue::Error(
                    CompilationIssueCode::E0001,
                    "Expected identifier.".into(),
                    None,
                    ctx.previous().get_span(),
                ));
            }
        } else {
            return Err(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected field name.".into(),
                None,
                span,
            ));
        }
    }

    let provided: usize = args.len();

    if provided != required {
        return Err(CompilationIssue::Error(
            CompilationIssueCode::E0027,
            format!(
                "Expected '{}' arguments, but '{}' was gived.",
                required, provided
            ),
            None,
            span,
        ));
    }

    ctx.consume(
        TokenType::RBrace,
        CompilationIssueCode::E0001,
        "Expected '}'.".into(),
    )?;

    Ok(Ast::Constructor {
        name,
        args: args.clone(),
        kind: args.get_type(name, modificator, span),
        span,
    })
}
