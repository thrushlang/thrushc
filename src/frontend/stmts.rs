use std::rc::Rc;

use ahash::AHashMap as HashMap;

use crate::{
    backend::llvm::compiler::{attributes::LLVMAttribute, conventions::CallConvention},
    lazy_static,
    middle::types::frontend::{
        lexer::{
            tokenkind::TokenKind,
            traits::ThrushStructTypeExtensions,
            types::{BindingsApplicant, ThrushStructType, ThrushType, generate_bindings},
        },
        parser::{
            stmts::{
                instruction::Instruction,
                traits::CompilerAttributesExtensions,
                types::{CompilerAttributes, CustomTypeFields, EnumFields, StructFields},
            },
            symbols::types::{Bindings, ParametersTypes},
        },
    },
    standard::error::ThrushCompilerIssue,
};

use super::{
    contexts::{BindingsType, InstructionPosition, SyncPosition, TypePosition},
    expressions,
    lexer::{Span, Token},
    parser::ParserContext,
    typegen,
};

const MINIMAL_SCOPE_CAPACITY: usize = 256;
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
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Declaration);

    let statement: Result<Instruction<'instr>, ThrushCompilerIssue> = match &parser_ctx.peek().kind
    {
        TokenKind::Type => Ok(build_custom_type(parser_ctx, false)?),
        TokenKind::Struct => Ok(build_struct(parser_ctx, false)?),
        TokenKind::Enum => Ok(build_enum(parser_ctx, false)?),
        TokenKind::Fn => Ok(build_function(parser_ctx, false)?),
        TokenKind::Const => Ok(build_const(parser_ctx, false)?),
        TokenKind::Bindings => Ok(build_bindings(parser_ctx, false)?),

        _ => Ok(statement(parser_ctx)?),
    };

    statement
}

fn statement<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Statement);

    let statement: Result<Instruction<'instr>, ThrushCompilerIssue> = match &parser_ctx.peek().kind
    {
        TokenKind::LBrace => Ok(build_block(parser_ctx)?),
        TokenKind::Return => Ok(build_return(parser_ctx)?),
        TokenKind::Local => Ok(build_local(parser_ctx, false)?),
        TokenKind::For => Ok(build_for_loop(parser_ctx)?),
        TokenKind::If => Ok(build_conditional(parser_ctx)?),
        TokenKind::Match => Ok(build_match(parser_ctx)?),
        TokenKind::While => Ok(build_while_loop(parser_ctx)?),
        TokenKind::Continue => Ok(build_continue(parser_ctx)?),
        TokenKind::Break => Ok(build_break(parser_ctx)?),
        TokenKind::Loop => Ok(build_loop(parser_ctx)?),

        _ => Ok(expressions::build_expression(parser_ctx)?),
    };

    statement
}

pub fn build_bindings<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_instr_position(InstructionPosition::Bindings);

    let bindings_tk: &Token = parser_ctx.consume(
        TokenKind::Bindings,
        String::from("Syntax error"),
        String::from("Expected 'bindings' keyword."),
    )?;

    let span: Span = bindings_tk.span;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Bindings are only defined globally."),
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
            bindings_tk.span,
        ));
    }

    parser_ctx
        .get_mut_type_ctx()
        .set_this_bindings_type(BindingsType::Struct(kind.clone()));

    let struct_type: ThrushStructType = kind.into_structure_type();
    let struct_name: String = struct_type.get_name();

    parser_ctx
        .get_symbols()
        .contains_structure(&struct_name, span)?;

    let mut binds: Vec<Instruction> = Vec::with_capacity(20);

    parser_ctx.consume(
        TokenKind::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    while parser_ctx.peek().kind != TokenKind::RBrace {
        let bind: Instruction = build_bind(declare, parser_ctx)?;

        parser_ctx
            .get_mut_control_ctx()
            .set_instr_position(InstructionPosition::Bindings);

        binds.push(bind);
    }

    parser_ctx.consume(
        TokenKind::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    parser_ctx
        .get_mut_type_ctx()
        .set_this_bindings_type(BindingsType::NoRelevant);

    parser_ctx
        .get_mut_control_ctx()
        .set_instr_position(InstructionPosition::NoRelevant);

    if declare {
        let bindings_generated: Bindings = generate_bindings(binds.clone());

        parser_ctx.get_mut_symbols().set_bindings(
            &struct_name,
            bindings_generated,
            BindingsApplicant::Struct,
            span,
        )?;

        return Ok(Instruction::Null);
    }

    Ok(Instruction::Bindings {
        name: struct_name,
        binds,
    })
}

fn build_bind<'instr>(
    declare: bool,
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenKind::Bind,
        String::from("Syntax error"),
        String::from("Expected 'bind' keyword."),
    )?;

    let bind_name_tk: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected name to the bind."),
    )?;

    let bind_name: &str = bind_name_tk.lexeme;

    if !parser_ctx
        .get_control_ctx()
        .get_instr_position()
        .is_bindings()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected bind inside the bindings definition."),
            None,
            bind_name_tk.span,
        ));
    }

    parser_ctx
        .get_mut_control_ctx()
        .set_instr_position(InstructionPosition::Bind);

    parser_ctx.consume(
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut bind_parameters: Vec<Instruction> = Vec::with_capacity(10);
    let mut bind_position: u32 = 0;

    let mut this_is_declared: bool = false;

    while !parser_ctx.match_token(TokenKind::RParen)? {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        parser_ctx
            .get_mut_type_ctx()
            .set_position(TypePosition::BindParameter);

        if this_is_declared && parser_ctx.check(TokenKind::This) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from(
                    "'This' keyword is already declared. Multiple instances are not allowed.",
                ),
                None,
                bind_name_tk.span,
            ));
        }

        if parser_ctx.check(TokenKind::This) {
            let this_tk: &Token = parser_ctx.consume(
                TokenKind::This,
                String::from("Syntax error"),
                String::from("Expected 'this' keyword."),
            )?;

            let is_mutable: bool = parser_ctx.match_token(TokenKind::Mut)?;

            bind_parameters.push(Instruction::This {
                kind: parser_ctx
                    .get_type_ctx()
                    .get_this_bindings_type()
                    .dissamble(),
                is_mutable,
                span: this_tk.span,
            });

            this_is_declared = true;

            continue;
        }

        let is_mutable: bool = parser_ctx.match_token(TokenKind::Mut)?;

        let parameter_tk: &Token = parser_ctx.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected parameter name."),
        )?;

        let parameter_name: &str = parameter_tk.lexeme;
        let parameter_span: Span = parameter_tk.span;

        parser_ctx.consume(
            TokenKind::ColonColon,
            String::from("Syntax error"),
            String::from("Expected '::'."),
        )?;

        let parameter_type: ThrushType = typegen::build_type(parser_ctx)?;

        if !declare {
            parser_ctx.get_mut_symbols().new_parameter(
                parameter_name,
                (parameter_type.clone(), false, is_mutable, parameter_span),
                parameter_span,
            )?;
        }

        parser_ctx
            .get_mut_type_ctx()
            .set_position(TypePosition::NoRelevant);

        bind_parameters.push(Instruction::BindParameter {
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

    let bind_attributes: CompilerAttributes =
        self::build_compiler_attributes(parser_ctx, &[TokenKind::LBrace])?;

    if !declare {
        parser_ctx.get_mut_control_ctx().set_inside_bind(true);

        parser_ctx
            .get_mut_type_ctx()
            .set_bind_instance(this_is_declared);

        let bind_body: Instruction = build_block(parser_ctx)?;

        parser_ctx.get_mut_symbols().end_parameters();
        parser_ctx.get_mut_control_ctx().set_inside_bind(false);
        parser_ctx.get_mut_type_ctx().set_bind_instance(false);

        parser_ctx
            .get_mut_control_ctx()
            .set_instr_position(InstructionPosition::NoRelevant);

        return Ok(Instruction::Bind {
            name: bind_name,
            parameters: bind_parameters,
            body: bind_body.into(),
            return_type,
            attributes: bind_attributes,
        });
    }

    parser_ctx
        .get_mut_control_ctx()
        .set_instr_position(InstructionPosition::NoRelevant);

    Ok(Instruction::Null)
}

fn build_entry_point<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let span: Span = parser_ctx.previous().span;

    if parser_ctx.get_control_ctx().get_entrypoint() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Duplicated entrypoint"),
            String::from("The language not support two entrypoints."),
            None,
            parser_ctx.previous().span,
        ));
    }

    parser_ctx.consume(
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    parser_ctx.consume(
        TokenKind::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    parser_ctx.get_mut_control_ctx().set_entrypoint(true);

    Ok(Instruction::EntryPoint {
        body: build_block(parser_ctx)?.into(),
        span,
    })
}

fn build_for_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let for_tk: &Token = parser_ctx.consume(
        TokenKind::For,
        String::from("Syntax error"),
        String::from("Expected 'for' keyword."),
    )?;

    let for_span: Span = for_tk.span;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            for_span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("For loop must be placed inside a function or a bind."),
            None,
            for_span,
        ));
    }

    let local: Instruction = build_local(parser_ctx, false)?;
    let cond: Instruction = expressions::build_expression(parser_ctx)?;

    parser_ctx.mismatch_types(
        &ThrushType::Bool,
        cond.get_type()?,
        cond.get_span()?,
        Some(&cond),
    );

    let actions: Instruction = expressions::build_expression(parser_ctx)?;

    let mut local_clone: Instruction = local.clone();

    if let Instruction::Local { comptime, .. } = &mut local_clone {
        *comptime = true;
    }

    //parser_ctx.add_lift_local(local_clone);

    parser_ctx.get_mut_control_ctx().set_inside_loop(true);

    let body: Instruction = build_block(parser_ctx)?;

    parser_ctx.get_mut_control_ctx().set_inside_loop(false);

    Ok(Instruction::For {
        local: local.into(),
        cond: cond.into(),
        actions: actions.into(),
        block: body.into(),
        span: for_span,
    })
}

fn build_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let loop_tk: &Token = parser_ctx.consume(
        TokenKind::Loop,
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

    let block: Instruction = build_block(parser_ctx)?;

    let scope: usize = parser_ctx.get_scope();

    if !block.has_break() && !block.has_return() && !block.has_continue() {
        parser_ctx
            .get_mut_control_ctx()
            .set_unreacheable_code_scope(scope);
    }

    parser_ctx.get_mut_control_ctx().set_inside_loop(false);

    Ok(Instruction::Loop {
        block: block.into(),
        span: loop_span,
    })
}

fn build_while_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let while_tk: &Token = parser_ctx.consume(
        TokenKind::While,
        String::from("Syntax error"),
        String::from("Expected 'while' keyword."),
    )?;

    let while_span: Span = while_tk.span;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            while_span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("While loop must be placed inside a function or a bind."),
            None,
            while_span,
        ));
    }

    let conditional: Instruction = expressions::build_expr(parser_ctx)?;

    parser_ctx.mismatch_types(
        &ThrushType::Bool,
        conditional.get_type()?,
        conditional.get_span()?,
        Some(&conditional),
    );

    let block: Instruction = build_block(parser_ctx)?;

    Ok(Instruction::While {
        cond: conditional.into(),
        block: block.into(),
        span: while_span,
    })
}

fn build_continue<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let continue_tk: &Token = parser_ctx.consume(
        TokenKind::Continue,
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
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Instruction::Continue { span })
}

fn build_break<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let break_tk: &Token = parser_ctx.consume(
        TokenKind::Break,
        String::from("Syntax error"),
        String::from("Expected 'break' keyword."),
    )?;

    let span: Span = break_tk.span;

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
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Instruction::Break { span })
}

fn build_match<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let match_tk: &Token = parser_ctx.consume(
        TokenKind::Match,
        String::from("Syntax error"),
        String::from("Expected 'match' keyword."),
    )?;

    let span: Span = match_tk.span;

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
            String::from("Match must be placed inside a function or a bind."),
            None,
            span,
        ));
    }

    let mut start_pattern: Instruction = expressions::build_expr(parser_ctx)?;
    let mut start_block: Instruction = Instruction::Block {
        stmts: Vec::new(),
        span,
    };

    let mut patterns: Vec<Instruction> = Vec::with_capacity(10);
    let mut patterns_stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

    let mut position: u32 = 0;

    while parser_ctx.match_token(TokenKind::Pattern)? {
        *parser_ctx.get_mut_scope() += 1;

        parser_ctx.get_mut_symbols().begin_local_scope();

        let pattern: Instruction = expressions::build_expr(parser_ctx)?;
        let pattern_span: Span = pattern.get_span()?;

        parser_ctx.mismatch_types(
            &ThrushType::Bool,
            pattern.get_type()?,
            pattern.get_span()?,
            Some(&pattern),
        );

        parser_ctx.consume(
            TokenKind::ColonColon,
            String::from("Syntax error"),
            String::from("Expected '::'."),
        )?;

        while !parser_ctx.match_token(TokenKind::Break)? {
            patterns_stmts.push(statement(parser_ctx)?);
        }

        parser_ctx.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        *parser_ctx.get_mut_scope() -= 1;

        parser_ctx.get_mut_symbols().end_local_scope();

        if patterns_stmts.is_empty() {
            continue;
        }

        if position != 0 {
            patterns.push(Instruction::Elif {
                cond: Rc::new(pattern),
                block: Rc::new(Instruction::Block {
                    stmts: patterns_stmts.clone(),
                    span: pattern_span,
                }),
                span: pattern_span,
            });

            patterns_stmts.clear();
            position += 1;

            continue;
        }

        start_pattern = pattern;

        start_block = Instruction::Block {
            stmts: patterns_stmts.clone(),
            span: pattern_span,
        };

        patterns_stmts.clear();
        position += 1;
    }

    if start_block.has_instruction() {
        parser_ctx.mismatch_types(
            &ThrushType::Bool,
            start_pattern.get_type()?,
            start_pattern.get_span()?,
            Some(&start_pattern),
        );
    }

    let otherwise: Option<Rc<Instruction>> = if parser_ctx.match_token(TokenKind::Else)? {
        let otherwise_span: Span = parser_ctx.previous().span;

        parser_ctx.consume(
            TokenKind::ColonColon,
            String::from("Syntax error"),
            String::from("Expected '::'."),
        )?;

        let mut stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

        while !parser_ctx.match_token(TokenKind::Break)? {
            stmts.push(statement(parser_ctx)?);
        }

        parser_ctx.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        if stmts.is_empty() {
            None
        } else {
            Some(
                Instruction::Else {
                    block: Instruction::Block {
                        stmts,
                        span: otherwise_span,
                    }
                    .into(),
                    span: otherwise_span,
                }
                .into(),
            )
        }
    } else {
        None
    };

    if !start_block.has_instruction() && patterns.is_empty() && otherwise.is_none() {
        return Ok(Instruction::Null);
    }

    Ok(Instruction::If {
        cond: Rc::new(start_pattern),
        block: Rc::new(start_block),
        elfs: patterns,
        otherwise,
        span,
    })
}

fn build_conditional<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let if_tk: &Token = parser_ctx.consume(
        TokenKind::If,
        String::from("Syntax error"),
        String::from("Expected 'if' keyword."),
    )?;

    let span: Span = if_tk.span;

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

    let if_condition: Instruction = expressions::build_expr(parser_ctx)?;

    parser_ctx.mismatch_types(
        &ThrushType::Bool,
        if_condition.get_type()?,
        if_condition.get_span()?,
        Some(&if_condition),
    );

    let if_body: Rc<Instruction> = Rc::new(build_block(parser_ctx)?);

    let mut elfs: Vec<Instruction> = Vec::with_capacity(10);

    while parser_ctx.match_token(TokenKind::Elif)? {
        let span: Span = parser_ctx.previous().span;

        let elif_condition: Instruction = expressions::build_expr(parser_ctx)?;

        parser_ctx.mismatch_types(
            &ThrushType::Bool,
            elif_condition.get_type()?,
            elif_condition.get_span()?,
            Some(&elif_condition),
        );

        let elif_body: Instruction = build_block(parser_ctx)?;

        if !elif_body.has_instruction() {
            continue;
        }

        elfs.push(Instruction::Elif {
            cond: Rc::new(elif_condition),
            block: Rc::new(elif_body),
            span,
        });
    }

    let mut otherwise: Option<Rc<Instruction>> = None;

    if parser_ctx.match_token(TokenKind::Else)? {
        let span: Span = parser_ctx.previous().span;
        let else_body: Instruction = build_block(parser_ctx)?;

        if else_body.has_instruction() {
            otherwise = Some(
                Instruction::Else {
                    block: else_body.into(),
                    span,
                }
                .into(),
            );
        }
    }

    Ok(Instruction::If {
        cond: Rc::new(if_condition),
        block: if_body,
        elfs,
        otherwise,
        span,
    })
}

pub fn build_custom_type<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let type_tk: &Token = parser_ctx.consume(
        TokenKind::Type,
        String::from("Syntax error"),
        String::from("Expected 'type' keyword."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Types are only defined globally."),
            None,
            type_tk.span,
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected type name."),
    )?;

    let span: Span = name.span;
    let custom_type_name: &str = name.lexeme;

    parser_ctx.consume(
        TokenKind::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let custom_type_attributes: CompilerAttributes =
        build_compiler_attributes(parser_ctx, &[TokenKind::LBrace])?;

    parser_ctx.consume(
        TokenKind::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut custom_type_fields: CustomTypeFields = Vec::with_capacity(10);

    while parser_ctx.peek().kind != TokenKind::RBrace {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        let kind: ThrushType = typegen::build_type(parser_ctx)?;

        custom_type_fields.push(kind);
    }

    parser_ctx.consume(
        TokenKind::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    if declare {
        parser_ctx.get_mut_symbols().new_custom_type(
            custom_type_name,
            (custom_type_fields, custom_type_attributes),
            span,
        )?;
    }

    Ok(Instruction::Null)
}

pub fn build_enum<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let enum_tk: &Token = parser_ctx.consume(
        TokenKind::Enum,
        String::from("Syntax error"),
        String::from("Expected 'enum'."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Enums are only defined globally."),
            None,
            enum_tk.span,
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected enum name."),
    )?;

    let span: Span = name.span;

    let enum_name: &str = name.lexeme;

    let enum_attributes: CompilerAttributes =
        self::build_compiler_attributes(parser_ctx, &[TokenKind::LBrace])?;

    parser_ctx.consume(
        TokenKind::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut enum_fields: EnumFields = Vec::with_capacity(10);
    let mut index: f64 = 0.0;

    while parser_ctx.peek().kind != TokenKind::RBrace {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        if parser_ctx.match_token(TokenKind::Identifier)? {
            let field_tk: &Token = parser_ctx.previous();
            let name: &str = field_tk.lexeme;
            let span: Span = field_tk.span;

            parser_ctx.consume(
                TokenKind::Colon,
                String::from("Syntax error"),
                String::from("Expected ':'."),
            )?;

            let field_type: ThrushType = typegen::build_type(parser_ctx)?;

            if !field_type.is_integer_type()
                && !field_type.is_float_type()
                && !field_type.is_bool_type()
            {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Expected integer, boolean or floating-point types."),
                    None,
                    span,
                ));
            }

            if parser_ctx.match_token(TokenKind::SemiColon)? {
                let field_value: Instruction = if field_type.is_float_type() {
                    Instruction::Float(field_type.clone(), index, false, span)
                } else if field_type.is_bool_type() {
                    Instruction::Boolean(ThrushType::Bool, index != 0.0, span)
                } else {
                    Instruction::Integer(field_type.clone(), index, false, span)
                };

                enum_fields.push((name, field_value));
                index += 1.0;

                continue;
            }

            parser_ctx.consume(
                TokenKind::Eq,
                String::from("Syntax error"),
                String::from("Expected '='."),
            )?;

            let expression: Instruction = expressions::build_expr(parser_ctx)?;

            expression.throw_attemping_use_jit(expression.get_span()?)?;

            parser_ctx.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            parser_ctx.mismatch_types(
                &field_type,
                expression.get_type()?,
                expression.get_span()?,
                Some(&expression),
            );

            enum_fields.push((name, expression));

            continue;
        }

        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected identifier in enum field."),
            None,
            parser_ctx.advance()?.span,
        ));
    }

    parser_ctx.consume(
        TokenKind::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    if declare {
        parser_ctx
            .get_mut_symbols()
            .new_enum(enum_name, (enum_fields, enum_attributes), span)?;
    }

    Ok(Instruction::Null)
}

pub fn build_struct<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let struct_tk: &Token = parser_ctx.consume(
        TokenKind::Struct,
        String::from("Syntax error"),
        String::from("Expected 'struct' keyword."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Structs are only defined globally."),
            None,
            struct_tk.span,
        ));
    }

    let name: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected structure name."),
    )?;

    let span: Span = name.span;

    let struct_name: &str = name.lexeme;

    let struct_attributes: CompilerAttributes =
        self::build_compiler_attributes(parser_ctx, &[TokenKind::LBrace])?;

    parser_ctx.consume(
        TokenKind::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let mut fields_types: StructFields = (struct_name, Vec::with_capacity(10));
    let mut field_position: u32 = 0;

    while parser_ctx.peek().kind != TokenKind::RBrace {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        let field_tk: &Token<'_> = parser_ctx.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected identifier."),
        )?;

        let field_name: &str = field_tk.lexeme;

        parser_ctx
            .get_mut_type_ctx()
            .set_position(TypePosition::StructureField);

        parser_ctx.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let field_type: ThrushType = typegen::build_type(parser_ctx)?;

        fields_types
            .1
            .push((field_name, field_type, field_position));

        field_position += 1;

        parser_ctx
            .get_mut_type_ctx()
            .set_position(TypePosition::NoRelevant);
    }

    parser_ctx.consume(
        TokenKind::RBrace,
        String::from("Syntax error"),
        String::from("Expected '}'."),
    )?;

    if declare {
        parser_ctx.get_mut_symbols().new_struct(
            struct_name,
            (
                struct_name,
                fields_types.1,
                struct_attributes,
                Vec::with_capacity(100),
            ),
            span,
        )?;
    }

    Ok(Instruction::Null)
}

pub fn build_const<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenKind::Const,
        String::from("Syntax error"),
        String::from("Expected 'const' keyword."),
    )?;

    let const_tk: &Token = parser_ctx.consume(
        TokenKind::Identifier,
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

    let name: &str = const_tk.lexeme;
    let span: Span = const_tk.span;

    parser_ctx.consume(
        TokenKind::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let const_type: ThrushType = typegen::build_type(parser_ctx)?;

    let const_attributes: CompilerAttributes =
        self::build_compiler_attributes(parser_ctx, &[TokenKind::Eq])?;

    parser_ctx.consume(
        TokenKind::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Instruction = expressions::build_expr(parser_ctx)?;

    value.throw_attemping_use_jit(span)?;

    parser_ctx.mismatch_types(
        &const_type,
        value.get_type()?,
        value.get_span()?,
        Some(&value),
    );

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    if declare {
        parser_ctx
            .get_mut_symbols()
            .new_constant(name, (const_type, const_attributes), span)?;

        return Ok(Instruction::Null);
    }

    Ok(Instruction::Const {
        name,
        kind: const_type,
        value: value.into(),
        attributes: const_attributes,
        span,
    })
}

fn build_local<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    comptime: bool,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_type_ctx()
        .set_position(TypePosition::Local);

    let local_tk: &Token = parser_ctx.consume(
        TokenKind::Local,
        String::from("Syntax error"),
        String::from("Expected 'local' keyword."),
    )?;

    if parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Locals variables should be contained at local scope."),
            None,
            local_tk.span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            local_tk.span,
        ));
    }

    let is_mutable: bool = parser_ctx.match_token(TokenKind::Mut)?;

    let local_tk: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected name."),
    )?;

    let local_name: &str = local_tk.lexeme;
    let span: Span = local_tk.span;

    parser_ctx.consume(
        TokenKind::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let local_type: ThrushType = typegen::build_type(parser_ctx)?;

    let scope: usize = parser_ctx.get_scope();

    if parser_ctx.match_token(TokenKind::SemiColon)? {
        parser_ctx.get_mut_symbols().new_local(
            scope,
            local_name,
            (local_type.clone(), is_mutable, true, span),
            span,
        )?;

        parser_ctx
            .get_mut_type_ctx()
            .set_position(TypePosition::NoRelevant);

        return Ok(Instruction::Local {
            name: local_name,
            kind: local_type,
            value: Rc::new(Instruction::Null),
            is_mutable,
            span,
            comptime,
        });
    }

    parser_ctx.get_mut_symbols().new_local(
        scope,
        local_name,
        (local_type.clone(), is_mutable, false, span),
        span,
    )?;

    parser_ctx.consume(
        TokenKind::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Instruction = expressions::build_expr(parser_ctx)?;

    parser_ctx.mismatch_types(
        &local_type,
        value.get_type()?,
        value.get_span()?,
        Some(&value),
    );

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    parser_ctx
        .get_mut_type_ctx()
        .set_position(TypePosition::NoRelevant);

    let local: Instruction = Instruction::Local {
        name: local_name,
        kind: local_type,
        value: value.into(),
        is_mutable,
        span,
        comptime,
    };

    Ok(local)
}

fn build_return<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let return_tk: &Token = parser_ctx.consume(
        TokenKind::Return,
        String::from("Syntax error"),
        String::from("Expected 'return' keyword."),
    )?;

    let span: Span = return_tk.span;

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

    let function_type: ThrushType = parser_ctx.get_type_ctx().get_function_type();

    if parser_ctx.match_token(TokenKind::SemiColon)? {
        if parser_ctx.get_type_ctx().get_function_type().is_void_type() {
            return Ok(Instruction::Null);
        }

        parser_ctx.mismatch_types(
            &ThrushType::Void,
            &function_type,
            parser_ctx.previous().span,
            None,
        );

        return Ok(Instruction::Return {
            expression: None,
            kind: ThrushType::Void,
            span,
        });
    }

    let value: Instruction = expressions::build_expr(parser_ctx)?;

    parser_ctx.mismatch_types(
        &function_type,
        value.get_type()?,
        value.get_span()?,
        Some(&value),
    );

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Instruction::Return {
        expression: Some(value.into()),
        kind: parser_ctx.get_type_ctx().get_function_type().clone(),
        span,
    })
}

fn build_block<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    let block_tk: &Token = parser_ctx.consume(
        TokenKind::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    let span: Span = block_tk.span;

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
    parser_ctx.get_mut_symbols().begin_local_scope();

    let mut stmts: Vec<Instruction> = Vec::with_capacity(100);

    while !parser_ctx.match_token(TokenKind::RBrace)? {
        let instruction: Instruction = statement(parser_ctx)?;
        stmts.push(instruction)
    }

    parser_ctx.get_mut_symbols().end_local_scope();
    *parser_ctx.get_mut_scope() -= 1;

    Ok(Instruction::Block { stmts, span })
}

pub fn build_function<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenKind::Fn,
        String::from("Syntax error"),
        String::from("Expected 'fn' keyword."),
    )?;

    let function_name_tk: &Token = parser_ctx.consume(
        TokenKind::Identifier,
        String::from("Syntax error"),
        String::from("Expected name to the function."),
    )?;

    let function_name: &str = function_name_tk.lexeme;
    let function_span: Span = function_name_tk.span;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Functions are only defined globally."),
            None,
            function_span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            function_span,
        ));
    }

    if function_name == "main" {
        if declare {
            return Ok(Instruction::Null);
        }

        parser_ctx.get_mut_control_ctx().set_inside_function(true);

        let entrypoint: Result<Instruction, ThrushCompilerIssue> = build_entry_point(parser_ctx);

        parser_ctx.get_mut_control_ctx().set_inside_function(false);

        return entrypoint;
    }

    let mut parameters: Vec<Instruction> = Vec::with_capacity(10);
    let mut parameters_types: Vec<ThrushType> = Vec::with_capacity(10);

    let mut position: u32 = 0;

    parser_ctx.consume(
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    while parser_ctx.peek().kind != TokenKind::RParen {
        if parser_ctx.match_token(TokenKind::Comma)? {
            continue;
        }

        parser_ctx
            .get_mut_type_ctx()
            .set_position(TypePosition::Parameter);

        let is_mutable: bool = parser_ctx.match_token(TokenKind::Mut)?;

        let parameter_tk: &Token = parser_ctx.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected parameter name."),
        )?;

        let parameter_name: &str = parameter_tk.lexeme;
        let parameter_span: Span = parameter_tk.span;

        parser_ctx.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let parameter_type: ThrushType = typegen::build_type(parser_ctx)?;

        if parameter_type.is_void_type() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Void type are not allowed as type parameter."),
                None,
                parameter_span,
            ));
        }

        parser_ctx
            .get_mut_type_ctx()
            .set_position(TypePosition::NoRelevant);

        parameters_types.push(parameter_type.clone());

        if !declare {
            parser_ctx.get_mut_symbols().new_parameter(
                parameter_name,
                (parameter_type.clone(), false, is_mutable, parameter_span),
                parameter_span,
            )?;
        }

        parameters.push(Instruction::FunctionParameter {
            name: parameter_name,
            kind: parameter_type,
            position,
            is_mutable,
            span: parameter_span,
        });

        position += 1;
    }

    parser_ctx.consume(
        TokenKind::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    let return_type: ThrushType = typegen::build_type(parser_ctx)?;

    parser_ctx
        .get_mut_type_ctx()
        .set_function_type(return_type.clone());

    let function_attributes: CompilerAttributes =
        build_compiler_attributes(parser_ctx, &[TokenKind::SemiColon, TokenKind::LBrace])?;

    let function_has_ffi: bool = function_attributes.has_ffi_attribute();
    let function_has_ignore: bool = function_attributes.has_ignore_attribute();

    if function_has_ignore && !function_has_ffi {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from(
                "The '@ignore' attribute can only be used if the function contains the '@extern' attribute.",
            ),
            None,
            function_span,
        ));
    }

    let mut function: Instruction = Instruction::Function {
        name: function_name,
        parameters: parameters.clone(),
        parameter_types: parameters_types.clone(),
        body: Instruction::Null.into(),
        return_type: return_type.clone(),
        attributes: function_attributes,
        span: function_span,
    };

    if function_has_ffi || declare {
        if declare {
            parser_ctx.get_mut_symbols().new_function(
                function_name,
                (
                    return_type,
                    ParametersTypes::new(parameters_types),
                    function_has_ignore,
                ),
                function_span,
            )?;
        }

        parser_ctx.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        parser_ctx.get_mut_control_ctx().set_inside_function(false);

        return Ok(function);
    }

    parser_ctx.get_mut_control_ctx().set_inside_function(true);

    let function_body: Rc<Instruction> = build_block(parser_ctx)?.into();

    parser_ctx.get_mut_symbols().end_parameters();
    parser_ctx.get_mut_control_ctx().set_inside_function(false);

    if !return_type.is_void_type() && !function_body.has_return() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Missing return with type '{}'.", return_type),
            None,
            function_span,
        ));
    }

    if let Instruction::Function { body, .. } = &mut function {
        *body = function_body;
    }

    Ok(function)
}

/* ######################################################################


    COMPILER ATTRIBUTES BUILDER


########################################################################*/

fn build_compiler_attributes<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    limits: &[TokenKind],
) -> Result<CompilerAttributes<'instr>, ThrushCompilerIssue> {
    let mut compiler_attributes: CompilerAttributes = Vec::with_capacity(10);

    while !limits.contains(&parser_ctx.peek().kind) {
        match parser_ctx.peek().kind {
            TokenKind::Extern => {
                compiler_attributes.push(LLVMAttribute::FFI(build_external_attribute(parser_ctx)?));
            }
            TokenKind::Convention => {
                compiler_attributes.push(LLVMAttribute::Convention(
                    build_call_convention_attribute(parser_ctx)?,
                ));
            }
            TokenKind::Public => {
                compiler_attributes.push(LLVMAttribute::Public);
                parser_ctx.only_advance()?;
            }

            attribute if attribute.as_compiler_attribute().is_some() => {
                if let Some(compiler_attribute) = attribute.as_compiler_attribute() {
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
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let name: &Token = parser_ctx.consume(
        TokenKind::Str,
        String::from("Syntax error"),
        String::from("Expected a literal 'str' for @extern(\"FFI NAME\")."),
    )?;

    let ffi_name: &str = name.lexeme;

    parser_ctx.consume(
        TokenKind::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ffi_name)
}

fn build_call_convention_attribute(
    parser_ctx: &mut ParserContext<'_>,
) -> Result<CallConvention, ThrushCompilerIssue> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let convention_tk: &Token = parser_ctx.consume(
        TokenKind::Str,
        String::from("Syntax error"),
        String::from("Expected a literal 'str' for @convention(\"CONVENTION NAME\")."),
    )?;

    let span: Span = convention_tk.span;
    let name: &[u8] = convention_tk.lexeme.as_bytes();

    if let Some(call_convention) = CALL_CONVENTIONS.get(name) {
        parser_ctx.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        return Ok(*call_convention);
    }

    parser_ctx.consume(
        TokenKind::RParen,
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
