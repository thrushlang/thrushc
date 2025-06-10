use std::rc::Rc;

use ahash::AHashMap as HashMap;

use crate::{
    backend::llvm::compiler::{attributes::LLVMAttribute, conventions::CallConvention},
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        types::{
            lexer::{
                MethodsApplicant, ThrushType, generate_methods,
                traits::ThrushTypeStructTypeExtensions,
            },
            parser::stmts::{
                stmt::ThrushStatement,
                traits::{StructFieldsExtensions, ThrushAttributesExtensions, TokenExtensions},
                types::{CustomTypeFields, EnumFields, StructFields, ThrushAttributes},
            },
            symbols::types::{Methods, ParametersTypes},
        },
    },
    lazy_static,
};

use super::{
    ParserContext,
    contexts::{InstructionPosition, MethodsType, SyncPosition},
    expression, typegen,
};

const CALL_CONVENTIONS_CAPACITY: usize = 10;

lazy_static! {
    static ref CALL_CONVENTIONS: HashMap<&'static [u8], CallConvention> = {
        let mut call_conventions: HashMap<&'static [u8], CallConvention> =
            HashMap::with_capacity(CALL_CONVENTIONS_CAPACITY);

        call_conventions.insert(b"C", CallConvention::Standard);
        call_conventions.insert(b"fast", CallConvention::Fast);
        call_conventions.insert(b"tail", CallConvention::Tail);
        call_conventions.insert(b"cold", CallConvention::Cold);
        call_conventions.insert(b"weakReg", CallConvention::PreserveMost);
        call_conventions.insert(b"strongReg", CallConvention::PreserveAll);
        call_conventions.insert(b"swift", CallConvention::Swift);
        call_conventions.insert(b"haskell", CallConvention::GHC);
        call_conventions.insert(b"erlang", CallConvention::HiPE);

        call_conventions
    };
}

pub fn parse<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Declaration);

    let statement: Result<ThrushStatement<'instr>, ThrushCompilerIssue> =
        match &parser_ctx.peek().kind {
            TokenType::Type => Ok(self::build_custom_type(parser_ctx, false)?),
            TokenType::Struct => Ok(self::build_struct(parser_ctx, false)?),
            TokenType::Enum => Ok(self::build_enum(parser_ctx, false)?),
            TokenType::Fn => Ok(self::build_function(parser_ctx, false)?),
            TokenType::AsmFn => Ok(self::build_assembler_function(parser_ctx, false)?),
            TokenType::Const => Ok(self::build_const(parser_ctx, false)?),
            TokenType::Methods => Ok(self::build_methods(parser_ctx, false)?),

            _ => Ok(statement(parser_ctx)?),
        };

    statement
}

fn statement<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Statement);

    let statement: Result<ThrushStatement<'instr>, ThrushCompilerIssue> =
        match &parser_ctx.peek().kind {
            TokenType::LBrace => Ok(self::build_block(parser_ctx)?),
            TokenType::Return => Ok(self::build_return(parser_ctx)?),
            TokenType::Local => Ok(self::build_local(parser_ctx)?),
            TokenType::Instr => Ok(self::build_instr(parser_ctx)?),
            TokenType::For => Ok(self::build_for_loop(parser_ctx)?),
            TokenType::If => Ok(self::build_conditional(parser_ctx)?),
            TokenType::While => Ok(self::build_while_loop(parser_ctx)?),
            TokenType::Continue => Ok(self::build_continue(parser_ctx)?),
            TokenType::Break => Ok(self::build_break(parser_ctx)?),
            TokenType::Loop => Ok(self::build_loop(parser_ctx)?),

            _ => Ok(expression::build_expression(parser_ctx)?),
        };

    statement
}

pub fn build_methods<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare_forward: bool,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_instr_position(InstructionPosition::Methods);

    let bindings_tk: &Token = parser_ctx.consume(
        TokenType::Methods,
        String::from("Syntax error"),
        String::from("Expected 'methods' keyword."),
    )?;

    let span: Span = bindings_tk.get_span();

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Methods are only defined globally."),
            None,
            span,
        ));
    }

    let kind: ThrushType = typegen::build_type(parser_ctx)?;

    if !kind.is_struct_type() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected struct type."),
            None,
            span,
        ));
    }

    parser_ctx
        .get_mut_type_ctx()
        .set_this_methods_type(MethodsType::Struct(kind.clone()));

    let struct_name: String = kind.parser_get_struct_name(span)?;

    parser_ctx
        .get_symbols()
        .contains_structure(&struct_name, span)?;

    let mut methods: Vec<ThrushStatement> = Vec::with_capacity(50);

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    while parser_ctx.peek().kind != TokenType::RBrace {
        let bind: ThrushStatement = self::build_method(declare_forward, parser_ctx)?;

        parser_ctx
            .get_mut_control_ctx()
            .set_sync_position(SyncPosition::Declaration);

        parser_ctx
            .get_mut_control_ctx()
            .set_instr_position(InstructionPosition::Methods);

        methods.push(bind);
    }

    parser_ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    parser_ctx
        .get_mut_type_ctx()
        .set_this_methods_type(MethodsType::NoRelevant);

    parser_ctx
        .get_mut_control_ctx()
        .set_instr_position(InstructionPosition::NoRelevant);

    if declare_forward {
        let bindings_generated: Methods = generate_methods(methods.clone())?;

        parser_ctx.get_mut_symbols().add_methods(
            &struct_name,
            bindings_generated,
            MethodsApplicant::Struct,
            span,
        )?;

        return Ok(ThrushStatement::Null { span });
    }

    Ok(ThrushStatement::Methods {
        name: struct_name,
        methods,
        span,
    })
}

fn build_method<'instr>(
    declare_forward: bool,
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Statement);

    parser_ctx.consume(
        TokenType::Fn,
        String::from("Syntax error"),
        String::from("Expected 'fn' keyword."),
    )?;

    let bind_name_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name to the method definition."),
    )?;

    let bind_name: &str = bind_name_tk.get_lexeme();
    let span: Span = bind_name_tk.get_span();

    if !parser_ctx
        .get_control_ctx()
        .get_instr_position()
        .is_methods()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected method definition inside the methods context definition."),
            None,
            bind_name_tk.span,
        ));
    }

    parser_ctx
        .get_mut_control_ctx()
        .set_instr_position(InstructionPosition::Method);

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut bind_parameters: Vec<ThrushStatement> = Vec::with_capacity(10);
    let mut bind_parameters_types: Vec<ThrushType> = Vec::with_capacity(10);
    let mut bind_position: u32 = 0;

    let mut this_is_declare_forwardd: bool = false;

    while !parser_ctx.match_token(TokenType::RParen)? {
        if parser_ctx.match_token(TokenType::Comma)? {
            continue;
        }

        if this_is_declare_forwardd && parser_ctx.check(TokenType::This) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from(
                    "'This' keyword is already declare_forwardd. Multiple instances are not allowed.",
                ),
                None,
                bind_name_tk.get_span(),
            ));
        }

        if parser_ctx.check(TokenType::This) {
            let this_tk: &Token = parser_ctx.consume(
                TokenType::This,
                String::from("Syntax error"),
                String::from("Expected 'this' keyword."),
            )?;

            let is_mutable: bool = parser_ctx.match_token(TokenType::Mut)?;

            bind_parameters.push(ThrushStatement::This {
                kind: parser_ctx
                    .get_type_ctx()
                    .get_this_methods_type()
                    .dissamble(),
                is_mutable,
                span: this_tk.get_span(),
            });

            this_is_declare_forwardd = true;

            continue;
        }

        let is_mutable: bool = parser_ctx.match_token(TokenType::Mut)?;

        let parameter_tk: &Token = parser_ctx.consume(
            TokenType::Identifier,
            String::from("Syntax error"),
            String::from("Expected parameter name."),
        )?;

        let parameter_name: &str = parameter_tk.get_lexeme();
        let parameter_span: Span = parameter_tk.get_span();

        parser_ctx.consume(
            TokenType::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let parameter_type: ThrushType = typegen::build_type(parser_ctx)?;

        if !declare_forward {
            /*parser_ctx.get_mut_symbols().new_parameter(
                parameter_name,
                (parameter_type.clone(), false, is_mutable, parameter_span),
                parameter_span,
            )?;*/
        }

        bind_parameters_types.push(parameter_type.clone());

        bind_parameters.push(ThrushStatement::BindParameter {
            name: parameter_name,
            kind: parameter_type,
            position: bind_position,
            is_mutable,
            span: parameter_span,
        });

        bind_position += 1;
    }

    let return_type: ThrushType = typegen::build_type(parser_ctx)?;

    parser_ctx
        .get_mut_type_ctx()
        .set_function_type(return_type.clone());

    let bind_attributes: ThrushAttributes =
        self::build_attributes(parser_ctx, &[TokenType::LBrace])?;

    if !declare_forward {
        parser_ctx.get_mut_control_ctx().set_inside_bind(true);

        parser_ctx
            .get_mut_type_ctx()
            .set_bind_instance(this_is_declare_forwardd);

        let bind_body: ThrushStatement = self::build_block(parser_ctx)?;

        parser_ctx.get_mut_symbols().end_parameters();
        parser_ctx.get_mut_control_ctx().set_inside_bind(false);
        parser_ctx.get_mut_type_ctx().set_bind_instance(false);

        parser_ctx
            .get_mut_control_ctx()
            .set_instr_position(InstructionPosition::NoRelevant);

        return Ok(ThrushStatement::Method {
            name: bind_name,
            parameters: bind_parameters,
            parameters_types: bind_parameters_types,
            body: bind_body.into(),
            return_type,
            attributes: bind_attributes,
            span,
        });
    }

    parser_ctx
        .get_mut_control_ctx()
        .set_instr_position(InstructionPosition::NoRelevant);

    Ok(ThrushStatement::Null { span })
}

fn build_entry_point<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let span: Span = parser_ctx.previous().span;

    if parser_ctx.get_control_ctx().get_entrypoint() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Duplicated entrypoint"),
            String::from("The language not support two entrypoints. :>"),
            None,
            parser_ctx.previous().get_span(),
        ));
    }

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    parser_ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    parser_ctx.get_mut_control_ctx().set_entrypoint(true);

    Ok(ThrushStatement::EntryPoint {
        body: build_block(parser_ctx)?.into(),
        span,
    })
}

fn build_for_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let for_tk: &Token = parser_ctx.consume(
        TokenType::For,
        String::from("Syntax error"),
        String::from("Expected 'for' keyword."),
    )?;

    let span: Span = for_tk.span;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("For loop must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    let local: ThrushStatement = self::build_local(parser_ctx)?;
    let cond: ThrushStatement = expression::build_expression(parser_ctx)?;
    let actions: ThrushStatement = expression::build_expression(parser_ctx)?;

    parser_ctx.get_mut_control_ctx().set_inside_loop(true);

    let body: ThrushStatement = self::build_block(parser_ctx)?;

    parser_ctx.get_mut_control_ctx().set_inside_loop(false);

    Ok(ThrushStatement::For {
        local: local.into(),
        cond: cond.into(),
        actions: actions.into(),
        block: body.into(),
        span,
    })
}

fn build_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let loop_tk: &Token = parser_ctx.consume(
        TokenType::Loop,
        String::from("Syntax error"),
        String::from("Expected 'loop' keyword."),
    )?;

    let loop_span: Span = loop_tk.span;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            loop_span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Loop must be placed inside a function or a bind."),
            None,
            loop_span,
        ));
    }

    parser_ctx.get_mut_control_ctx().set_inside_loop(true);

    let block: ThrushStatement = self::build_block(parser_ctx)?;

    let scope: usize = parser_ctx.get_scope();

    if !block.has_break() && !block.has_return() && !block.has_continue() {
        parser_ctx
            .get_mut_control_ctx()
            .set_unreacheable_code_scope(scope);
    }

    parser_ctx.get_mut_control_ctx().set_inside_loop(false);

    Ok(ThrushStatement::Loop {
        block: block.into(),
        span: loop_span,
    })
}

fn build_while_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let while_tk: &Token = parser_ctx.consume(
        TokenType::While,
        String::from("Syntax error"),
        String::from("Expected 'while' keyword."),
    )?;

    let span: Span = while_tk.get_span();

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("While loop must be placed inside a function or a structure method."),
            None,
            span,
        ));
    }

    let cond: ThrushStatement = expression::build_expr(parser_ctx)?;
    let block: ThrushStatement = self::build_block(parser_ctx)?;

    Ok(ThrushStatement::While {
        cond: cond.into(),
        block: block.into(),
        span,
    })
}

fn build_continue<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let continue_tk: &Token = parser_ctx.consume(
        TokenType::Continue,
        String::from("Syntax error"),
        String::from("Expected 'continue' keyword."),
    )?;

    let span: Span = continue_tk.span;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Continue must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    let scope: usize = parser_ctx.get_scope();

    parser_ctx
        .get_mut_control_ctx()
        .set_unreacheable_code_scope(scope);

    if !parser_ctx.get_control_ctx().get_inside_loop() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("The flow changer of a loop must go inside one."),
            None,
            span,
        ));
    }

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(ThrushStatement::Continue { span })
}

fn build_break<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let break_tk: &Token = parser_ctx.consume(
        TokenType::Break,
        String::from("Syntax error"),
        String::from("Expected 'break' keyword."),
    )?;

    let span: Span = break_tk.get_span();

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    let scope: usize = parser_ctx.get_scope();

    parser_ctx
        .get_mut_control_ctx()
        .set_unreacheable_code_scope(scope);

    if !parser_ctx.get_control_ctx().get_inside_loop() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("The flow changer of a loop must go inside one."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Break must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(ThrushStatement::Break { span })
}

fn build_conditional<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let if_tk: &Token = parser_ctx.consume(
        TokenType::If,
        String::from("Syntax error"),
        String::from("Expected 'if' keyword."),
    )?;

    let span: Span = if_tk.get_span();

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Conditionals must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    let if_condition: ThrushStatement = expression::build_expr(parser_ctx)?;
    let if_body: ThrushStatement = self::build_block(parser_ctx)?;

    let mut elfs: Vec<ThrushStatement> = Vec::with_capacity(10);

    while parser_ctx.match_token(TokenType::Elif)? {
        let span: Span = parser_ctx.previous().span;

        let elif_condition: ThrushStatement = expression::build_expr(parser_ctx)?;

        let elif_body: ThrushStatement = self::build_block(parser_ctx)?;

        if !elif_body.has_block() {
            continue;
        }

        elfs.push(ThrushStatement::Elif {
            cond: elif_condition.into(),
            block: elif_body.into(),
            span,
        });
    }

    let mut otherwise: Option<Rc<ThrushStatement>> = None;

    if parser_ctx.match_token(TokenType::Else)? {
        let span: Span = parser_ctx.previous().span;
        let else_body: ThrushStatement = self::build_block(parser_ctx)?;

        if else_body.has_block() {
            otherwise = Some(
                ThrushStatement::Else {
                    block: else_body.into(),
                    span,
                }
                .into(),
            );
        }
    }

    Ok(ThrushStatement::If {
        cond: if_condition.into(),
        block: if_body.into(),
        elfs,
        otherwise,
        span,
    })
}

pub fn build_custom_type<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare_forward: bool,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let type_tk: &Token = parser_ctx.consume(
        TokenType::Type,
        String::from("Syntax error"),
        String::from("Expected 'type' keyword."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Types are only defined globally."),
            None,
            type_tk.get_span(),
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected type name."),
    )?;

    let custom_type_name: &str = name.get_lexeme();

    let span: Span = name.get_span();

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let custom_type_attributes: ThrushAttributes =
        self::build_attributes(parser_ctx, &[TokenType::LBrace])?;

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut custom_type_fields: CustomTypeFields = Vec::with_capacity(10);

    while parser_ctx.peek().kind != TokenType::RBrace {
        let kind: ThrushType = typegen::build_type(parser_ctx)?;
        custom_type_fields.push(kind);
    }

    parser_ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if declare_forward {
        if let Err(error) = parser_ctx.get_mut_symbols().new_custom_type(
            custom_type_name,
            (custom_type_fields, custom_type_attributes),
            span,
        ) {
            parser_ctx.add_error(error);
        }
    }

    Ok(ThrushStatement::Null { span })
}

pub fn build_enum<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare_forward: bool,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let enum_tk: &Token = parser_ctx.consume(
        TokenType::Enum,
        String::from("Syntax error"),
        String::from("Expected 'enum'."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Enums are only defined globally."),
            None,
            enum_tk.get_span(),
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected enum name."),
    )?;

    let enum_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let enum_attributes: ThrushAttributes =
        self::build_attributes(parser_ctx, &[TokenType::LBrace])?;

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut enum_fields: EnumFields = Vec::with_capacity(10);

    let mut default_float_value: f64 = 0.0;
    let mut default_integer_value: u64 = 0;

    loop {
        if parser_ctx.check(TokenType::RBrace) {
            break;
        }

        if parser_ctx.match_token(TokenType::Identifier)? {
            let field_tk: &Token = parser_ctx.previous();

            let name: &str = field_tk.get_lexeme();
            let span: Span = field_tk.get_span();

            parser_ctx.consume(
                TokenType::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            let field_type: ThrushType = typegen::build_type(parser_ctx)?;

            if !field_type.is_numeric() {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected integer, boolean, char or floating-point types."),
                    None,
                    span,
                ));
            }

            if parser_ctx.match_token(TokenType::SemiColon)? {
                let field_value: ThrushStatement = if field_type.is_float_type() {
                    ThrushStatement::new_float(field_type, default_float_value, false, span)
                } else if field_type.is_bool_type() {
                    ThrushStatement::new_boolean(field_type, default_integer_value, span)
                } else if field_type.is_char_type() {
                    if default_integer_value > char::MAX as u64 {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Char overflow."),
                            None,
                            span,
                        ));
                    }

                    ThrushStatement::new_char(field_type, default_integer_value, span)
                } else {
                    ThrushStatement::new_integer(field_type, default_integer_value, false, span)
                };

                enum_fields.push((name, field_value));

                default_float_value += 1.0;
                default_integer_value += 1;

                continue;
            }

            parser_ctx.consume(
                TokenType::Eq,
                String::from("Syntax error"),
                String::from("Expected '='."),
            )?;

            let expression: ThrushStatement = expression::build_expr(parser_ctx)?;

            expression.throw_attemping_use_jit(expression.get_span())?;

            parser_ctx.consume(
                TokenType::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            enum_fields.push((name, expression));

            continue;
        }

        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected identifier in enum field."),
            None,
            parser_ctx.advance()?.get_span(),
        ));
    }

    parser_ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    if declare_forward {
        if let Err(error) =
            parser_ctx
                .get_mut_symbols()
                .new_enum(enum_name, (enum_fields, enum_attributes), span)
        {
            parser_ctx.add_error(error);
        }

        return Ok(ThrushStatement::Null { span });
    }

    Ok(ThrushStatement::Enum {
        name: enum_name,
        fields: enum_fields,
        span,
    })
}

pub fn build_struct<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare_forward: bool,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let struct_tk: &Token = parser_ctx.consume(
        TokenType::Struct,
        String::from("Syntax error"),
        String::from("Expected 'struct' keyword."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Structs are only defined globally."),
            None,
            struct_tk.get_span(),
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected structure name."),
    )?;

    let struct_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let attributes: ThrushAttributes = self::build_attributes(parser_ctx, &[TokenType::LBrace])?;

    parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut fields_types: StructFields = (struct_name, Vec::with_capacity(10));
    let mut field_position: u32 = 0;

    loop {
        if parser_ctx.check(TokenType::RBrace) {
            break;
        }

        if parser_ctx.check(TokenType::Identifier) {
            let field_tk: &Token = parser_ctx.consume(
                TokenType::Identifier,
                String::from("Syntax error"),
                String::from("Expected identifier."),
            )?;

            let field_name: &str = field_tk.get_lexeme();
            let field_span: Span = field_tk.get_span();

            parser_ctx.consume(
                TokenType::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            let field_type: ThrushType = typegen::build_type(parser_ctx)?;

            fields_types
                .1
                .push((field_name, field_type, field_position, field_span));

            field_position += 1;

            if parser_ctx.check(TokenType::RBrace) {
                break;
            } else if parser_ctx.match_token(TokenType::Comma)? {
                if parser_ctx.check(TokenType::RBrace) {
                    break;
                }
            } else if parser_ctx.check_to(TokenType::Identifier, 0) {
                parser_ctx.consume(
                    TokenType::Comma,
                    String::from("Syntax error"),
                    String::from("Expected ','."),
                )?;
            } else {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected identifier."),
                    None,
                    parser_ctx.previous().get_span(),
                ));
            }
        } else {
            parser_ctx.only_advance()?;

            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected structure fields identifiers."),
                None,
                parser_ctx.previous().get_span(),
            ));
        }
    }

    parser_ctx.consume(
        TokenType::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if declare_forward {
        if let Err(error) = parser_ctx.get_mut_symbols().new_struct(
            struct_name,
            (
                struct_name,
                fields_types.1,
                attributes,
                Vec::with_capacity(100),
            ),
            span,
        ) {
            parser_ctx.add_error(error);
        }

        return Ok(ThrushStatement::Null { span });
    }

    Ok(ThrushStatement::Struct {
        name: struct_name,
        fields: fields_types.clone(),
        kind: fields_types.get_type(),
        attributes,
        span,
    })
}

pub fn build_const<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare_forward: bool,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenType::Const,
        String::from("Syntax error"),
        String::from("Expected 'const' keyword."),
    )?;

    let const_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Constants are only defined globally."),
            None,
            const_tk.span,
        ));
    }

    let name: &str = const_tk.get_lexeme();
    let span: Span = const_tk.get_span();

    parser_ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let const_type: ThrushType = typegen::build_type(parser_ctx)?;

    let const_attributes: ThrushAttributes = self::build_attributes(parser_ctx, &[TokenType::Eq])?;

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: ThrushStatement = expression::build_expr(parser_ctx)?;

    value.throw_attemping_use_jit(span)?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    if declare_forward {
        if let Err(error) =
            parser_ctx
                .get_mut_symbols()
                .new_constant(name, (const_type, const_attributes), span)
        {
            parser_ctx.add_error(error);
        }

        return Ok(ThrushStatement::Null { span });
    }

    Ok(ThrushStatement::Const {
        name,
        kind: const_type,
        value: value.into(),
        attributes: const_attributes,
        span,
    })
}

fn build_instr<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let instr_tk: &Token = parser_ctx.consume(
        TokenType::Instr,
        String::from("Syntax error"),
        String::from("Expected 'instr' keyword."),
    )?;

    let span: Span = instr_tk.get_span();

    if parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("LLI's should be contained at local scope."),
            None,
            span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    let instr_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name."),
    )?;

    let name: &str = instr_tk.get_lexeme();
    let span: Span = instr_tk.get_span();

    parser_ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let instr_type: ThrushType = typegen::build_type(parser_ctx)?;

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: ThrushStatement = expression::build_expr(parser_ctx)?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    parser_ctx
        .get_mut_symbols()
        .new_lli(name, (instr_type.clone(), span), span)?;

    let lli: ThrushStatement = ThrushStatement::LLI {
        name,
        kind: instr_type,
        value: value.into(),
        span,
    };

    Ok(lli)
}

fn build_local<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let local_tk: &Token = parser_ctx.consume(
        TokenType::Local,
        String::from("Syntax error"),
        String::from("Expected 'local' keyword."),
    )?;

    let span: Span = local_tk.get_span();

    if parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Locals variables should be contained at local scope."),
            None,
            span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    let is_mutable: bool = parser_ctx.match_token(TokenType::Mut)?;

    let local_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name."),
    )?;

    let name: &str = local_tk.get_lexeme();
    let span: Span = local_tk.get_span();

    parser_ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let local_type: ThrushType = typegen::build_type(parser_ctx)?;

    let attributes: ThrushAttributes =
        self::build_attributes(parser_ctx, &[TokenType::SemiColon, TokenType::Eq])?;

    if parser_ctx.match_token(TokenType::SemiColon)? {
        parser_ctx.get_mut_symbols().new_local(
            name,
            (local_type.clone(), is_mutable, true, span),
            span,
        )?;

        return Ok(ThrushStatement::Local {
            name,
            kind: local_type,
            value: ThrushStatement::Null { span }.into(),
            attributes,
            is_mutable,
            span,
        });
    }

    parser_ctx.get_mut_symbols().new_local(
        name,
        (local_type.clone(), is_mutable, false, span),
        span,
    )?;

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: ThrushStatement = expression::build_expr(parser_ctx)?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    let local: ThrushStatement = ThrushStatement::Local {
        name,
        kind: local_type,
        value: value.into(),
        attributes,
        is_mutable,
        span,
    };

    Ok(local)
}

fn build_return<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let return_tk: &Token = parser_ctx.consume(
        TokenType::Return,
        String::from("Syntax error"),
        String::from("Expected 'return' keyword."),
    )?;

    let span: Span = return_tk.get_span();

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Return outside of bind or function."),
            None,
            span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    if parser_ctx.match_token(TokenType::SemiColon)? {
        if parser_ctx.get_type_ctx().get_function_type().is_void_type() {
            return Ok(ThrushStatement::Null { span });
        }

        return Ok(ThrushStatement::Return {
            expression: None,
            kind: ThrushType::Void,
            span,
        });
    }

    let value: ThrushStatement = expression::build_expr(parser_ctx)?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(ThrushStatement::Return {
        expression: Some(value.into()),
        kind: parser_ctx.get_type_ctx().get_function_type().clone(),
        span,
    })
}

fn build_block<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let block_tk: &Token = parser_ctx.consume(
        TokenType::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let span: Span = block_tk.get_span();

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Block of code must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    *parser_ctx.get_mut_scope() += 1;
    parser_ctx.get_mut_symbols().begin_scope();

    let mut stmts: Vec<ThrushStatement> = Vec::with_capacity(100);

    while !parser_ctx.match_token(TokenType::RBrace)? {
        let stmt: ThrushStatement = self::statement(parser_ctx)?;
        stmts.push(stmt)
    }

    parser_ctx.get_mut_symbols().end_scope();
    *parser_ctx.get_mut_scope() -= 1;

    Ok(ThrushStatement::Block { stmts, span })
}

pub fn build_assembler_function<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare_forward: bool,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
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

    let mut parameters: Vec<ThrushStatement> = Vec::with_capacity(10);
    let mut parameters_types: Vec<ThrushType> = Vec::with_capacity(10);

    let mut parameter_position: u32 = 0;

    loop {
        if parser_ctx.check(TokenType::RParen) {
            break;
        }

        let parameter_name_tk: &'instr Token = parser_ctx.consume(
            TokenType::Identifier,
            String::from("Syntax error"),
            String::from("Expected 'identifier'."),
        )?;

        let parameter_name: &str = parameter_name_tk.get_lexeme();
        let parameter_span: Span = parameter_name_tk.get_span();

        let parameter_type: ThrushType = typegen::build_type(parser_ctx)?;

        parameters_types.push(parameter_type.clone());

        parameters.push(ThrushStatement::AssemblerFunctionParameter {
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

    let attributes: ThrushAttributes = self::build_attributes(parser_ctx, &[TokenType::LBrace])?;

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

        let raw_str: ThrushStatement = expression::build_expr(parser_ctx)?;
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

        let raw_str: ThrushStatement = expression::build_expr(parser_ctx)?;
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

        return Ok(ThrushStatement::Null { span });
    }

    Ok(ThrushStatement::AssemblerFunction {
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

pub fn build_function<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare_forward: bool,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenType::Fn,
        String::from("Syntax error"),
        String::from("Expected 'fn' keyword."),
    )?;

    let function_name_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name to the function."),
    )?;

    let function_name: &str = function_name_tk.get_lexeme();
    let function_ascii_name: &str = function_name_tk.get_ascii_lexeme();

    let span: Span = function_name_tk.get_span();

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Functions can only be defined globally."),
            None,
            span,
        ));
    }

    if function_name == "main" {
        if declare_forward {
            return Ok(ThrushStatement::Null { span });
        }

        parser_ctx.get_mut_control_ctx().set_inside_function(true);

        let entrypoint: Result<ThrushStatement, ThrushCompilerIssue> =
            self::build_entry_point(parser_ctx);

        parser_ctx.get_mut_control_ctx().set_inside_function(false);

        return entrypoint;
    }

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut parameters: Vec<ThrushStatement> = Vec::with_capacity(10);
    let mut parameters_types: Vec<ThrushType> = Vec::with_capacity(10);

    let mut parameter_position: u32 = 0;

    loop {
        if parser_ctx.check(TokenType::RParen) {
            break;
        }

        let is_mutable: bool = parser_ctx.match_token(TokenType::Mut)?;

        let parameter_tk: &Token = parser_ctx.consume(
            TokenType::Identifier,
            String::from("Syntax error"),
            String::from("Expected parameter name."),
        )?;

        let parameter_name: &str = parameter_tk.get_lexeme();
        let parameter_span: Span = parameter_tk.get_span();

        parser_ctx.consume(
            TokenType::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let parameter_type: ThrushType = typegen::build_type(parser_ctx)?;

        parameters_types.push(parameter_type.clone());

        parameters.push(ThrushStatement::FunctionParameter {
            name: parameter_name,
            kind: parameter_type,
            position: parameter_position,
            is_mutable,
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

    let function_attributes: ThrushAttributes =
        self::build_attributes(parser_ctx, &[TokenType::SemiColon, TokenType::LBrace])?;

    let function_has_ignore: bool = function_attributes.has_ignore_attribute();

    let mut function: ThrushStatement = ThrushStatement::Function {
        name: function_name,
        ascii_name: function_ascii_name,
        parameters: parameters.clone(),
        parameter_types: parameters_types.clone(),
        body: ThrushStatement::Null { span }.into(),
        return_type: return_type.clone(),
        attributes: function_attributes,
        span,
    };

    if declare_forward {
        if let Err(error) = parser_ctx.get_mut_symbols().new_function(
            function_name,
            (
                return_type,
                ParametersTypes::new(parameters_types),
                function_has_ignore,
            ),
            span,
        ) {
            parser_ctx.add_error(error);
        }

        if parser_ctx.match_token(TokenType::SemiColon)? {
            return Ok(function);
        }

        return Ok(ThrushStatement::Null { span });
    }

    if parser_ctx.match_token(TokenType::SemiColon)? {
        return Ok(function);
    }

    parser_ctx.get_mut_control_ctx().set_inside_function(true);

    parser_ctx
        .get_mut_type_ctx()
        .set_function_type(return_type.clone());

    parser_ctx.get_mut_symbols().start_parameters(&parameters)?;

    let function_body: ThrushStatement = self::build_block(parser_ctx)?;

    parser_ctx.get_mut_symbols().end_parameters();
    parser_ctx.get_mut_control_ctx().set_inside_function(false);

    if let ThrushStatement::Function { body, .. } = &mut function {
        *body = function_body.into();
    }

    Ok(function)
}

/* ######################################################################


    COMPILER ATTRIBUTES BUILDER


########################################################################*/

pub fn build_attributes<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    limits: &[TokenType],
) -> Result<ThrushAttributes<'instr>, ThrushCompilerIssue> {
    let mut compiler_attributes: ThrushAttributes = Vec::with_capacity(10);

    while !limits.contains(&parser_ctx.peek().kind) {
        let current_tk: &Token = parser_ctx.peek();
        let span: Span = current_tk.span;

        match current_tk.kind {
            TokenType::Extern => {
                compiler_attributes.push(LLVMAttribute::Extern(
                    self::build_external_attribute(parser_ctx)?,
                    span,
                ));
            }

            TokenType::Convention => {
                compiler_attributes.push(LLVMAttribute::Convention(
                    self::build_call_convention_attribute(parser_ctx)?,
                    span,
                ));
            }

            TokenType::Public => {
                compiler_attributes.push(self::LLVMAttribute::Public(span));
                parser_ctx.only_advance()?;
            }

            TokenType::AsmSyntax => compiler_attributes.push(LLVMAttribute::AsmSyntax(
                self::build_assembler_syntax_attribute(parser_ctx)?,
                span,
            )),

            attribute if attribute.as_compiler_attribute(span).is_some() => {
                if let Some(compiler_attribute) = attribute.as_compiler_attribute(span) {
                    compiler_attributes.push(compiler_attribute);
                    parser_ctx.only_advance()?;
                }
            }

            _ => break,
        }
    }

    Ok(compiler_attributes)
}

/* ######################################################################


    COMPILER SPECIAL ATTRIBUTES


########################################################################*/

fn build_external_attribute<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<&'instr str, ThrushCompilerIssue> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let name: &Token = parser_ctx.consume(
        TokenType::Str,
        String::from("Syntax error"),
        String::from("Expected a string literal for @extern(\"FFI NAME\")."),
    )?;

    let ffi_name: &str = name.get_lexeme();

    parser_ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ffi_name)
}

fn build_assembler_syntax_attribute<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<&'instr str, ThrushCompilerIssue> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let syntax_tk: &Token = parser_ctx.consume(
        TokenType::Str,
        String::from("Syntax error"),
        String::from("Expected a string literal for @asmsyntax(\"INTEL\")."),
    )?;

    let specified_syntax: &str = syntax_tk.get_lexeme();
    let syntax_span: Span = syntax_tk.get_span();

    let syntaxes: [&'static str; 2] = ["ATT", "INTEL"];

    if !syntaxes.contains(&specified_syntax) {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            format!(
                "Unknown assembler syntax, valid are '{}'.",
                syntaxes.join(", ")
            ),
            None,
            syntax_span,
        ));
    }

    parser_ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(specified_syntax)
}

fn build_call_convention_attribute(
    parser_ctx: &mut ParserContext<'_>,
) -> Result<CallConvention, ThrushCompilerIssue> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let convention_tk: &Token = parser_ctx.consume(
        TokenType::Str,
        String::from("Syntax error"),
        String::from("Expected a literal 'str' for @convention(\"CONVENTION NAME\")."),
    )?;

    let span: Span = convention_tk.span;
    let name: &[u8] = convention_tk.lexeme.as_bytes();

    if let Some(call_convention) = CALL_CONVENTIONS.get(name) {
        parser_ctx.consume(
            TokenType::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        return Ok(*call_convention);
    }

    parser_ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Err(ThrushCompilerIssue::Error(
        String::from("Syntax error"),
        String::from("Unknown call convention."),
        None,
        span,
    ))
}
