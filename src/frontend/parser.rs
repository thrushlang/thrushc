use {
    super::{
        super::{
            backend::{
                compiler::{
                    misc::CompilerFile, traits::AttributesExtensions, types::CompilerAttributes,
                },
                instruction::{Attribute, Instruction},
            },
            constants::MINIMAL_ERROR_CAPACITY,
            diagnostic::Diagnostic,
            error::ThrushCompilerError,
            logging::LogType,
        },
        lexer::{Token, TokenKind, Type},
        objects::{FoundObject, Function, Functions, Local, ParserObjects, Struct},
        scoper::ThrushScoper,
        traits::{FoundObjectEither, StructureBasics, TokenLexemeBasics},
        type_checking,
        types::StructFields,
    },
    ahash::AHashMap as HashMap,
    std::{mem, process},
};

const MINIMAL_STATEMENT_CAPACITY: usize = 100_000;
const MINIMAL_GLOBAL_CAPACITY: usize = 2024;

pub struct Parser<'instr> {
    stmts: Vec<Instruction<'instr>>,
    tokens: &'instr [Token<'instr>],
    errors: Vec<ThrushCompilerError>,
    inside_a_function: bool,
    inside_a_loop: bool,
    at_typed_function: (Type, String),
    at_typed_variable: Type,
    at_unreacheable_code_scope: usize,
    current: usize,
    scope: usize,
    has_entry_point: bool,
    scoper: ThrushScoper<'instr>,
    diagnostic: Diagnostic,
    parser_objects: ParserObjects<'instr>,
}

impl<'instr> Parser<'instr> {
    pub fn new(tokens: &'instr Vec<Token<'instr>>, file: &'instr CompilerFile) -> Self {
        let mut functions: Functions = HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY);

        functions.insert(
            "sizeof!",
            (
                Type::S64,
                Vec::from([Type::Generic]),
                Vec::new(),
                String::new(),
                false,
            ),
        );

        functions.insert(
            "is_signed!",
            (
                Type::Bool,
                Vec::from([Type::Generic]),
                Vec::new(),
                String::new(),
                false,
            ),
        );

        Self {
            stmts: Vec::with_capacity(MINIMAL_STATEMENT_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            tokens,
            current: 0,
            inside_a_function: false,
            inside_a_loop: false,
            at_typed_function: (Type::Void, String::new()),
            at_typed_variable: Type::Void,
            at_unreacheable_code_scope: 0,
            scope: 0,
            has_entry_point: false,
            scoper: ThrushScoper::new(file),
            diagnostic: Diagnostic::new(file),
            parser_objects: ParserObjects::with_functions(functions),
        }
    }

    pub fn start(&mut self) -> &[Instruction<'instr>] {
        self.predefine();

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
                self.diagnostic.report(error, LogType::Error);
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
            TokenKind::Local => Ok(self.build_local_variable(false)?),
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
            self.errors.push(ThrushCompilerError::Error(
                String::from("Duplicated entrypoint"),
                String::from("The language not support two entrypoints."),
                self.previous().line,
                Some(self.previous().span),
            ));
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
        let line: usize = self.advance()?.line;

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        self.inside_a_loop = true;

        let block: Instruction = self.build_code_block(&mut [])?;

        if !block.has_break() && !block.has_return() && !block.has_continue() {
            self.at_unreacheable_code_scope = self.scope;
        }

        self.inside_a_loop = false;

        Ok(Instruction::Loop {
            block: Box::new(block),
        })
    }

    fn build_while_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = self.advance()?.line;

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        let conditional: Instruction = self.expression()?;

        self.check_type_mismatch(Type::Bool, conditional.get_data_type(), Some(&conditional));

        let block: Instruction = self.build_code_block(&mut [])?;

        Ok(Instruction::WhileLoop {
            cond: Box::new(conditional),
            block: Box::new(block),
        })
    }

    fn build_continue(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = self.advance()?.line;

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        self.at_unreacheable_code_scope = self.scope;

        self.throw_if_not_inside_a_loop()?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Continue)
    }

    fn build_break(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = self.advance()?.line;

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        self.at_unreacheable_code_scope = self.scope;

        self.throw_if_not_inside_a_loop()?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Break)
    }

    fn build_match(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        let mut if_cond: Instruction = self.expression()?;
        let mut if_block: Option<Instruction> = None;

        let mut patterns: Vec<Instruction> = Vec::with_capacity(10);
        let mut patterns_stmts: Vec<Instruction> = Vec::with_capacity(10);

        let mut index: u32 = 0;

        while self.match_token(TokenKind::Pattern)? {
            self.scope += 1;
            self.parser_objects.begin_local_scope();

            let pattern: Instruction = self.expression()?;

            self.check_type_mismatch(Type::Bool, pattern.get_data_type(), Some(&pattern));

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

            if index == 0 {
                if_cond = pattern;
                if_block = Some(Instruction::Block {
                    stmts: patterns_stmts.clone(),
                });
            } else {
                patterns.push(Instruction::Elif {
                    cond: Box::new(pattern),
                    block: Box::new(Instruction::Block {
                        stmts: patterns_stmts.clone(),
                    }),
                });
            }

            self.scope -= 1;
            self.parser_objects.end_local_scope();

            patterns_stmts.clear();

            index += 1;
        }

        if if_block.is_none() || !if_cond.get_data_type().is_bool_type() {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Expected at least one pattern."),
                self.previous().line,
                Some(self.previous().span),
            ));
        }

        let otherwise: Option<Box<Instruction>> = if self.match_token(TokenKind::Else)? {
            self.consume(
                TokenKind::ColonColon,
                String::from("Syntax error"),
                String::from("Expected '::'."),
            )?;

            let mut stmts: Vec<Instruction> = Vec::with_capacity(100);

            while !self.match_token(TokenKind::Break)? {
                stmts.push(self.parse()?);
            }

            Some(Box::new(Instruction::Else {
                block: Box::new(Instruction::Block { stmts }),
            }))
        } else {
            None
        };

        Ok(Instruction::If {
            cond: Box::new(if_cond),
            block: Box::new(if_block.unwrap()),
            elfs: patterns,
            otherwise,
        })
    }

    fn build_if_elif_else(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let if_keyword: &Token = self.advance()?;
        let line: usize = if_keyword.line;
        let span: (usize, usize) = if_keyword.span;

        if !self.inside_a_function {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("The if-elif-else must be placed inside a function."),
                line,
                Some(span),
            ));
        }

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        let if_condition: Instruction = self.expression()?;

        if !if_condition.get_data_type().is_bool_type() {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Condition must be type boolean."),
                line,
                Some(span),
            ));
        }

        let if_body: Box<Instruction> = Box::new(self.build_code_block(&mut [])?);

        let mut elfs: Vec<Instruction> = Vec::with_capacity(10);

        while self.match_token(TokenKind::Elif)? {
            let elif_condition: Instruction = self.expression()?;

            self.check_type_mismatch(
                Type::Bool,
                elif_condition.get_data_type(),
                Some(&elif_condition),
            );

            let elif_body: Instruction = self.build_code_block(&mut [])?;

            if !elif_body.has_more_than_a_statement() {
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

            if else_body.has_more_than_a_statement() {
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

    fn build_struct(
        &mut self,
        predefine: bool,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
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

        let mut fields_types: Vec<(&str, Type, u32)> = Vec::with_capacity(10);
        let mut field_position: u32 = 0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let line: usize = self.previous().line;
                let span: (usize, usize) = self.previous().span;

                match &self.peek().kind {
                    TokenKind::DataType(kind) => fields_types.push(("", *kind, field_position)),
                    ident
                        if *ident == TokenKind::Identifier && self.peak_structure_type()
                            || struct_name == self.peek().lexeme.to_str() =>
                    {
                        fields_types.push((
                            self.peek().lexeme.to_str(),
                            Type::Struct,
                            field_position,
                        ))
                    }
                    what => {
                        return Err(ThrushCompilerError::Error(
                            String::from("Undeterminated type"),
                            format!("Type '{}' not exist.", what),
                            line,
                            Some(span),
                        ));
                    }
                };

                self.only_advance()?;

                field_position += 1;

                self.consume(
                    TokenKind::SemiColon,
                    String::from("Syntax error"),
                    String::from("Expected ';'."),
                )?;

                continue;
            }

            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Expected identifier in structure field."),
                self.peek().line,
                Some(self.peek().span),
            ));
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

        if predefine {
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

        let struct_name: &str = name.lexeme.to_str();

        let line: usize = name.line;
        let span: (usize, usize) = name.span;

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        let struct_found: Struct = self
            .parser_objects
            .get_struct(name.lexeme.to_str(), (line, span))?;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut fields: StructFields = Vec::with_capacity(10);
        let mut field_index: u32 = 0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_name: &str = self.previous().lexeme.to_str();
                let span: (usize, usize) = self.previous().span;

                let structure_fields_amount: usize = struct_found.len();

                if field_index as usize >= structure_fields_amount {
                    return Err(ThrushCompilerError::Error(
                        String::from("Too many fields in structure"),
                        format!(
                            "Expected '{}' fields, not '{}'.",
                            structure_fields_amount, field_index
                        ),
                        line,
                        Some(span),
                    ));
                }

                if !struct_found.contains_field(field_name) {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Expected existing field name."),
                        line,
                        Some(span),
                    ));
                }

                let line: usize = self.previous().line;

                let instruction: Instruction = self.expression()?;

                self.throw_if_is_struct_initializer(&instruction, line)?;

                let field_type: Type = instruction.get_data_type();
                let target_type: Type = struct_found.get_field_type(field_name);

                self.check_type_mismatch(target_type, field_type, Some(&instruction));

                fields.push((field_name, instruction, target_type, field_index));

                field_index += 1;
                continue;
            }

            self.only_advance()?;
        }

        let fields_size: usize = fields.len();
        let fields_needed_size: usize = struct_found.len();

        if fields_size != fields_needed_size {
            return Err(ThrushCompilerError::Error(
                String::from("Missing fields"),
                format!(
                    "Expected '{}' fields, but '{}' was gived.",
                    fields_needed_size, fields_size
                ),
                line,
                None,
            ));
        }

        self.consume(
            TokenKind::RBrace,
            String::from("Syntax error"),
            String::from("Expected '}'."),
        )?;

        Ok(Instruction::InitStruct {
            name: struct_name,
            fields,
            kind: Type::Struct,
        })
    }

    fn build_for_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = self.advance()?.line;

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        let variable: Instruction = self.build_local_variable(false)?;
        let conditional: Instruction = self.expression()?;

        self.check_type_mismatch(Type::Bool, conditional.get_data_type(), None);

        let actions: Instruction = self.expression()?;

        let mut variable_clone: Instruction = variable.clone();

        if let Instruction::Local {
            exist_only_comptime,
            ..
        } = &mut variable_clone
        {
            *exist_only_comptime = true;
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

    fn build_local_variable(
        &mut self,
        exist_only_comptime: bool,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = self.advance()?.line;

        if self.scope == 0 {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Locals variables should be contained at local scope."),
                line,
                Some(self.previous().span),
            ));
        }

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name."),
        )?;

        let line: usize = name.line;
        let span: (usize, usize) = name.span;

        self.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let kind: (Type, String) = match &self.peek().kind {
            TokenKind::DataType(kind) => (*kind, String::new()),
            ident if *ident == TokenKind::Identifier && self.peak_structure_type() => {
                (Type::Struct, self.peek().lexeme.to_string())
            }
            _ => {
                return Err(ThrushCompilerError::Error(
                    String::from("Undeterminated type"),
                    format!("Type '{}' not exist.", self.peek().lexeme.to_str()),
                    line,
                    Some(self.peek().span),
                ));
            }
        };

        self.throw_if_is_generic_type(kind.0, line, span)?;
        self.only_advance()?;

        if self.match_token(TokenKind::SemiColon)? {
            self.parser_objects.insert_new_local(
                self.scope,
                name.lexeme.to_str(),
                (kind.0, kind.1, false, true),
                line,
                span,
                &mut self.errors,
            );

            return Ok(Instruction::Local {
                name: name.lexeme.to_str(),
                kind: kind.0,
                value: Box::new(Instruction::Null),
                line,
                exist_only_comptime,
            });
        }

        self.consume(
            TokenKind::Eq,
            String::from("Syntax error"),
            String::from("Expected '='."),
        )?;

        self.at_typed_variable = kind.0;

        let value: Instruction = self.expression()?;

        self.check_type_mismatch(kind.0, value.get_data_type(), Some(&value));
        self.check_struct_type_mismatch(&kind.1, &value, line)?;

        self.parser_objects.insert_new_local(
            self.scope,
            name.lexeme.to_str(),
            (kind.0, kind.1, false, false),
            line,
            span,
            &mut self.errors,
        );

        let local_variable: Instruction = Instruction::Local {
            name: name.lexeme.to_str(),
            kind: kind.0,
            value: Box::new(value),
            line,
            exist_only_comptime,
        };

        Ok(local_variable)
    }

    fn build_return(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = self.advance()?.line;

        if !self.inside_a_function {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Return statement outside of function body."),
                line,
                Some(self.previous().span),
            ));
        }

        if self.at_unreacheable_code_scope == self.scope {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
                line,
                Some(self.previous().span),
            ));
        }

        if self.match_token(TokenKind::SemiColon)? {
            if self.at_typed_function.0.is_void_type() {
                return Ok(Instruction::Null);
            }

            self.check_type_mismatch(Type::Void, self.at_typed_function.0, None);

            return Ok(Instruction::Return(Box::new(Instruction::Null), Type::Void));
        }

        let value: Instruction = self.expression()?;

        self.check_type_mismatch(
            self.at_typed_function.0,
            value.get_data_type(),
            Some(&value),
        );

        self.check_struct_type_mismatch(&self.at_typed_function.1, &value, line)?;

        Ok(Instruction::Return(
            Box::new(value),
            self.at_typed_function.0,
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

        self.scope += 1;
        self.parser_objects.begin_local_scope();

        let mut stmts: Vec<Instruction> = Vec::with_capacity(255);
        let mut was_emited_deallocators: bool = false;

        with_instrs.iter_mut().for_each(|instruction| {
            if let Instruction::FunctionParameter {
                name,
                kind,
                struct_type,
                line,
                span,
                ..
            } = instruction
            {
                self.parser_objects.insert_new_local(
                    self.scope,
                    name,
                    (*kind, struct_type.clone(), false, false),
                    *line,
                    *span,
                    &mut self.errors,
                );
            }

            stmts.push(mem::take(instruction));
        });

        while !self.match_token(TokenKind::RBrace)? {
            let instr: Instruction = self.parse()?;

            if instr.is_return() {
                if let Some(name) = instr.return_with_heaped_ptr() {
                    self.parser_objects
                        .modify_local_deallocation(self.scope, name, true);
                }

                let deallocators: Vec<Instruction> =
                    self.parser_objects.create_deallocators(self.scope);

                stmts.extend(deallocators);

                was_emited_deallocators = true;
            }

            stmts.push(instr)
        }

        if !was_emited_deallocators {
            stmts.extend(self.parser_objects.create_deallocators(self.scope));
        }

        self.parser_objects.end_local_scope();

        self.scoper.add_scope(stmts.clone());
        self.scope -= 1;

        Ok(Instruction::Block { stmts })
    }

    fn build_function(
        &mut self,
        predefine: bool,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        if self.scope != 0 {
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
            if predefine {
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
        let mut parameters_types: Vec<Type> = Vec::with_capacity(10);
        let mut struct_types_params: Vec<(String, usize)> = Vec::with_capacity(10);

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

            let parameter_type: (Type, String) = match self.peek().kind {
                TokenKind::DataType(kind) => (kind, String::new()),
                ident if ident == TokenKind::Identifier && self.peak_structure_type() => {
                    struct_types_params
                        .push((self.peek().lexeme.to_string(), parameter_position as usize));

                    (Type::Struct, self.peek().lexeme.to_string())
                }
                what => {
                    return Err(ThrushCompilerError::Error(
                        String::from("Undeterminated type"),
                        format!("Type '{}' not exist.", what),
                        self.peek().line,
                        Some(self.peek().span),
                    ));
                }
            };

            self.only_advance()?;

            parameters_types.push(parameter_type.0);

            params.push(Instruction::FunctionParameter {
                name: parameter_name,
                kind: parameter_type.0,
                struct_type: parameter_type.1,
                position: parameter_position,
                line: parameter_line,
                span: parameter_span,
            });

            parameter_position += 1;
        }

        let return_type: (Type, String) = match &self.peek().kind {
            TokenKind::DataType(kind) => (*kind, String::new()),
            ident if *ident == TokenKind::Identifier && self.peak_structure_type() => {
                (Type::Struct, self.peek().lexeme.to_string())
            }
            what => {
                return Err(ThrushCompilerError::Error(
                    String::from("Undeterminated type"),
                    format!("Type '{}' not exist.", what),
                    self.peek().line,
                    Some(self.peek().span),
                ));
            }
        };

        self.only_advance()?;

        let function_attributes: CompilerAttributes = self.build_compiler_attributes()?;

        let function_has_ffi: bool = function_attributes.contain_ffi_attribute();
        let function_has_ignore: bool = function_attributes.contain_ignore_attribute();

        if function_has_ignore && !function_has_ffi {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from(
                    "The '@ignore' attribute can only be used if the function contains the '@extern' attribute.",
                ),
                self.peek().line,
                Some(self.peek().span),
            ));
        }

        self.at_typed_function = return_type.clone();

        let mut function: Instruction = Instruction::Function {
            name: function_name,
            params: params.clone(),
            body: None,
            return_type: return_type.0,
            attributes: function_attributes,
        };

        if function_has_ffi || predefine {
            if predefine {
                self.parser_objects.insert_new_function(
                    function_name,
                    (
                        return_type.0,
                        parameters_types,
                        struct_types_params,
                        return_type.1,
                        function_has_ignore,
                    ),
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
    ) -> Result<CompilerAttributes<'instr>, ThrushCompilerError> {
        let mut compiler_attributes: CompilerAttributes = Vec::with_capacity(10);

        while !self.check_type(TokenKind::SemiColon) && !self.check_type(TokenKind::LParen) {
            match self.peek().kind {
                TokenKind::Extern => {
                    compiler_attributes.push(Attribute::FFI(self.build_external_attribute()?));
                }
                TokenKind::Public => {
                    compiler_attributes.push(Attribute::Public(true));
                    self.only_advance()?;
                }
                TokenKind::Ignore => {
                    compiler_attributes.push(Attribute::Ignore);
                    self.only_advance()?;
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
            String::from("Expected a string for @extern(\"FFI_NAME\")."),
        )?;

        let ffi_name: &str = name.lexeme.to_str();

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        Ok(ffi_name)
    }

    /* ######################################################################


        PARSER EXPRESSIONS


    ########################################################################*/

    fn expression(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let instr: Instruction = self.or()?;

        if self.check_type(TokenKind::SemiColon) {
            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;
        }

        Ok(instr)
    }

    fn or(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut instr: Instruction = self.and()?;

        while self.match_token(TokenKind::Or)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.and()?;

            type_checking::check_binary_types(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                (self.previous().line, self.previous().span),
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &instr,
                self.previous().line,
                self.previous().span,
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &right,
                self.previous().line,
                self.previous().span,
            )?;

            instr = Instruction::BinaryOp {
                left: Box::new(instr),
                op,
                right: Box::new(right),
                kind: Type::Bool,
            }
        }

        Ok(instr)
    }

    fn and(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut instr: Instruction = self.equality()?;

        while self.match_token(TokenKind::And)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.equality()?;

            type_checking::check_binary_types(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                (self.previous().line, self.previous().span),
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &instr,
                self.previous().line,
                self.previous().span,
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &right,
                self.previous().line,
                self.previous().span,
            )?;

            instr = Instruction::BinaryOp {
                left: Box::new(instr),
                op,
                right: Box::new(right),
                kind: Type::Bool,
            }
        }

        Ok(instr)
    }

    fn equality(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut instr: Instruction = self.comparison()?;

        while self.match_token(TokenKind::BangEq)? || self.match_token(TokenKind::EqEq)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.comparison()?;

            let left_type: Type = instr.get_data_type_recursive();
            let right_type: Type = right.get_data_type_recursive();

            type_checking::check_binary_types(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            instr.is_chained(&right, (self.previous().line, self.previous().span))?;

            self.throw_if_is_decrement_increment_instruction(
                &instr,
                self.previous().line,
                self.previous().span,
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &right,
                self.previous().line,
                self.previous().span,
            )?;

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind: Type::Bool,
            }
        }

        Ok(instr)
    }

    fn comparison(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut instr: Instruction = self.term()?;

        while self.match_token(TokenKind::Greater)?
            || self.match_token(TokenKind::GreaterEq)?
            || self.match_token(TokenKind::Less)?
            || self.match_token(TokenKind::LessEq)?
        {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.term()?;

            let left_type: Type = instr.get_data_type_recursive();
            let right_type: Type = right.get_data_type_recursive();

            type_checking::check_binary_types(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            instr.is_chained(&right, (self.previous().line, self.previous().span))?;

            self.throw_if_is_decrement_increment_instruction(
                &instr,
                self.previous().line,
                self.previous().span,
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &right,
                self.previous().line,
                self.previous().span,
            )?;

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind: Type::Bool,
            };
        }

        Ok(instr)
    }

    fn term(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut instr: Instruction = self.factor()?;

        while self.match_token(TokenKind::Plus)?
            || self.match_token(TokenKind::Minus)?
            || self.match_token(TokenKind::LShift)?
            || self.match_token(TokenKind::RShift)?
        {
            let op: &Token = self.previous();
            let right: Instruction = self.factor()?;

            let left_type: Type = instr.get_data_type_recursive();
            let right_type: Type = right.get_data_type_recursive();

            type_checking::check_binary_types(
                &op.kind,
                &left_type,
                &right_type,
                (op.line, op.span),
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &instr,
                self.previous().line,
                self.previous().span,
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &right,
                self.previous().line,
                self.previous().span,
            )?;

            let kind: Type = left_type.precompute_numeric_type(right_type, self.at_typed_variable);

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op: &op.kind,
                right: Box::from(right),
                kind,
            };
        }

        Ok(instr)
    }

    fn factor(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut instr: Instruction = self.unary()?;

        while self.match_token(TokenKind::Slash)? || self.match_token(TokenKind::Star)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.unary()?;

            let left_type: Type = instr.get_data_type_recursive();
            let right_type: Type = right.get_data_type_recursive();

            type_checking::check_binary_types(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &instr,
                self.previous().line,
                self.previous().span,
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &right,
                self.previous().line,
                self.previous().span,
            )?;

            let kind: Type = left_type.precompute_numeric_type(right_type, self.at_typed_variable);

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind,
            };
        }

        Ok(instr)
    }

    fn unary(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        if self.match_token(TokenKind::Bang)? {
            let op: &TokenKind = &self.previous().kind;
            let value: Instruction = self.primary()?;

            type_checking::check_unary_types(
                op,
                &value.get_data_type(),
                (self.previous().line, self.previous().span),
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &value,
                self.previous().line,
                self.previous().span,
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                value: Box::from(value),
                kind: Type::Bool,
                is_pre: false,
            });
        } else if self.match_token(TokenKind::Minus)? {
            let op: &TokenKind = &self.previous().kind;
            let mut value: Instruction = self.primary()?;

            if let Instruction::Integer(type_, _, is_signed) = &mut value {
                if op.is_minus_operator() {
                    *type_ = type_.reverse_signed_integer_type();
                    *is_signed = true;
                }
            }

            if let Instruction::Float(_, _, is_signed) = &mut value {
                if op.is_minus_operator() {
                    *is_signed = true;
                }
            }

            if let Instruction::LocalRef { kind: type_, .. } = &mut value {
                if type_.is_integer_type() && op.is_minus_operator() {
                    *type_ = type_.reverse_signed_integer_type();
                }
            }

            let value_type: Type = value.get_data_type();

            type_checking::check_unary_types(
                op,
                &value_type,
                (self.previous().line, self.previous().span),
            )?;

            self.throw_if_is_decrement_increment_instruction(
                &value,
                self.previous().line,
                self.previous().span,
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                value: Box::from(value),
                kind: value_type,
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

                let value: Instruction = self.expression()?;

                if !value.is_local_reference() {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Only local references can be pre-incremented."),
                        self.previous().line,
                        Some(self.previous().span),
                    ));
                }

                let expr: Instruction = Instruction::UnaryOp {
                    op,
                    value: Box::from(value),
                    kind: Type::S64,
                    is_pre: true,
                };

                return Ok(expr);
            }
            TokenKind::MinusMinus => {
                let op: &TokenKind = &self.advance()?.kind;

                let value: Instruction = self.expression()?;

                if !value.is_local_reference() {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Only local references can be pre-decremented."),
                        self.previous().line,
                        Some(self.previous().span),
                    ));
                }

                let expr: Instruction = Instruction::UnaryOp {
                    op,
                    value: Box::from(value),
                    kind: Type::S64,
                    is_pre: true,
                };

                return Ok(expr);
            }
            TokenKind::DataType(dt) => {
                let datatype: &Token = self.advance()?;

                let line: usize = datatype.line;
                let span: (usize, usize) = datatype.span;

                match dt {
                    dt if dt.is_integer_type() => Instruction::Type(*dt),
                    dt if dt.is_float_type() => Instruction::Type(*dt),
                    dt if dt.is_bool_type() => Instruction::Type(*dt),
                    dt if dt == &Type::Ptr => Instruction::Type(*dt),
                    what_heck_dt => {
                        return Err(ThrushCompilerError::Error(
                            String::from("Syntax error"),
                            format!("The type '{}' cannot be a value.", what_heck_dt),
                            line,
                            Some(span),
                        ));
                    }
                }
            }
            TokenKind::LParen => {
                let lparen: &Token = self.advance()?;

                let instr: Instruction = self.expression()?;
                let kind: Type = instr.get_data_type();

                if !instr.is_binary() && !instr.is_group() {
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
                    instr: Box::new(instr),
                    kind,
                });
            }
            TokenKind::NullPtr => {
                self.only_advance()?;
                Instruction::NullPtr
            }
            TokenKind::Str => {
                let token: &Token = self.advance()?;
                Instruction::Str(token.lexeme.parse_scapes(token.line, token.span)?)
            }
            TokenKind::Char => {
                let char: &Token = self.advance()?;
                Instruction::Char(char.lexeme[0])
            }
            kind => match kind {
                TokenKind::Integer(kind, num, is_signed) => {
                    self.only_advance()?;
                    Instruction::Integer(*kind, *num, *is_signed)
                }
                TokenKind::Float(kind, num, is_signed) => {
                    self.only_advance()?;
                    Instruction::Float(*kind, *num, *is_signed)
                }
                TokenKind::Identifier => {
                    let object_token: &Token = self.advance()?;

                    let object_name: &str = object_token.lexeme.to_str();
                    let object_type: TokenKind = object_token.kind;
                    let object_span: (usize, usize) = object_token.span;
                    let object_line: usize = object_token.line;

                    if self.at_unreacheable_code_scope == self.scope {
                        self.errors.push(ThrushCompilerError::Error(
                            String::from("Syntax error"),
                            String::from("Unreacheable code."),
                            object_line,
                            Some(object_span),
                        ));
                    }

                    if self.match_token(TokenKind::Eq)? {
                        let object: FoundObject = self
                            .parser_objects
                            .get_object(object_name, (object_line, object_span))?;

                        let local: &Local = object.expected_local(object_line, object_span)?;
                        let local_type: Type = local.0;

                        let expr: Instruction = self.expression()?;

                        self.check_type_mismatch(local_type, expr.get_data_type(), Some(&expr));

                        return Ok(Instruction::LocalMut {
                            name: object_name,
                            value: Box::new(expr),
                            kind: local_type,
                        });
                    } else if self.match_token(TokenKind::LParen)? {
                        return self.build_function_call(object_name, (object_line, object_span));
                    } else {
                        let object: FoundObject = self
                            .parser_objects
                            .get_object(object_name, (object_line, object_span))?;

                        let local: &Local = object.expected_local(object_line, object_span)?;
                        let local_type: Type = local.0;

                        if local.3 {
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
                            kind: local_type,
                            struct_type: local.1.clone(),
                        };

                        if self.match_token(TokenKind::PlusPlus)?
                            | self.match_token(TokenKind::MinusMinus)?
                        {
                            let op: &TokenKind = &self.previous().kind;

                            type_checking::check_unary_types(
                                &object_type,
                                &local_type,
                                (object_line, object_span),
                            )?;

                            let expr: Instruction = Instruction::UnaryOp {
                                op,
                                value: Box::from(localref),
                                kind: Type::S64,
                                is_pre: false,
                            };

                            return Ok(expr);
                        }

                        localref
                    }
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

    fn build_function_call(
        &mut self,
        name: &'instr str,
        location: (usize, (usize, usize)),
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut arguments_provided: Vec<Instruction> = Vec::with_capacity(10);

        let object: FoundObject = self.parser_objects.get_object(name, location)?;
        let function: Function = object.expected_function(location.0, location.1)?.clone();

        let maximun_function_arguments: usize = function.1.len();

        while self.peek().kind != TokenKind::RParen {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            let instruction: Instruction<'instr> = self.expression()?;

            self.throw_if_is_struct_initializer(&instruction, location.0)?;

            arguments_provided.push(instruction);
        }

        let amount_arguments_provided: usize = arguments_provided.len();

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        if arguments_provided.len() > maximun_function_arguments && !function.4 {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                format!(
                    "Expected '{}' arguments, not '{}'.",
                    maximun_function_arguments,
                    arguments_provided.len()
                ),
                location.0,
                Some(location.1),
            ));
        }

        if amount_arguments_provided != function.1.len() && !function.4 {
            let display_args_types: String = if !arguments_provided.is_empty() {
                arguments_provided
                    .iter()
                    .map(|parameter| parameter.get_data_type().to_string())
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
                        .map(|param| param.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    display_args_types,
                ),
                location.0,
                Some(location.1),
            ));
        }

        if !function.4 {
            for (position, argument) in arguments_provided.iter().enumerate() {
                let argument_type: Type = argument.get_data_type();
                let target_type: Type = function.1[position];

                self.check_type_mismatch(target_type, argument_type, Some(argument));

                if target_type.is_struct_type() {
                    let struct_type: &(String, usize) =
                        function.2.iter().find(|obj| obj.1 == position).unwrap();

                    self.check_struct_type_mismatch(&struct_type.0, argument, location.0)?;
                }
            }
        }

        Ok(Instruction::Call {
            name,
            args: arguments_provided,
            kind: function.0,
            struct_type: function.3.clone(),
        })
    }

    /* ######################################################################


        PARSER - FUNCTIONS & STRUCTS PRE-DECLARATION


    ########################################################################*/

    fn predefine(&mut self) {
        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_struct_keyword())
            .for_each(|(pos, _)| {
                let _ = self.predefine_struct(pos);
                self.current = 0;
            });

        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_function_keyword())
            .for_each(|(pos, _)| {
                let _ = self.predefine_function(pos);
                self.current = 0;
            });
    }

    fn predefine_function(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_function(true)?;

        Ok(())
    }

    fn predefine_struct(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_struct(true)?;

        Ok(())
    }

    /* ######################################################################


        PARSER - HELPERS


    ########################################################################*/

    fn throw_if_is_decrement_increment_instruction(
        &self,
        instruction: &Instruction,
        line: usize,
        span: (usize, usize),
    ) -> Result<(), ThrushCompilerError> {
        if let Instruction::UnaryOp {
            op: TokenKind::PlusPlus | TokenKind::MinusMinus,
            ..
        } = instruction
        {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from(
                    "Increment/decrement operators (--x or x++) can only be used outside expressions.",
                ),
                line,
                Some(span),
            ));
        }

        Ok(())
    }

    fn throw_if_not_inside_a_loop(&self) -> Result<(), ThrushCompilerError> {
        if !self.inside_a_loop {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("The flow changer of a loop must go inside one."),
                self.peek().line,
                Some(self.peek().span),
            ));
        }

        Ok(())
    }

    fn throw_if_is_struct_initializer(
        &self,
        instruction: &Instruction,
        line: usize,
    ) -> Result<(), ThrushCompilerError> {
        if matches!(instruction, Instruction::InitStruct { .. }) {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("A structure initializer should be stored a variable."),
                line,
                None,
            ));
        }

        Ok(())
    }

    fn throw_if_is_generic_type(
        &self,
        kind: Type,
        line: usize,
        span: (usize, usize),
    ) -> Result<(), ThrushCompilerError> {
        if matches!(kind, Type::Generic) {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from(
                    "Generic type 'T' is only allowed in function parameters or structure field types.",
                ),
                line,
                Some(span),
            ));
        }

        Ok(())
    }

    fn check_struct_type_mismatch(
        &self,
        target_type: &str,
        value: &Instruction,
        line: usize,
    ) -> Result<(), ThrushCompilerError> {
        if self.at_structure_type() {
            let mut structure_type: &str = "unreacheable type";

            if let Instruction::InitStruct {
                name: struct_name, ..
            } = &value
            {
                structure_type = struct_name;
            }

            if let Instruction::Call { struct_type, .. } = &value {
                structure_type = struct_type;
            }

            if let Instruction::LocalRef { struct_type, .. } = &value {
                structure_type = struct_type;
            }

            if target_type.to_lowercase() != structure_type.to_lowercase() {
                return Err(ThrushCompilerError::Error(
                    String::from("Mismatched types"),
                    format!("Expected '{}' but found '{}'.", target_type, structure_type),
                    line,
                    None,
                ));
            }
        }

        Ok(())
    }

    fn check_type_mismatch(
        &mut self,
        target_type: Type,
        from_type: Type,
        expression: Option<&Instruction>,
    ) {
        if expression.is_some_and(|expression| expression.is_binary() || expression.is_group()) {
            if let Err(error) = type_checking::check_types(
                target_type,
                None,
                expression,
                None,
                ThrushCompilerError::Error(
                    String::from("Mismatched types"),
                    format!("Expected '{}' but found '{}'.", target_type, from_type),
                    self.previous().line,
                    Some(self.previous().span),
                ),
            ) {
                self.errors.push(error);
            }
        } else if let Err(error) = type_checking::check_types(
            target_type,
            Some(from_type),
            None,
            None,
            ThrushCompilerError::Error(
                String::from("Mismatched types"),
                format!("Expected '{}' but found '{}'.", target_type, from_type),
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
                TokenKind::Local | TokenKind::Fn => return,
                _ => {}
            }

            self.current += 1;
        }
    }

    #[inline]
    fn peak_structure_type(&self) -> bool {
        self.parser_objects
            .get_struct(
                self.peek().lexeme.to_str(),
                (self.peek().line, self.peek().span),
            )
            .is_ok()
    }

    #[inline(always)]
    const fn at_structure_type(&self) -> bool {
        self.at_typed_function.0.is_struct_type() || self.at_typed_variable.is_struct_type()
    }

    #[inline(always)]
    fn check_type(&self, other_type: TokenKind) -> bool {
        if self.end() {
            return false;
        }

        self.peek().kind == other_type
    }

    #[inline(always)]
    fn end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    #[inline(always)]
    fn peek(&self) -> &'instr Token<'instr> {
        &self.tokens[self.current]
    }

    #[inline(always)]
    fn previous(&self) -> &'instr Token<'instr> {
        &self.tokens[self.current - 1]
    }
}
