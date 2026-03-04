use thrustc_ast::{Ast, data::EnumData, traits::AstEnumFieldsDataExtensions};
use thrustc_entities::parser::FoundSymbolId;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::{
    ParserContext,
    traits::{EnumExtensions, FoundSymbolEitherExtensions},
};

pub fn build_enum_value<'parser>(
    ctx: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, CompilationIssue> {
    let field_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected enum name.".into(),
    )?;

    let field_span: Span = field_tk.get_span();

    let reference: Result<FoundSymbolId, CompilationIssue> =
        ctx.get_symbols().get_symbols_id(name, span);

    match reference {
        Ok(object) => {
            let enum_id: (&str, usize) = object.expected_enum(span)?;
            let id: &str = enum_id.0;
            let scope_idx: usize = enum_id.1;

            match ctx.get_symbols().get_enum_by_id(id, scope_idx, span) {
                Ok(enum_) => {
                    let data: EnumData = enum_.get_fields();
                    let field_name: &str = field_tk.get_lexeme();

                    match data.get_field(field_name) {
                        Some(field) => {
                            let field_type: Type = field.1;
                            let field_value: Ast = field.2;

                            let canonical_name: String = format!("{}.{}", name, field_name);

                            Ok(Ast::EnumValue {
                                name: canonical_name,
                                value: field_value.into(),
                                kind: field_type,
                                span,
                            })
                        }
                        None => {
                            ctx.add_error(CompilationIssue::Error(
                                CompilationIssueCode::E0028,
                                format!("'{}' not found as field member.", field_name),
                                None,
                                field_span,
                            ));

                            Ok(Ast::invalid_ast(span))
                        }
                    }
                }
                Err(error) => {
                    ctx.add_error(error);
                    Ok(Ast::invalid_ast(span))
                }
            }
        }

        Err(error) => {
            ctx.add_error(error);
            Ok(Ast::invalid_ast(span))
        }
    }
}
