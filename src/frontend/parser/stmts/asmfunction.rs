use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, expression, typegen},
        types::{
            ast::Ast,
            lexer::ThrushType,
            parser::{
                stmts::{
                    traits::{ThrushAttributesExtensions, TokenExtensions},
                    types::ThrushAttributes,
                },
                symbols::types::ParametersTypes,
            },
        },
    },
};

pub fn build_assembler_function<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenType::AsmFn,
        String::from("Syntax error"),
        String::from("Expected 'asmfn' keyword."),
    )?;

    let asm_function_name_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name to the function."),
    )?;

    let asm_function_name: &str = asm_function_name_tk.get_lexeme();
    let asm_function_ascii_name: &str = asm_function_name_tk.get_ascii_lexeme();

    let span: Span = asm_function_name_tk.get_span();

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Assembler functions can only be defined globally."),
            None,
            span,
        ));
    }

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut parameters: Vec<Ast> = Vec::with_capacity(10);
    let mut parameters_types: Vec<ThrushType> = Vec::with_capacity(10);

    let mut parameter_position: u32 = 0;

    loop {
        if parser_ctx.check(TokenType::RParen) {
            break;
        }

        let parameter_name_tk: &'parser Token = parser_ctx.consume(
            TokenType::Identifier,
            String::from("Syntax error"),
            String::from("Expected 'identifier'."),
        )?;

        let parameter_name: &str = parameter_name_tk.get_lexeme();
        let parameter_span: Span = parameter_name_tk.get_span();

        let parameter_type: ThrushType = typegen::build_type(parser_ctx)?;

        parameters_types.push(parameter_type.clone());

        parameters.push(Ast::AssemblerFunctionParameter {
            name: parameter_name,
            kind: parameter_type,
            position: parameter_position,
            span: parameter_span,
        });

        parameter_position += 1;

        if parser_ctx.check(TokenType::RParen) {
            break;
        } else {
            parser_ctx.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    let return_type: ThrushType = typegen::build_type(parser_ctx)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(parser_ctx, &[TokenType::LBrace])?;

    let is_public: bool = attributes.has_public_attribute();

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut assembler: String = String::with_capacity(100);
    let mut assembler_pos: usize = 0;

    loop {
        if parser_ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expression::build_expr(parser_ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let assembly: &str = raw_str.get_str_content()?;

        if assembler_pos != 0 {
            assembler.push('\n');
        }

        assembler.push_str(assembly);

        if parser_ctx.check(TokenType::RBrace) {
            break;
        } else {
            parser_ctx.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }

        assembler_pos += 1;
    }

    parser_ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut constraints: String = String::with_capacity(100);
    let mut constraint_pos: usize = 0;

    loop {
        if parser_ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expression::build_expr(parser_ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let constraint: &str = raw_str.get_str_content()?;

        if constraint_pos != 0 {
            constraints.push('\n');
        }

        constraints.push_str(constraint);

        if parser_ctx.check(TokenType::RBrace) {
            break;
        } else {
            parser_ctx.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }

        constraint_pos += 1;
    }

    parser_ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if declare_forward {
        if let Err(error) = parser_ctx.get_mut_symbols().new_asm_function(
            asm_function_name,
            (
                return_type,
                ParametersTypes::new(parameters_types),
                is_public,
            ),
            span,
        ) {
            parser_ctx.add_error(error);
        }

        return Ok(Ast::Null { span });
    }

    Ok(Ast::AssemblerFunction {
        name: asm_function_name,
        ascii_name: asm_function_ascii_name,
        parameters,
        parameters_types,
        assembler,
        constraints,
        return_type,
        attributes,
        span,
    })
}
