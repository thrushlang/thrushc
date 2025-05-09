use std::rc::Rc;

use ahash::AHashMap as HashMap;

use crate::{
    backend::llvm::compiler::{attributes::LLVMAttribute, conventions::CallConvention},
    common::error::ThrushCompilerError,
    lazy_static,
    middle::{
        instruction::Instruction,
        statement::{
            CustomTypeFields, EnumFields, StructFields, ThrushAttributes,
            traits::AttributesExtensions,
        },
        symbols::types::{Bindings, Parameters},
        traits::ThrushStructTypeExtensions,
        types::{BindingsApplicant, ThrushStructType, TokenKind, Type, generate_bindings},
    },
};

use super::{
    contexts::{BindingsType, InstructionPosition, SyncPosition, TypePosition},
    expression,
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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Declaration);

    let statement: Result<Instruction<'instr>, ThrushCompilerError> = match &parser_ctx.peek().kind
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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Statement);

    let statement: Result<Instruction<'instr>, ThrushCompilerError> = match &parser_ctx.peek().kind
    {
        TokenKind::LBrace => Ok(build_block(parser_ctx)?),
        TokenKind::Return => Ok(build_return(parser_ctx)?),
        TokenKind::Local => Ok(build_local(parser_ctx, false)?),
        TokenKind::For => Ok(build_for_loop(parser_ctx)?),
        TokenKind::If => Ok(build_if_elif_else(parser_ctx)?),
        TokenKind::Match => Ok(build_match(parser_ctx)?),
        TokenKind::While => Ok(build_while_loop(parser_ctx)?),
        TokenKind::Continue => Ok(build_continue(parser_ctx)?),
        TokenKind::Break => Ok(build_break(parser_ctx)?),
        TokenKind::Loop => Ok(build_loop(parser_ctx)?),

        _ => Ok(expression::build_expression(parser_ctx)?),
    };

    statement
}

pub fn build_bindings<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
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
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Bindings are only defined globally."),
            String::default(),
            span,
        ));
    }

    let kind: Type = typegen::build_type(parser_ctx, None)?;

    if !kind.is_struct_type() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Expected struct type."),
            String::default(),
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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
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
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Expected bind inside the bindings definition."),
            String::default(),
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
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from(
                    "'This' keyword is already declared. Multiple instances are not allowed.",
                ),
                String::default(),
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

        let parameter_type: Type = typegen::build_type(parser_ctx, None)?;

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

    let return_type: Type = typegen::build_type(parser_ctx, None)?;

    parser_ctx
        .get_mut_type_ctx()
        .set_function_type(return_type.clone());

    let bind_attributes: ThrushAttributes =
        build_compiler_attributes(parser_ctx, &[TokenKind::LBrace])?;

    if !declare {
        bind_parameters.iter().cloned().for_each(|bind_parameter| {
            parser_ctx.add_lift_local(bind_parameter);
        });

        parser_ctx.get_mut_control_ctx().set_inside_bind(true);

        parser_ctx
            .get_mut_type_ctx()
            .set_bind_instance(this_is_declared);

        let bind_body: Instruction = build_block(parser_ctx)?;

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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    if parser_ctx.get_control_ctx().get_entrypoint() {
        return Err(ThrushCompilerError::Error(
            String::from("Duplicated entrypoint"),
            String::from("The language not support two entrypoints."),
            String::default(),
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
    })
}

fn build_for_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let for_tk: &Token = parser_ctx.consume(
        TokenKind::For,
        String::from("Syntax error"),
        String::from("Expected 'for' keyword."),
    )?;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            for_tk.span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("For loop must be placed inside a function or a bind."),
            String::default(),
            for_tk.span,
        ));
    }

    let local: Instruction = build_local(parser_ctx, false)?;
    let cond: Instruction = expression::build_expression(parser_ctx)?;

    parser_ctx.mismatch_types(&Type::Bool, cond.get_type(), cond.get_span(), Some(&cond));

    let actions: Instruction = expression::build_expression(parser_ctx)?;

    let mut local_clone: Instruction = local.clone();

    if let Instruction::Local { comptime, .. } = &mut local_clone {
        *comptime = true;
    }

    parser_ctx.add_lift_local(local_clone);

    parser_ctx.get_mut_control_ctx().set_inside_loop(true);

    let body: Instruction = build_block(parser_ctx)?;

    parser_ctx.get_mut_control_ctx().set_inside_loop(false);

    Ok(Instruction::ForLoop {
        variable: local.into(),
        cond: cond.into(),
        actions: actions.into(),
        block: body.into(),
    })
}

fn build_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let loop_tk: &Token = parser_ctx.consume(
        TokenKind::Loop,
        String::from("Syntax error"),
        String::from("Expected 'loop' keyword."),
    )?;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            loop_tk.span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Loop must be placed inside a function or a bind."),
            String::default(),
            loop_tk.span,
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
    })
}

fn build_while_loop<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let while_tk: &Token = parser_ctx.consume(
        TokenKind::While,
        String::from("Syntax error"),
        String::from("Expected 'while' keyword."),
    )?;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            while_tk.span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("While loop must be placed inside a function or a bind."),
            String::default(),
            while_tk.span,
        ));
    }

    let conditional: Instruction = expression::build_expr(parser_ctx)?;

    parser_ctx.mismatch_types(
        &Type::Bool,
        conditional.get_type(),
        conditional.get_span(),
        Some(&conditional),
    );

    let block: Instruction = build_block(parser_ctx)?;

    Ok(Instruction::WhileLoop {
        cond: conditional.into(),
        block: block.into(),
    })
}

fn build_continue<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let continue_tk: &Token = parser_ctx.consume(
        TokenKind::Continue,
        String::from("Syntax error"),
        String::from("Expected 'continue' keyword."),
    )?;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            continue_tk.span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Continue must be placed inside a function or a bind."),
            String::default(),
            continue_tk.span,
        ));
    }

    let scope: usize = parser_ctx.get_scope();

    parser_ctx
        .get_mut_control_ctx()
        .set_unreacheable_code_scope(scope);

    if !parser_ctx.get_control_ctx().get_inside_loop() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("The flow changer of a loop must go inside one."),
            String::default(),
            parser_ctx.previous().span,
        ));
    }

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Instruction::Continue)
}

fn build_break<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let break_tk: &Token = parser_ctx.consume(
        TokenKind::Break,
        String::from("Syntax error"),
        String::from("Expected 'break' keyword."),
    )?;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            break_tk.span,
        ));
    }

    let scope: usize = parser_ctx.get_scope();

    parser_ctx
        .get_mut_control_ctx()
        .set_unreacheable_code_scope(scope);

    if !parser_ctx.get_control_ctx().get_inside_loop() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("The flow changer of a loop must go inside one."),
            String::default(),
            parser_ctx.previous().span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Break must be placed inside a function or a bind."),
            String::default(),
            break_tk.span,
        ));
    }

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Instruction::Break)
}

fn build_match<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let match_tk: &Token = parser_ctx.consume(
        TokenKind::Match,
        String::from("Syntax error"),
        String::from("Expected 'match' keyword."),
    )?;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            match_tk.span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Match must be placed inside a function or a bind."),
            String::default(),
            match_tk.span,
        ));
    }

    let mut start_pattern: Instruction = expression::build_expr(parser_ctx)?;
    let mut start_block: Instruction = Instruction::Block { stmts: Vec::new() };

    let mut patterns: Vec<Instruction> = Vec::with_capacity(10);
    let mut patterns_stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

    let mut position: u32 = 0;

    while parser_ctx.match_token(TokenKind::Pattern)? {
        *parser_ctx.get_mut_scope() += 1;

        parser_ctx.get_mut_symbols().begin_local_scope();

        let pattern: Instruction = expression::build_expr(parser_ctx)?;

        parser_ctx.mismatch_types(
            &Type::Bool,
            pattern.get_type(),
            pattern.get_span(),
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
                }),
            });

            patterns_stmts.clear();
            position += 1;

            continue;
        }

        start_pattern = pattern;

        start_block = Instruction::Block {
            stmts: patterns_stmts.clone(),
        };

        patterns_stmts.clear();
        position += 1;
    }

    if start_block.has_instruction() {
        parser_ctx.mismatch_types(
            &Type::Bool,
            start_pattern.get_type(),
            start_pattern.get_span(),
            Some(&start_pattern),
        );
    }

    let otherwise: Option<Rc<Instruction>> = if parser_ctx.match_token(TokenKind::Else)? {
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
                    block: Instruction::Block { stmts }.into(),
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
    })
}

fn build_if_elif_else<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let if_tk: &Token = parser_ctx.consume(
        TokenKind::If,
        String::from("Syntax error"),
        String::from("Expected 'if' keyword."),
    )?;

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Conditionals must be placed inside a function or a bind."),
            String::default(),
            if_tk.span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            if_tk.span,
        ));
    }

    let if_condition: Instruction = expression::build_expr(parser_ctx)?;

    parser_ctx.mismatch_types(
        &Type::Bool,
        if_condition.get_type(),
        if_condition.get_span(),
        Some(&if_condition),
    );

    let if_body: Rc<Instruction> = Rc::new(build_block(parser_ctx)?);

    let mut elfs: Vec<Instruction> = Vec::with_capacity(10);

    while parser_ctx.match_token(TokenKind::Elif)? {
        let elif_condition: Instruction = expression::build_expr(parser_ctx)?;

        parser_ctx.mismatch_types(
            &Type::Bool,
            elif_condition.get_type(),
            elif_condition.get_span(),
            Some(&elif_condition),
        );

        let elif_body: Instruction = build_block(parser_ctx)?;

        if !elif_body.has_instruction() {
            continue;
        }

        elfs.push(Instruction::Elif {
            cond: Rc::new(elif_condition),
            block: Rc::new(elif_body),
        });
    }

    let mut otherwise: Option<Rc<Instruction>> = None;

    if parser_ctx.match_token(TokenKind::Else)? {
        let else_body: Instruction = build_block(parser_ctx)?;

        if else_body.has_instruction() {
            otherwise = Some(
                Instruction::Else {
                    block: else_body.into(),
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
    })
}

pub fn build_custom_type<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let type_tk: &Token = parser_ctx.consume(
        TokenKind::Type,
        String::from("Syntax error"),
        String::from("Expected 'type' keyword."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Types are only defined globally."),
            String::default(),
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

    let custom_type_attributes: ThrushAttributes =
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

        let kind: Type = typegen::build_type(parser_ctx, Some(TokenKind::SemiColon))?;

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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let enum_tk: &Token = parser_ctx.consume(
        TokenKind::Enum,
        String::from("Syntax error"),
        String::from("Expected 'enum'."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Enums are only defined globally."),
            String::default(),
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

    let enum_attributes: ThrushAttributes =
        build_compiler_attributes(parser_ctx, &[TokenKind::LBrace])?;

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

            let field_type: Type = typegen::build_type(parser_ctx, None)?;

            if !field_type.is_integer_type()
                && !field_type.is_float_type()
                && !field_type.is_bool_type()
            {
                return Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("Expected integer, boolean or floating-point types."),
                    String::default(),
                    span,
                ));
            }

            if parser_ctx.match_token(TokenKind::SemiColon)? {
                let field_value: Instruction = if field_type.is_float_type() {
                    Instruction::Float(field_type.clone(), index, false, span)
                } else if field_type.is_bool_type() {
                    Instruction::Boolean(Type::Bool, index != 0.0, span)
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

            let expression: Instruction = expression::build_expr(parser_ctx)?;

            expression.throw_attemping_use_jit(expression.get_span())?;

            parser_ctx.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            parser_ctx.mismatch_types(
                &field_type,
                expression.get_type(),
                expression.get_span(),
                Some(&expression),
            );

            enum_fields.push((name, expression));

            continue;
        }

        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Expected identifier in enum field."),
            String::default(),
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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let struct_tk: &Token = parser_ctx.consume(
        TokenKind::Struct,
        String::from("Syntax error"),
        String::from("Expected 'struct' keyword."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Structs are only defined globally."),
            String::default(),
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

    let struct_attributes: ThrushAttributes =
        build_compiler_attributes(parser_ctx, &[TokenKind::LBrace])?;

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

        if parser_ctx.match_token(TokenKind::Identifier)? {
            let field_name: &str = parser_ctx.previous().lexeme;

            // solucionar con Self type talvez?
            if parser_ctx.peek().lexeme == struct_name {
                todo!()
            }

            let field_type: Type = typegen::build_type(parser_ctx, Some(TokenKind::SemiColon))?;

            fields_types
                .1
                .push((field_name, field_type, field_position));

            field_position += 1;

            continue;
        }

        parser_ctx.only_advance()?;

        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Expected identifier in structure field."),
            String::default(),
            parser_ctx.previous().span,
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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
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
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Constants are only defined globally."),
            String::default(),
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

    let const_type: Type = typegen::build_type(parser_ctx, None)?;

    let const_attributes: ThrushAttributes =
        build_compiler_attributes(parser_ctx, &[TokenKind::Eq])?;

    parser_ctx.consume(
        TokenKind::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Instruction = expression::build_expr(parser_ctx)?;

    value.throw_attemping_use_jit(span)?;

    parser_ctx.mismatch_types(
        &const_type,
        value.get_type(),
        value.get_span(),
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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    parser_ctx
        .get_mut_type_ctx()
        .set_position(TypePosition::Local);

    let local_tk: &Token = parser_ctx.consume(
        TokenKind::Local,
        String::from("Syntax error"),
        String::from("Expected 'local' keyword."),
    )?;

    if parser_ctx.is_main_scope() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Locals variables should be contained at local scope."),
            String::default(),
            local_tk.span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
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

    let local_type: Type = typegen::build_type(parser_ctx, None)?;

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

    let value: Instruction = expression::build_expr(parser_ctx)?;

    parser_ctx.mismatch_types(
        &local_type,
        value.get_type(),
        value.get_span(),
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
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let return_tk: &Token = parser_ctx.consume(
        TokenKind::Return,
        String::from("Syntax error"),
        String::from("Expected 'return' keyword."),
    )?;

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Return outside of bind or function."),
            String::default(),
            return_tk.span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            return_tk.span,
        ));
    }

    let function_type: Type = parser_ctx.get_type_ctx().get_function_type();

    if parser_ctx.match_token(TokenKind::SemiColon)? {
        if parser_ctx.get_type_ctx().get_function_type().is_void_type() {
            return Ok(Instruction::Null);
        }

        parser_ctx.mismatch_types(
            &Type::Void,
            &function_type,
            parser_ctx.previous().span,
            None,
        );

        return Ok(Instruction::Return(Type::Void, Instruction::Null.into()));
    }

    let value: Instruction = expression::build_expr(parser_ctx)?;

    parser_ctx.mismatch_types(
        &function_type,
        value.get_type(),
        value.get_span(),
        Some(&value),
    );

    parser_ctx.consume(
        TokenKind::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Instruction::Return(
        parser_ctx.get_type_ctx().get_function_type().clone(),
        value.into(),
    ))
}

fn build_block<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
    let block_tk: &Token = parser_ctx.consume(
        TokenKind::LBrace,
        String::from("Syntax error"),
        String::from("Expected '{'."),
    )?;

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            block_tk.span,
        ));
    }

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Block of code must be placed inside a function or a bind."),
            String::default(),
            block_tk.span,
        ));
    }

    *parser_ctx.get_mut_scope() += 1;
    parser_ctx.get_mut_symbols().begin_local_scope();

    let scope: usize = parser_ctx.get_scope();

    parser_ctx.get_mut_symbols().lift_instructions(scope)?;

    let mut stmts: Vec<Instruction> = Vec::with_capacity(100);

    while !parser_ctx.match_token(TokenKind::RBrace)? {
        let instruction: Instruction = statement(parser_ctx)?;
        stmts.push(instruction)
    }

    parser_ctx.get_mut_symbols().end_local_scope();
    *parser_ctx.get_mut_scope() -= 1;

    Ok(Instruction::Block { stmts })
}

pub fn build_function<'instr>(
    parser_ctx: &mut ParserContext<'instr>,
    declare: bool,
) -> Result<Instruction<'instr>, ThrushCompilerError> {
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

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Functions are only defined globally."),
            String::default(),
            function_name_tk.span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            String::default(),
            function_name_tk.span,
        ));
    }

    let function_name: &str = function_name_tk.lexeme;
    let function_span: Span = function_name_tk.span;

    if function_name == "main" {
        if declare {
            return Ok(Instruction::Null);
        }

        parser_ctx.get_mut_control_ctx().set_inside_function(true);

        let entrypoint: Result<Instruction, ThrushCompilerError> = build_entry_point(parser_ctx);

        parser_ctx.get_mut_control_ctx().set_inside_function(false);

        return entrypoint;
    }

    let mut parameters: Vec<Instruction> = Vec::with_capacity(10);
    let mut parameters_types: Vec<Type> = Vec::with_capacity(10);

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
            TokenKind::ColonColon,
            String::from("Syntax error"),
            String::from("Expected '::'."),
        )?;

        let parameter_type: Type = typegen::build_type(parser_ctx, None)?;

        if parameter_type.is_void_type() {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Void type are not allowed as parameters."),
                String::default(),
                parameter_span,
            ));
        }

        parser_ctx
            .get_mut_type_ctx()
            .set_position(TypePosition::NoRelevant);

        parameters_types.push(parameter_type.clone());

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

    let return_type: Type = typegen::build_type(parser_ctx, None)?;

    parser_ctx
        .get_mut_type_ctx()
        .set_function_type(return_type.clone());

    let function_attributes: ThrushAttributes =
        build_compiler_attributes(parser_ctx, &[TokenKind::SemiColon, TokenKind::LBrace])?;

    let function_has_ffi: bool = function_attributes.contain_ffi_attribute();
    let function_has_ignore: bool = function_attributes.contain_ignore_attribute();

    if function_has_ignore && !function_has_ffi {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from(
                "The '@ignore' attribute can only be used if the function contains the '@extern' attribute.",
            ),
            String::default(),
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
    };

    if function_has_ffi || declare {
        if declare {
            parser_ctx.get_mut_symbols().new_function(
                function_name,
                (
                    return_type,
                    Parameters::new(parameters_types),
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

    parameters.iter().cloned().for_each(|parameter| {
        parser_ctx.add_lift_local(parameter);
    });

    parser_ctx.get_mut_control_ctx().set_inside_function(true);

    let function_body: Rc<Instruction> = build_block(parser_ctx)?.into();

    parser_ctx.get_mut_control_ctx().set_inside_function(false);

    if !return_type.is_void_type() && !function_body.has_return() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            format!("Missing return with type '{}'.", return_type),
            String::default(),
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
) -> Result<ThrushAttributes<'instr>, ThrushCompilerError> {
    let mut compiler_attributes: ThrushAttributes = Vec::with_capacity(10);

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
) -> Result<&'instr str, ThrushCompilerError> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let name: &Token = parser_ctx.consume(
        TokenKind::Str,
        String::from("Syntax error"),
        String::from("Expected a string for @extern(\"FFI NAME\")."),
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
) -> Result<CallConvention, ThrushCompilerError> {
    parser_ctx.only_advance()?;

    parser_ctx.consume(
        TokenKind::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let convention_tk: &Token = parser_ctx.consume(
        TokenKind::Str,
        String::from("Syntax error"),
        String::from("Expected a string for @convention(\"CONVENTION NAME\")."),
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

    Err(ThrushCompilerError::Error(
        String::from("Syntax error"),
        String::from("Unknown call convention."),
        String::default(),
        span,
    ))
}
