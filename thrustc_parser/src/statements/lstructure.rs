use thrustc_ast::traits::AstStructureDataExtensions;
use thrustc_ast::{Ast, data::StructureData};
use thrustc_attributes::ThrustAttributes;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::{Type, modificators::StructureTypeModificator};

use crate::{ParserContext, attributes, modificators, traits::StructFieldsExtensions, typegen};

pub fn build_structure<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Struct,
        CompilationIssueCode::E0001,
        "Expected 'struct' keyword.".into(),
    )?;

    let name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
    )?;

    let attributes: ThrustAttributes =
        attributes::build_compiler_attributes(ctx, &[TokenType::LBrace])?;
    let modificator: StructureTypeModificator =
        modificators::build_structure_modificator(&attributes);

    ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
        "Expected '{'.".into(),
    )?;

    let name: &str = name_tk.get_lexeme();
    let span: Span = name_tk.get_span();

    let mut data: StructureData = StructureData::new(name, modificator, span);
    let mut field_position: u32 = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        if ctx.check(TokenType::Identifier) {
            let field_tk: &Token = ctx.consume(
                TokenType::Identifier,
                CompilationIssueCode::E0001,
                "Expected identifier.".into(),
            )?;

            let field_name: &str = field_tk.get_lexeme();
            let field_span: Span = field_tk.get_span();

            ctx.consume(
                TokenType::Colon,
                CompilationIssueCode::E0001,
                "Expected ':'.".into(),
            )?;

            let field_type: Type = typegen::build_type(ctx, false)?;

            data.1
                .push((field_name, field_type, field_position, field_span));

            field_position += 1;

            if ctx.check(TokenType::RBrace) {
                break;
            } else if ctx.match_token(TokenType::Comma)? {
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
                ctx.consume(
                    TokenType::Identifier,
                    CompilationIssueCode::E0001,
                    "Expected identifier.".into(),
                )?;
            }
        } else {
            ctx.consume(
                TokenType::Identifier,
                CompilationIssueCode::E0001,
                "Expected identifier.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RBrace,
        CompilationIssueCode::E0001,
        "Expected '}'.".into(),
    )?;

    let kind: Type = data.get_type();

    if !ctx.is_main_scope() {
        ctx.get_mut_symbols().new_struct(
            name,
            (name, data.1.clone(), attributes.clone(), modificator, span),
            span,
        )?;

        Ok(Ast::Struct {
            name,
            data,
            kind,
            attributes,
            span,
        })
    } else {
        Ok(Ast::invalid_ast(span))
    }
}
