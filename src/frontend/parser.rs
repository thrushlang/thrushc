use super::{
    lexer::{Token, TokenKind, Type},
    objects::{FoundObjectId, Function, Functions, Local, ParserObjects, Struct},
    scoper::ThrushScoper,
    traits::{
        FoundObjectEither, FoundObjectExtensions, StructureExtensions, TokenLexemeExtensions,
    },
    type_checking,
    types::Constructor,
};

use super::super::{
    backend::compiler::{
        attributes::LLVMAttribute, builtins, conventions::CallConvention, instruction::Instruction,
        misc::CompilerFile, traits::AttributesExtensions, types::ThrushAttributes,
    },
    common::{
        constants::MINIMAL_ERROR_CAPACITY, diagnostic::Diagnostician, error::ThrushCompilerError,
    },
    logging::LoggingType,
};

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;
use std::{mem, process};

const MINIMAL_STATEMENT_CAPACITY: usize = 100_000;
const MINIMAL_GLOBAL_CAPACITY: usize = 2024;
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

pub struct Parser<'instr> {
    stmts: Vec<Instruction<'instr>>,
    tokens: &'instr [Token<'instr>],
    errors: Vec<ThrushCompilerError>,
    inside_a_function: bool,
    inside_a_loop: bool,
    in_function_type: Instruction<'instr>,
    in_local_type: Instruction<'instr>,
    in_unreacheable_code: usize,
    current: usize,
    scope_position: usize,
    has_entry_point: bool,
    scoper: ThrushScoper<'instr>,
    diagnostician: Diagnostician,
    parser_objects: ParserObjects<'instr>,
}

impl<'instr> Parser<'instr> {
    pub fn new(tokens: &'instr Vec<Token<'instr>>, file: &'instr CompilerFile) -> Self {
        let mut functions: Functions = HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY);

        builtins::include(&mut functions);

        Self {
            stmts: Vec::with_capacity(MINIMAL_STATEMENT_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            tokens,
            current: 0,
            inside_a_function: false,
            inside_a_loop: false,
            in_function_type: Instruction::Null,
            in_local_type: Instruction::Null,
            in_unreacheable_code: 0,
            scope_position: 0,
            has_entry_point: false,
            scoper: ThrushScoper::new(file),
            diagnostician: Diagnostician::new(file),
            parser_objects: ParserObjects::with_functions(functions),
        }
    }

    pub fn start(&mut self) -> &[Instruction<'instr>] {
        self.declare();

        while !self.end() {
            match self.parse() {
                Ok(instr) => {
                    self.stmts.push(instr);
                }
                Err(e) => {
                    self.errors.push(e);
                    self.sync();
                }
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error: &ThrushCompilerError| {
                self.diagnostician.report_error(error, LoggingType::Error);
            });

            process::exit(1);
        }

        self.scoper.analyze();

        self.stmts.as_slice()
    }

    fn parse(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        match &self.peek().kind {
            TokenKind::Struct => Ok(self.build_struct(false)?),
            TokenKind::Fn => Ok(self.build_function(false)?),
            TokenKind::LBrace => Ok(self.build_code_block(&mut [])?),
            TokenKind::Return => Ok(self.build_return()?),
            TokenKind::Local => Ok(self.build_local(false)?),
            TokenKind::For => Ok(self.build_for_loop()?),
            TokenKind::New => Ok(self.build_struct_initializer()?),
            TokenKind::If => Ok(self.build_if_elif_else()?),
            TokenKind::Match => Ok(self.build_match()?),
            TokenKind::While => Ok(self.build_while_loop()?),
            TokenKind::Continue => Ok(self.build_continue()?),
            TokenKind::Break => Ok(self.build_break()?),
            TokenKind::Loop => Ok(self.build_loop()?),
            _ => Ok(self.expression()?),
        }
    }

    fn build_entry_point(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        if self.has_entry_point {
            self.push_error(
                String::from("Duplicated entrypoint"),
                String::from("The language not support two entrypoints."),
            );
        }

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
        )?;

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        self.has_entry_point = true;

        let body: Box<Instruction> = Box::new(self.build_code_block(&mut [])?);

        self.inside_a_function = false;

        Ok(Instruction::EntryPoint { body })
    }

    fn build_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        self.inside_a_loop = true;

        let block: Instruction = self.build_code_block(&mut [])?;

        if !block.has_break() && !block.has_return() && !block.has_continue() {
            self.in_unreacheable_code = self.scope_position;
        }

        self.inside_a_loop = false;

        Ok(Instruction::Loop {
            block: Box::new(block),
        })
    }

    fn build_while_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        let conditional: Instruction = self.expr()?;

        self.check_type_mismatch(
            Instruction::ComplexType(Type::Bool, ""),
            conditional.get_type(),
            Some(&conditional),
        );

        let block: Instruction = self.build_code_block(&mut [])?;

        Ok(Instruction::WhileLoop {
            cond: Box::new(conditional),
            block: Box::new(block),
        })
    }

    fn build_continue(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        self.in_unreacheable_code = self.scope_position;

        self.throw_if_not_inside_a_loop();

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Continue)
    }

    fn build_break(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        self.in_unreacheable_code = self.scope_position;

        self.throw_if_not_inside_a_loop();

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Break)
    }

    fn build_match(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        let mut if_cond: Instruction = self.expr()?;

        let mut if_block: Instruction = Instruction::Block { stmts: Vec::new() };

        let mut patterns: Vec<Instruction> = Vec::with_capacity(10);
        let mut patterns_stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

        let mut index: u32 = 0;

        while self.match_token(TokenKind::Pattern)? {
            self.scope_position += 1;
            self.parser_objects.begin_local_scope();

            let pattern: Instruction = self.expr()?;

            self.check_type_mismatch(
                Instruction::ComplexType(Type::Bool, ""),
                pattern.get_type(),
                Some(&pattern),
            );

            self.consume(
                TokenKind::ColonColon,
                String::from("Syntax error"),
                String::from("Expected '::'."),
            )?;

            while !self.match_token(TokenKind::Break)? {
                patterns_stmts.push(self.parse()?);
            }

            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            self.scope_position -= 1;
            self.parser_objects.end_local_scope();

            if patterns_stmts.is_empty() {
                continue;
            }

            if index != 0 {
                patterns.push(Instruction::Elif {
                    cond: Box::new(pattern),
                    block: Box::new(Instruction::Block {
                        stmts: patterns_stmts.clone(),
                    }),
                });

                patterns_stmts.clear();
                index += 1;

                continue;
            }

            if_cond = pattern;
            if_block = Instruction::Block {
                stmts: patterns_stmts.clone(),
            };

            patterns_stmts.clear();
            index += 1;
        }

        if if_block.has_instruction() {
            self.check_type_mismatch(
                Instruction::ComplexType(Type::Bool, ""),
                if_cond.get_type(),
                Some(&if_cond),
            );
        }

        let otherwise: Option<Box<Instruction>> = if self.match_token(TokenKind::Else)? {
            self.consume(
                TokenKind::ColonColon,
                String::from("Syntax error"),
                String::from("Expected '::'."),
            )?;

            let mut stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

            while !self.match_token(TokenKind::Break)? {
                stmts.push(self.parse()?);
            }

            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            if stmts.is_empty() {
                None
            } else {
                Some(Box::new(Instruction::Else {
                    block: Box::new(Instruction::Block { stmts }),
                }))
            }
        } else {
            None
        };

        if !if_block.has_instruction() && patterns.is_empty() && otherwise.is_none() {
            return Ok(Instruction::Null);
        }

        Ok(Instruction::If {
            cond: Box::new(if_cond),
            block: Box::new(if_block),
            elfs: patterns,
            otherwise,
        })
    }

    fn build_if_elif_else(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        if !self.inside_a_function {
            self.push_error(
                String::from("Syntax error"),
                String::from("The if-elif-else must be placed inside a function."),
            );
        }

        self.throw_if_is_unreacheable_code();

        let if_condition: Instruction = self.expr()?;

        if !if_condition.get_basic_type().is_bool_type() {
            self.push_error(
                String::from("Syntax error"),
                String::from("Condition must be type boolean."),
            );
        }

        let if_body: Box<Instruction> = Box::new(self.build_code_block(&mut [])?);

        let mut elfs: Vec<Instruction> = Vec::with_capacity(10);

        while self.match_token(TokenKind::Elif)? {
            let elif_condition: Instruction = self.expr()?;

            self.check_type_mismatch(
                Instruction::ComplexType(Type::Bool, ""),
                elif_condition.get_type(),
                Some(&elif_condition),
            );

            let elif_body: Instruction = self.build_code_block(&mut [])?;

            if !elif_body.has_instruction() {
                continue;
            }

            elfs.push(Instruction::Elif {
                cond: Box::new(elif_condition),
                block: Box::new(elif_body),
            });
        }

        let mut otherwise: Option<Box<Instruction>> = None;

        if self.match_token(TokenKind::Else)? {
            let else_body: Instruction = self.build_code_block(&mut [])?;

            if else_body.has_instruction() {
                otherwise = Some(Box::new(Instruction::Else {
                    block: Box::new(else_body),
                }));
            }
        }

        Ok(Instruction::If {
            cond: Box::new(if_condition),
            block: if_body,
            elfs,
            otherwise,
        })
    }

    fn build_struct(&mut self, declare: bool) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name."),
        )?;

        let struct_name: &str = name.lexeme.to_str();

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut fields_types: Vec<(&str, Instruction, u32)> = Vec::with_capacity(10);
        let mut field_position: u32 = 0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_name: &str = self.previous().lexeme.to_str();
                let line: usize = self.previous().line;
                let span: (usize, usize) = self.previous().span;

                let field_type: Instruction = if self.peek().lexeme.to_str() != struct_name {
                    self.expression()?
                } else {
                    self.only_advance()?;

                    self.consume(
                        TokenKind::SemiColon,
                        String::from("Syntax error"),
                        String::from("Expected ';'."),
                    )?;

                    Instruction::ComplexType(Type::Struct, struct_name)
                };

                field_type.expected_type(line, span)?;

                fields_types.push((field_name, field_type, field_position));
                field_position += 1;

                continue;
            }

            self.only_advance()?;

            self.push_error(
                String::from("Syntax error"),
                String::from("Expected identifier in structure field."),
            );
        }

        self.consume(
            TokenKind::RBrace,
            String::from("Syntax error"),
            String::from("Expected '}'."),
        )?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        if declare {
            self.parser_objects
                .insert_new_struct(name.lexeme.to_str(), fields_types);

            return Ok(Instruction::Null);
        }

        Ok(Instruction::Struct {
            name: struct_name,
            fields_types,
        })
    }

    fn build_struct_initializer(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected structure reference."),
        )?;

        let structure_name: &str = name.lexeme.to_str();

        let line: usize = name.line;
        let span: (usize, usize) = name.span;

        self.throw_if_is_unreacheable_code();

        let struct_found: Struct = self
            .parser_objects
            .get_struct(name.lexeme.to_str(), (line, span))?;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut arguments: Constructor = Vec::with_capacity(10);

        let mut field_index: u32 = 0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_name: &str = self.previous().lexeme.to_str();

                let fields_required: usize = struct_found.len();

                if !struct_found.contains_field(field_name) {
                    self.push_error(
                        String::from("Syntax error"),
                        String::from("Expected existing structure field name."),
                    );
                }

                if field_index as usize >= fields_required {
                    self.push_error(
                        String::from("Too many fields in structure"),
                        format!(
                            "Expected '{}' fields, not '{}'.",
                            fields_required, field_index
                        ),
                    );

                    field_index = (fields_required - 1) as u32;
                }

                let expression: Instruction = self.expr()?;

                self.throw_if_is_structure_initializer(&expression);

                let expression_type: Instruction = expression.get_type();

                if let Some(target_type) = struct_found.get_field_type(field_name) {
                    self.check_type_mismatch(
                        target_type.clone(),
                        expression_type,
                        Some(&expression),
                    );

                    arguments.push((field_name, expression, target_type, field_index));
                }

                field_index += 1;

                continue;
            }

            self.only_advance()?;
        }

        let fields_size: usize = arguments.len();
        let fields_needed_size: usize = struct_found.len();

        if fields_size != fields_needed_size {
            self.push_error(
                String::from("Missing fields"),
                format!(
                    "Expected '{}' arguments, but '{}' was gived.",
                    fields_needed_size, fields_size
                ),
            );
        }

        self.consume(
            TokenKind::RBrace,
            String::from("Syntax error"),
            String::from("Expected '}'."),
        )?;

        Ok(Instruction::InitStruct {
            name: structure_name,
            arguments,
        })
    }

    fn build_for_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        let variable: Instruction = self.build_local(false)?;

        let conditional: Instruction = self.expression()?;

        self.check_type_mismatch(
            Instruction::ComplexType(Type::Bool, ""),
            conditional.get_type(),
            None,
        );

        let actions: Instruction = self.expression()?;

        let mut variable_clone: Instruction = variable.clone();

        if let Instruction::Local { comptime, .. } = &mut variable_clone {
            *comptime = true;
        }

        self.inside_a_loop = true;

        let body: Instruction = self.build_code_block(&mut [variable_clone])?;

        self.inside_a_loop = false;

        Ok(Instruction::ForLoop {
            variable: Box::new(variable),
            cond: Box::new(conditional),
            actions: Box::new(actions),
            block: Box::new(body),
        })
    }

    fn build_local(&mut self, comptime: bool) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        if self.scope_position == 0 {
            self.push_error(
                String::from("Syntax error"),
                String::from("Locals variables should be contained at local scope."),
            );
        }

        self.throw_if_is_unreacheable_code();

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name."),
        )?;

        self.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let local_type: Instruction = self.expr()?;

        local_type.expected_type(self.previous().line, self.previous().span)?;

        self.parser_objects.insert_new_local(
            self.scope_position,
            name.lexeme.to_str(),
            (local_type.clone(), false, false),
            name.line,
            name.span,
        )?;

        if self.match_token(TokenKind::SemiColon)? {
            return Ok(Instruction::Local {
                name: name.lexeme.to_str(),
                kind: Box::new(local_type),
                value: Box::new(Instruction::Null),
                line: name.line,
                comptime,
            });
        }

        self.consume(
            TokenKind::Eq,
            String::from("Syntax error"),
            String::from("Expected '='."),
        )?;

        self.in_local_type = local_type.clone();

        let local_value: Instruction = self.expr()?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        self.check_type_mismatch(
            local_type.clone(),
            local_value.get_type(),
            Some(&local_value),
        );

        let local: Instruction = Instruction::Local {
            name: name.lexeme.to_str(),
            kind: Box::new(local_type),
            value: Box::new(local_value),
            line: name.line,
            comptime,
        };

        Ok(local)
    }

    fn build_return(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        if !self.inside_a_function {
            self.push_error(
                String::from("Syntax error"),
                String::from("Return outside of function body."),
            );
        }

        self.throw_if_is_unreacheable_code();

        if self.match_token(TokenKind::SemiColon)? {
            let basic_function_type: &Type = self.in_function_type.get_basic_type();

            if basic_function_type.is_void_type() {
                return Ok(Instruction::Null);
            }

            self.check_type_mismatch(
                Instruction::ComplexType(Type::Void, ""),
                self.in_function_type.clone(),
                None,
            );

            return Ok(Instruction::Return(
                Box::new(Instruction::Null),
                Box::new(Instruction::ComplexType(Type::Void, "")),
            ));
        }

        let value: Instruction = self.expr()?;

        self.check_type_mismatch(
            self.in_function_type.clone(),
            value.get_type(),
            Some(&value),
        );

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Return(
            Box::new(value),
            Box::new(self.in_function_type.clone()),
        ))
    }

    fn build_code_block(
        &mut self,
        with_instrs: &mut [Instruction<'instr>],
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        self.throw_if_is_unreacheable_code();

        self.scope_position += 1;
        self.parser_objects.begin_local_scope();

        let mut stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

        for instruction in with_instrs.iter_mut() {
            if let Instruction::FunctionParameter {
                name,
                kind,
                line,
                span,
                ..
            } = instruction
            {
                self.parser_objects.insert_new_local(
                    self.scope_position,
                    name,
                    ((**kind).clone(), false, false),
                    *line,
                    *span,
                )?;
            }

            stmts.push(mem::take(instruction));
        }

        while !self.match_token(TokenKind::RBrace)? {
            let instruction: Instruction = self.parse()?;
            stmts.push(instruction)
        }

        self.parser_objects.end_local_scope();

        self.scoper.add_scope(stmts.clone());
        self.scope_position -= 1;

        Ok(Instruction::Block { stmts })
    }

    fn build_function(
        &mut self,
        declare: bool,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        if self.scope_position != 0 {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Functions are only defined globally."),
                self.previous().line,
                Some(self.previous().span),
            ));
        }

        self.inside_a_function = true;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name to the function."),
        )?;

        let function_name: &str = name.lexeme.to_str();

        if name.lexeme.to_str() == "main" {
            if declare {
                return Ok(Instruction::Null);
            }

            return self.build_entry_point();
        }

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
        )?;

        let mut params: Vec<Instruction> = Vec::with_capacity(10);
        let mut parameters_types: Vec<Instruction> = Vec::with_capacity(10);
        let mut parameter_position: u32 = 0;

        while !self.match_token(TokenKind::RParen)? {
            let parameter_line: usize = self.previous().line;
            let parameter_span: (usize, usize) = self.previous().span;

            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            self.consume(
                TokenKind::Identifier,
                String::from("Syntax error"),
                String::from("Expected parameter name."),
            )?;

            let parameter_name: &str = self.previous().lexeme.to_str();

            self.consume(
                TokenKind::ColonColon,
                String::from("Syntax error"),
                String::from("Expected '::'."),
            )?;

            let parameter_type: Instruction = self.expr()?;

            parameters_types.push(parameter_type.get_type());

            params.push(Instruction::FunctionParameter {
                name: parameter_name,
                kind: Box::new(parameter_type),
                position: parameter_position,
                line: parameter_line,
                span: parameter_span,
            });

            parameter_position += 1;
        }

        let return_type: Instruction = self.expr()?;

        return_type.expected_type(self.previous().line, self.previous().span)?;

        let function_attributes: ThrushAttributes = self.build_compiler_attributes()?;

        let function_has_ffi: bool = function_attributes.contain_ffi_attribute();
        let function_has_ignore: bool = function_attributes.contain_ignore_attribute();

        if function_has_ignore && !function_has_ffi {
            self.push_error(
                String::from("Syntax error"),
                String::from(
                    "The '@ignore' attribute can only be used if the function contains the '@extern' attribute.",
                )
            );
        }

        self.in_function_type = return_type.clone();

        let mut function: Instruction = Instruction::Function {
            name: function_name,
            params: params.clone(),
            body: None,
            return_type: Box::new(return_type.clone()),
            attributes: function_attributes,
        };

        if function_has_ffi || declare {
            if declare {
                self.parser_objects.insert_new_function(
                    function_name,
                    (return_type, parameters_types, function_has_ignore),
                );
            }

            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            self.inside_a_function = false;
            return Ok(function);
        }

        let function_body: Box<Instruction> = Box::new(self.build_code_block(&mut params)?);

        if !return_type.is_void_type() && !function_body.has_return() {
            self.push_error(
                String::from("Syntax error"),
                format!(
                    "Missing return type with type '{}'.",
                    return_type.get_basic_type()
                ),
            );
        }

        self.inside_a_function = false;

        if let Instruction::Function { body, .. } = &mut function {
            *body = Some(function_body);
        }

        Ok(function)
    }

    /* ######################################################################


        COMPILER ATTRIBUTES BUILDER


    ########################################################################*/

    fn build_compiler_attributes(
        &mut self,
    ) -> Result<ThrushAttributes<'instr>, ThrushCompilerError> {
        let mut compiler_attributes: ThrushAttributes = Vec::with_capacity(10);

        while !self.check_type(TokenKind::SemiColon) && !self.check_type(TokenKind::LParen) {
            match self.peek().kind {
                TokenKind::Extern => {
                    compiler_attributes.push(LLVMAttribute::FFI(self.build_external_attribute()?));
                }
                TokenKind::Convention => {
                    compiler_attributes.push(LLVMAttribute::Convention(
                        self.build_call_convention_attribute()?,
                    ));
                }
                TokenKind::Public => {
                    compiler_attributes.push(LLVMAttribute::Public(true));
                    self.only_advance()?;
                }

                attribute if attribute.as_compiler_attribute().is_some() => {
                    if let Some(compiler_attribute) = attribute.as_compiler_attribute() {
                        compiler_attributes.push(compiler_attribute);
                        self.only_advance()?;
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

    fn build_external_attribute(&mut self) -> Result<&'instr str, ThrushCompilerError> {
        self.only_advance()?;

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
        )?;

        let name: &Token = self.consume(
            TokenKind::Str,
            String::from("Syntax error"),
            String::from("Expected a string for @extern(\"FFI NAME\")."),
        )?;

        let ffi_name: &str = name.lexeme.to_str();

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        Ok(ffi_name)
    }

    fn build_call_convention_attribute(&mut self) -> Result<CallConvention, ThrushCompilerError> {
        self.only_advance()?;

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
        )?;

        let name: &Token = self.consume(
            TokenKind::Str,
            String::from("Syntax error"),
            String::from("Expected a string for @convention(\"CONVENTION NAME\")."),
        )?;

        if let Some(call_convention) = CALL_CONVENTIONS.get(name.lexeme) {
            self.consume(
                TokenKind::RParen,
                String::from("Syntax error"),
                String::from("Expected ')'."),
            )?;

            return Ok(*call_convention);
        }

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Unknown call convention."),
            name.line,
            Some(name.span),
        ))
    }

    /* ######################################################################


        PARSER EXPRESSIONS


    ########################################################################*/

    fn expression(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let instruction: Instruction = self.or()?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(instruction)
    }

    fn expr(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let instruction: Instruction = self.or()?;

        Ok(instruction)
    }

    fn or(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.and()?;

        while self.match_token(TokenKind::Or)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.and()?;

            type_checking::check_binary_types(
                op,
                expression.get_basic_type(),
                right.get_basic_type(),
                (self.previous().line, self.previous().span),
            )?;

            expression = Instruction::BinaryOp {
                left: Box::new(expression),
                op,
                right: Box::new(right),
                kind: Box::new(Instruction::ComplexType(Type::Bool, "")),
            }
        }

        Ok(expression)
    }

    fn and(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.equality()?;

        while self.match_token(TokenKind::And)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.equality()?;

            type_checking::check_binary_types(
                op,
                expression.get_basic_type(),
                right.get_basic_type(),
                (self.previous().line, self.previous().span),
            )?;

            expression = Instruction::BinaryOp {
                left: Box::new(expression),
                op,
                right: Box::new(right),
                kind: Box::new(Instruction::ComplexType(Type::Bool, "")),
            }
        }

        Ok(expression)
    }

    fn equality(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.comparison()?;

        while self.match_token(TokenKind::BangEq)? || self.match_token(TokenKind::EqEq)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.comparison()?;

            let left_type: Type = *expression.get_basic_type();
            let right_type: Type = *right.get_basic_type();

            type_checking::check_binary_types(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            expression.is_chained(&right, (self.previous().line, self.previous().span))?;

            expression = Instruction::BinaryOp {
                left: Box::from(expression),
                op,
                right: Box::from(right),
                kind: Box::new(Instruction::ComplexType(Type::Bool, "")),
            }
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.term()?;

        while self.match_token(TokenKind::Greater)?
            || self.match_token(TokenKind::GreaterEq)?
            || self.match_token(TokenKind::Less)?
            || self.match_token(TokenKind::LessEq)?
        {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.term()?;

            let left_type: Type = *expression.get_basic_type();
            let right_type: Type = *right.get_basic_type();

            type_checking::check_binary_types(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            expression.is_chained(&right, (self.previous().line, self.previous().span))?;

            expression = Instruction::BinaryOp {
                left: Box::from(expression),
                op,
                right: Box::from(right),
                kind: Box::new(Instruction::ComplexType(Type::Bool, "")),
            };
        }

        Ok(expression)
    }

    fn term(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.factor()?;

        while self.match_token(TokenKind::Plus)?
            || self.match_token(TokenKind::Minus)?
            || self.match_token(TokenKind::LShift)?
            || self.match_token(TokenKind::RShift)?
        {
            let op: &Token = self.previous();
            let right: Instruction = self.factor()?;

            let left_type: Type = *expression.get_basic_type();
            let right_type: Type = *right.get_basic_type();

            type_checking::check_binary_types(
                &op.kind,
                &left_type,
                &right_type,
                (op.line, op.span),
            )?;

            let kind: Type = left_type.precompute_type(right_type);

            expression = Instruction::BinaryOp {
                left: Box::from(expression),
                op: &op.kind,
                right: Box::from(right),
                kind: Box::new(Instruction::ComplexType(kind, "")),
            };
        }

        Ok(expression)
    }

    fn factor(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.unary()?;

        while self.match_token(TokenKind::Slash)? || self.match_token(TokenKind::Star)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.unary()?;

            let left_type: Type = *expression.get_basic_type();
            let right_type: Type = *right.get_basic_type();

            type_checking::check_binary_types(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            let kind: Type = left_type.precompute_type(right_type);

            expression = Instruction::BinaryOp {
                left: Box::from(expression),
                op,
                right: Box::from(right),
                kind: Box::new(Instruction::ComplexType(kind, "")),
            };
        }

        Ok(expression)
    }

    fn unary(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        if self.match_token(TokenKind::Bang)? {
            let op: &TokenKind = &self.previous().kind;
            let expression: Instruction = self.primary()?;

            type_checking::check_unary_types(
                op,
                expression.get_basic_type(),
                (self.previous().line, self.previous().span),
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                expression: Box::from(expression),
                kind: Box::new(Instruction::ComplexType(Type::Bool, "")),
                is_pre: false,
            });
        }

        if self.match_token(TokenKind::Minus)? {
            let op: &TokenKind = &self.previous().kind;

            let mut expression: Instruction = self.primary()?;

            if let Instruction::Integer(kind, _, is_signed) = &mut expression {
                if op.is_minus_operator() {
                    *kind = Box::new(self.negate_numeric_type(kind));
                    *is_signed = true;
                }
            }

            if let Instruction::Float(_, _, is_signed) = &mut expression {
                if op.is_minus_operator() {
                    *is_signed = true;
                }
            }

            if let Instruction::LocalRef { kind, .. } = &mut expression {
                if kind.is_integer_type() && op.is_minus_operator() {
                    *kind = Box::new(kind.narrowing_cast());
                }
            }

            let expression_type: Type = *expression.get_basic_type();

            type_checking::check_unary_types(
                op,
                &expression_type,
                (self.previous().line, self.previous().span),
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                expression: Box::from(expression),
                kind: Box::new(Instruction::ComplexType(expression_type, "")),
                is_pre: false,
            });
        }

        let instr: Instruction = self.primary()?;

        Ok(instr)
    }

    fn primary(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let primary: Instruction = match &self.peek().kind {
            TokenKind::New => self.build_struct_initializer()?,

            TokenKind::PlusPlus => {
                let op: &TokenKind = &self.advance()?.kind;

                let expression: Instruction = self.expr()?;

                if !expression.is_local_reference() {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Only local references can be pre-incremented."),
                        self.previous().line,
                        Some(self.previous().span),
                    ));
                }

                let unaryop: Instruction = Instruction::UnaryOp {
                    op,
                    expression: Box::from(expression),
                    kind: Box::new(Instruction::ComplexType(Type::Void, "")),
                    is_pre: true,
                };

                return Ok(unaryop);
            }

            TokenKind::MinusMinus => {
                let op: &TokenKind = &self.advance()?.kind;

                let expression: Instruction = self.expr()?;

                if !expression.is_local_reference() {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Only local references can be pre-decremented."),
                        self.previous().line,
                        Some(self.previous().span),
                    ));
                }

                let unaryop: Instruction = Instruction::UnaryOp {
                    op,
                    expression: Box::from(expression),
                    kind: Box::new(Instruction::ComplexType(Type::Void, "")),
                    is_pre: true,
                };

                return Ok(unaryop);
            }

            TokenKind::DataType(tp) => {
                let datatype: &Token = self.advance()?;

                let line: usize = datatype.line;
                let span: (usize, usize) = datatype.span;

                match tp {
                    tp if tp.is_integer_type() => Instruction::ComplexType(*tp, ""),
                    tp if tp.is_float_type() => Instruction::ComplexType(*tp, ""),
                    tp if tp.is_bool_type() => Instruction::ComplexType(*tp, ""),
                    tp if tp.is_raw_ptr_type() => Instruction::ComplexType(*tp, ""),
                    tp if tp.is_void_type() => Instruction::ComplexType(*tp, ""),
                    tp if tp.is_str_type() => Instruction::ComplexType(*tp, ""),
                    what_heck_tp => {
                        return Err(ThrushCompilerError::Error(
                            String::from("Syntax error"),
                            format!(
                                "The type '{}' cannot be a value during the compile time.",
                                what_heck_tp
                            ),
                            line,
                            Some(span),
                        ));
                    }
                }
            }

            TokenKind::LParen => {
                let lparen: &Token = self.advance()?;

                let expression: Instruction = self.expression()?;
                let expression_type: Type = *expression.get_basic_type();

                if !expression.is_binary() && !expression.is_group() {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from(
                            "Grouping '(...)' is only allowed with binary expressions or other grouped expressions.",
                        ),
                        lparen.line,
                        Some((lparen.span.0, self.peek().span.1)),
                    ));
                }

                self.consume(
                    TokenKind::RParen,
                    String::from("Syntax error"),
                    String::from("Expected ')'."),
                )?;

                return Ok(Instruction::Group {
                    expression: Box::new(expression),
                    kind: Box::new(Instruction::ComplexType(expression_type, "")),
                });
            }

            TokenKind::Str => {
                let token: &Token = self.advance()?;
                let token_lexeme: &[u8] = token.lexeme;

                Instruction::Str(token_lexeme.parse_scapes(token.line, token.span)?)
            }

            TokenKind::Char => {
                let char: &Token = self.advance()?;

                Instruction::Char(char.lexeme[0])
            }

            kind => match kind {
                TokenKind::Integer(kind, number, is_signed) => {
                    self.only_advance()?;

                    Instruction::Integer(
                        Box::new(Instruction::ComplexType(*kind, "")),
                        *number,
                        *is_signed,
                    )
                }

                TokenKind::Float(kind, number, is_signed) => {
                    self.only_advance()?;

                    Instruction::Float(
                        Box::new(Instruction::ComplexType(*kind, "")),
                        *number,
                        *is_signed,
                    )
                }

                TokenKind::Identifier => {
                    let object_token: &Token = self.advance()?;

                    let object_name: &str = object_token.lexeme.to_str();
                    let object_span: (usize, usize) = object_token.span;
                    let object_line: usize = object_token.line;

                    self.throw_if_is_unreacheable_code();

                    let object: FoundObjectId = self
                        .parser_objects
                        .get_object_id(object_name, (object_line, object_span))?;

                    if object.is_structure() {
                        return Ok(Instruction::ComplexType(Type::Struct, object_name));
                    }

                    if self.match_token(TokenKind::Eq)? {
                        let object: FoundObjectId = self
                            .parser_objects
                            .get_object_id(object_name, (object_line, object_span))?;

                        let local_position: (&str, usize) =
                            object.expected_local((object_line, object_span))?;

                        let local: &Local = self.parser_objects.get_local_by_id(
                            (object_line, object_span),
                            local_position.0,
                            local_position.1,
                        )?;

                        let local_type: Instruction = local.0.clone();

                        let expression: Instruction = self.expression()?;

                        self.check_type_mismatch(
                            local_type.clone(),
                            expression.get_type(),
                            Some(&expression),
                        );

                        return Ok(Instruction::LocalMut {
                            name: object_name,
                            value: Box::new(expression),
                            kind: Box::new(local_type),
                        });
                    }

                    if self.match_token(TokenKind::LBracket)? {
                        return self.build_gep(object_name, (object_line, object_span));
                    }

                    if self.match_token(TokenKind::LParen)? {
                        return self.build_function_call(object_name, (object_line, object_span));
                    }

                    let object: FoundObjectId = self
                        .parser_objects
                        .get_object_id(object_name, (object_line, object_span))?;

                    let local_position: (&str, usize) =
                        object.expected_local((object_line, object_span))?;

                    let local: &Local = self.parser_objects.get_local_by_id(
                        (object_line, object_span),
                        local_position.0,
                        local_position.1,
                    )?;

                    let local_type: Instruction = local.0.clone();

                    if local.2 {
                        return Err(ThrushCompilerError::Error(
                            String::from("Syntax error"),
                            format!("Local reference '{}' is undefined for use.", object_name),
                            object_line,
                            Some(object_span),
                        ));
                    }

                    let localref: Instruction = Instruction::LocalRef {
                        name: object_name,
                        line: object_line,
                        kind: Box::new(local_type.clone()),
                    };

                    if self.match_token(TokenKind::PlusPlus)?
                        | self.match_token(TokenKind::MinusMinus)?
                    {
                        let op: &TokenKind = &self.previous().kind;

                        let local_basic_type: &Type = local_type.get_basic_type();

                        type_checking::check_unary_types(
                            op,
                            local_basic_type,
                            (object_line, object_span),
                        )?;

                        let unaryop: Instruction = Instruction::UnaryOp {
                            op,
                            expression: Box::from(localref),
                            kind: Box::new(Instruction::ComplexType(Type::Void, "")),
                            is_pre: false,
                        };

                        return Ok(unaryop);
                    }

                    localref
                }

                TokenKind::True => {
                    self.only_advance()?;
                    Instruction::Boolean(true)
                }

                TokenKind::False => {
                    self.only_advance()?;
                    Instruction::Boolean(false)
                }

                _ => {
                    let previous: &Token = self.advance()?;

                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        format!("Statement \"{}\" don't allowed.", previous.lexeme.to_str()),
                        previous.line,
                        Some(previous.span),
                    ));
                }
            },
        };

        Ok(primary)
    }

    fn build_gep(
        &mut self,
        name: &'instr str,
        location: (usize, (usize, usize)),
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let object: FoundObjectId = self
            .parser_objects
            .get_object_id(name, (location.0, location.1))?;

        let local_position: (&str, usize) = object.expected_local(location)?;

        let local: &Local = self.parser_objects.get_local_by_id(
            (location.0, location.1),
            local_position.0,
            local_position.1,
        )?;

        let local_type: Instruction = local.0.clone();

        self.check_type_mismatch(Instruction::ComplexType(Type::T, ""), local_type, None);

        let index: Instruction = self.expr()?;

        if !index.is_unsigned_integer() {
            self.push_error(
                String::from("Syntax error"),
                format!(
                    "Expected unsigned integer type (u8, u16, u32, u64), not {}. ",
                    index.get_basic_type(),
                ),
            );
        }

        self.consume(
            TokenKind::RBracket,
            String::from("Syntax error"),
            String::from("Expected ']'."),
        )?;

        Ok(Instruction::GEP {
            name,
            index: Box::new(index),
        })
    }

    fn build_function_call(
        &mut self,
        name: &'instr str,
        location: (usize, (usize, usize)),
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut args_provided: Vec<Instruction> = Vec::with_capacity(10);

        let object: FoundObjectId = self.parser_objects.get_object_id(name, location)?;

        let function_id: &str = object.expected_function(location)?;

        let function: Function = self
            .parser_objects
            .get_function_by_id((location.0, location.1), function_id)?;

        let function_type: Instruction = function.0;
        let ignore_extra_args: bool = function.2;

        let maximun_function_arguments: usize = function.1.len();

        while self.peek().kind != TokenKind::RParen {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            let instruction: Instruction = self.expr()?;

            self.throw_if_is_structure_initializer(&instruction);

            args_provided.push(instruction);
        }

        let amount_arguments_provided: usize = args_provided.len();

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        if args_provided.len() > maximun_function_arguments && !ignore_extra_args {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                format!(
                    "Expected '{}' arguments, not '{}'.",
                    maximun_function_arguments,
                    args_provided.len()
                ),
                location.0,
                Some(location.1),
            ));
        }

        if amount_arguments_provided != function.1.len() && !ignore_extra_args {
            let display_args_types: String = if !args_provided.is_empty() {
                args_provided
                    .iter()
                    .map(|parameter| parameter.get_basic_type().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                String::from("none")
            };

            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                format!(
                    "Function expected all arguments with types ({}), not ({}).",
                    function
                        .1
                        .iter()
                        .map(|param| param.get_basic_type().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    display_args_types,
                ),
                location.0,
                Some(location.1),
            ));
        }

        if !ignore_extra_args {
            for (position, argument) in args_provided.iter().enumerate() {
                let arg_type: Instruction = argument.get_type();
                let target_type: Instruction = function.1[position].get_type();

                self.check_type_mismatch(target_type, arg_type, Some(argument));
            }
        }

        Ok(Instruction::Call {
            name,
            args: args_provided,
            kind: Box::new(function_type),
        })
    }

    /* ######################################################################


        PARSER - FUNCTIONS & STRUCTS DECLARATION


    ########################################################################*/

    fn declare(&mut self) {
        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_struct_keyword())
            .for_each(|(pos, _)| {
                let _ = self.declare_struct(pos);
                self.current = 0;
            });

        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_function_keyword())
            .for_each(|(pos, _)| {
                let _ = self.declare_function(pos);
                self.current = 0;
            });
    }

    fn declare_function(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_function(true)?;

        Ok(())
    }

    fn declare_struct(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_struct(true)?;

        Ok(())
    }

    /* ######################################################################


        PARSER - HELPERS


    ########################################################################*/

    fn negate_numeric_type(&self, from: &Instruction) -> Instruction<'instr> {
        if let Instruction::ComplexType(tp, _) = from {
            return match tp {
                Type::U64 => Instruction::ComplexType(Type::S64, ""),
                Type::U32 => Instruction::ComplexType(Type::S32, ""),
                Type::U16 => Instruction::ComplexType(Type::S16, ""),
                Type::U8 => Instruction::ComplexType(Type::S8, ""),
                _ => Instruction::ComplexType(*from.get_basic_type(), ""),
            };
        }

        Instruction::ComplexType(*from.get_basic_type(), "")
    }

    fn throw_if_is_unreacheable_code(&mut self) {
        if self.in_unreacheable_code == self.scope_position && self.scope_position != 0 {
            self.push_error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
            );
        }
    }

    fn throw_if_not_inside_a_loop(&mut self) {
        if !self.inside_a_loop {
            self.push_error(
                String::from("Syntax error"),
                String::from("The flow changer of a loop must go inside one."),
            )
        }
    }

    fn throw_if_is_structure_initializer(&mut self, instruction: &Instruction) {
        if matches!(instruction, Instruction::InitStruct { .. }) {
            self.push_error(
                String::from("Syntax error"),
                String::from("A structure initializer should be stored a variable."),
            );
        }
    }

    fn check_type_mismatch(
        &mut self,
        target_type: Instruction,
        from_type: Instruction,
        expression: Option<&Instruction>,
    ) {
        if expression.is_some_and(|expression| expression.is_binary() || expression.is_group()) {
            if let Err(error) = type_checking::check_types(
                target_type.clone(),
                Instruction::Null,
                expression,
                None,
                ThrushCompilerError::Error(
                    String::from("Mismatched types"),
                    format!(
                        "Expected '{}' but found '{}'.",
                        target_type.get_basic_type(),
                        from_type.get_basic_type()
                    ),
                    self.previous().line,
                    Some(self.previous().span),
                ),
            ) {
                self.errors.push(error);
            }
        } else if let Err(error) = type_checking::check_types(
            target_type.clone(),
            from_type.clone(),
            None,
            None,
            ThrushCompilerError::Error(
                String::from("Mismatched types"),
                format!(
                    "Expected '{}' but found '{}'.",
                    target_type.get_basic_type(),
                    from_type.get_basic_type()
                ),
                self.previous().line,
                Some(self.previous().span),
            ),
        ) {
            self.errors.push(error);
        }
    }

    fn consume(
        &mut self,
        kind: TokenKind,
        error_title: String,
        help: String,
    ) -> Result<&'instr Token<'instr>, ThrushCompilerError> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushCompilerError::Error(
            error_title,
            help,
            self.previous().line,
            Some(self.previous().span),
        ))
    }

    fn push_error(&mut self, error_title: String, help: String) {
        self.errors.push(ThrushCompilerError::Error(
            error_title,
            help,
            self.previous().line,
            Some(self.previous().span),
        ));
    }

    fn match_token(&mut self, kind: TokenKind) -> Result<bool, ThrushCompilerError> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn only_advance(&mut self) -> Result<(), ThrushCompilerError> {
        if !self.end() {
            self.current += 1;
            return Ok(());
        }

        Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            self.peek().line,
            Some(self.peek().span),
        ))
    }

    fn advance(&mut self) -> Result<&'instr Token<'instr>, ThrushCompilerError> {
        if !self.end() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            self.peek().line,
            None,
        ))
    }

    #[inline]
    fn sync(&mut self) {
        self.inside_a_function = false;
        self.inside_a_loop = false;

        while !self.end() {
            match self.peek().kind {
                kind if kind.is_keyword() => return,
                _ => {}
            }

            self.current += 1;
        }
    }

    #[must_use]
    #[inline(always)]
    fn check_type(&self, other_type: TokenKind) -> bool {
        if self.end() {
            return false;
        }

        self.peek().kind == other_type
    }

    #[must_use]
    #[inline(always)]
    fn end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    #[must_use]
    #[inline(always)]
    fn peek(&self) -> &'instr Token<'instr> {
        &self.tokens[self.current]
    }

    #[must_use]
    #[inline(always)]
    fn previous(&self) -> &'instr Token<'instr> {
        &self.tokens[self.current - 1]
    }
}
