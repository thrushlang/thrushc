use {
    super::{
        super::{
            backend::compiler::{
                attributes::CompilerAttribute, builtins, conventions::CallConvention,
                instruction::Instruction, misc::CompilerFile, traits::AttributesExtensions,
                types::CompilerAttributes,
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
    lazy_static::lazy_static,
    std::{mem, process},
};

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
    in_function_type: (Type, String),
    in_local_type: Type,
    in_unreacheable_code: usize,
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

        builtins::include(&mut functions);

        Self {
            stmts: Vec::with_capacity(MINIMAL_STATEMENT_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            tokens,
            current: 0,
            inside_a_function: false,
            inside_a_loop: false,
            in_function_type: (Type::Void, String::new()),
            in_local_type: Type::Void,
            in_unreacheable_code: 0,
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
            self.push_error(
                String::from("Duplicated entrypoint"),
                String::from("The language not support two entrypoints :>."),
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
            self.in_unreacheable_code = self.scope;
        }

        self.inside_a_loop = false;

        Ok(Instruction::Loop {
            block: Box::new(block),
        })
    }

    fn build_while_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        let conditional: Instruction = self.expression()?;

        self.check_type_mismatch(Type::Bool, *conditional.get_type(), Some(&conditional));

        let block: Instruction = self.build_code_block(&mut [])?;

        Ok(Instruction::WhileLoop {
            cond: Box::new(conditional),
            block: Box::new(block),
        })
    }

    fn build_continue(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        self.in_unreacheable_code = self.scope;

        self.throw_if_not_inside_a_loop();

        self.optional_consume(TokenKind::SemiColon)?;

        Ok(Instruction::Continue)
    }

    fn build_break(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        self.in_unreacheable_code = self.scope;

        self.throw_if_not_inside_a_loop();

        self.optional_consume(TokenKind::SemiColon)?;

        Ok(Instruction::Break)
    }

    fn build_match(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        let mut if_cond: Instruction = self.expression()?;
        let mut if_block: Instruction = Instruction::Block { stmts: Vec::new() };

        let mut patterns: Vec<Instruction> = Vec::with_capacity(10);
        let mut patterns_stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

        let mut index: u32 = 0;

        while self.match_token(TokenKind::Pattern)? {
            self.scope += 1;
            self.parser_objects.begin_local_scope();

            let pattern: Instruction = self.expression()?;

            self.check_type_mismatch(Type::Bool, *pattern.get_type(), Some(&pattern));

            self.consume(
                TokenKind::ColonColon,
                String::from("Syntax error"),
                String::from("Expected '::'."),
            )?;

            while !self.match_token(TokenKind::Break)? {
                patterns_stmts.push(self.parse()?);
            }

            self.optional_consume(TokenKind::SemiColon)?;

            self.scope -= 1;
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

        if if_block.has_more_than_a_statement() {
            self.check_type_mismatch(Type::Bool, *if_cond.get_type(), Some(&if_cond));
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

            self.optional_consume(TokenKind::SemiColon)?;

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

        if !if_block.has_more_than_a_statement() && patterns.is_empty() && otherwise.is_none() {
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

        let if_condition: Instruction = self.expression()?;

        if !if_condition.get_type().is_bool_type() {
            self.push_error(
                String::from("Syntax error"),
                String::from("Condition must be type boolean."),
            );
        }

        let if_body: Box<Instruction> = Box::new(self.build_code_block(&mut [])?);

        let mut elfs: Vec<Instruction> = Vec::with_capacity(10);

        while self.match_token(TokenKind::Elif)? {
            let elif_condition: Instruction = self.expression()?;

            self.check_type_mismatch(
                Type::Bool,
                *elif_condition.get_type(),
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
                        if ident.is_identifier() && self.peak_structure_type()
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

        self.throw_if_is_unreacheable_code();

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

                let structure_fields_amount: usize = struct_found.len();

                if field_index as usize >= structure_fields_amount {
                    self.push_error(
                        String::from("Too many fields in structure"),
                        format!(
                            "Expected '{}' fields, not '{}'.",
                            structure_fields_amount, field_index
                        ),
                    );

                    field_index = (structure_fields_amount - 1) as u32;
                }

                if !struct_found.contains_field(field_name) {
                    self.push_error(
                        String::from("Syntax error"),
                        String::from("Expected existing field name."),
                    );
                }

                let instruction: Instruction = self.expression()?;

                self.throw_if_is_structure_initializer(&instruction);

                let field_type: Type = *instruction.get_type();
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
            self.push_error(
                String::from("Missing fields"),
                format!(
                    "Expected '{}' fields, but '{}' was gived.",
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
            name: struct_name,
            fields,
            kind: Type::Struct,
        })
    }

    fn build_for_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.only_advance()?;

        self.throw_if_is_unreacheable_code();

        let variable: Instruction = self.build_local_variable(false)?;
        let conditional: Instruction = self.expression()?;

        self.check_type_mismatch(Type::Bool, *conditional.get_type(), None);

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
        self.only_advance()?;

        if self.scope == 0 {
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

        let line: usize = name.line;
        let span: (usize, usize) = name.span;

        self.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let kind: (Type, String) = match &self.peek().kind {
            TokenKind::DataType(kind) => (*kind, String::new()),
            ident if ident.is_identifier() && self.peak_structure_type() => {
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

        self.in_local_type = kind.0;

        let value: Instruction = self.expression()?;

        self.check_type_mismatch(kind.0, *value.get_type(), Some(&value));
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
            self.push_error(
                String::from("Syntax error"),
                String::from("Return statement outside of function body."),
            );
        }

        self.throw_if_is_unreacheable_code();

        if self.match_token(TokenKind::SemiColon)? {
            if self.in_function_type.0.is_void_type() {
                return Ok(Instruction::Null);
            }

            self.check_type_mismatch(Type::Void, self.in_function_type.0, None);

            return Ok(Instruction::Return(Box::new(Instruction::Null), Type::Void));
        }

        let value: Instruction = self.expression()?;

        self.check_type_mismatch(self.in_function_type.0, *value.get_type(), Some(&value));

        self.check_struct_type_mismatch(&self.in_function_type.1, &value, line)?;

        Ok(Instruction::Return(
            Box::new(value),
            self.in_function_type.0,
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

        self.scope += 1;
        self.parser_objects.begin_local_scope();

        let mut stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);
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
                ident if ident.is_identifier() && self.peak_structure_type() => {
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
            ident if ident.is_identifier() && self.peak_structure_type() => {
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

        if !return_type.0.is_void_type() && !function_body.has_return() {
            self.push_error(
                String::from("Syntax error"),
                format!("Missing return type with type '{}'.", return_type.0),
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
    ) -> Result<CompilerAttributes<'instr>, ThrushCompilerError> {
        let mut compiler_attributes: CompilerAttributes = Vec::with_capacity(10);

        while !self.check_type(TokenKind::SemiColon) && !self.check_type(TokenKind::LParen) {
            match self.peek().kind {
                TokenKind::Extern => {
                    compiler_attributes
                        .push(CompilerAttribute::FFI(self.build_external_attribute()?));
                }
                TokenKind::Convention => {
                    compiler_attributes.push(CompilerAttribute::Convention(
                        self.build_call_convention_attribute()?,
                    ));
                }
                TokenKind::Public => {
                    compiler_attributes.push(CompilerAttribute::Public(true));
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
        let instr: Instruction = self.or()?;

        self.optional_consume(TokenKind::SemiColon)?;

        Ok(instr)
    }

    fn or(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.and()?;

        while self.match_token(TokenKind::Or)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.and()?;

            type_checking::check_binary_types(
                op,
                expression.get_type(),
                right.get_type(),
                (self.previous().line, self.previous().span),
            )?;

            expression = Instruction::BinaryOp {
                left: Box::new(expression),
                op,
                right: Box::new(right),
                kind: Type::Bool,
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
                expression.get_type(),
                right.get_type(),
                (self.previous().line, self.previous().span),
            )?;

            expression = Instruction::BinaryOp {
                left: Box::new(expression),
                op,
                right: Box::new(right),
                kind: Type::Bool,
            }
        }

        Ok(expression)
    }

    fn equality(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.comparison()?;

        while self.match_token(TokenKind::BangEq)? || self.match_token(TokenKind::EqEq)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.comparison()?;

            let left_type: Type = *expression.get_type();
            let right_type: Type = *right.get_type();

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
                kind: Type::Bool,
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

            let left_type: Type = *expression.get_type();
            let right_type: Type = *right.get_type();

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
                kind: Type::Bool,
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

            let left_type: Type = *expression.get_type();
            let right_type: Type = *right.get_type();

            type_checking::check_binary_types(
                &op.kind,
                &left_type,
                &right_type,
                (op.line, op.span),
            )?;

            let kind: Type = left_type.precompute_numeric_type(right_type);

            expression = Instruction::BinaryOp {
                left: Box::from(expression),
                op: &op.kind,
                right: Box::from(right),
                kind,
            };
        }

        Ok(expression)
    }

    fn factor(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.unary()?;

        while self.match_token(TokenKind::Slash)? || self.match_token(TokenKind::Star)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.unary()?;

            let left_type: Type = *expression.get_type();
            let right_type: Type = *right.get_type();

            type_checking::check_binary_types(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            let kind: Type = left_type.precompute_numeric_type(right_type);

            expression = Instruction::BinaryOp {
                left: Box::from(expression),
                op,
                right: Box::from(right),
                kind,
            };
        }

        Ok(expression)
    }

    fn unary(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        if self.match_token(TokenKind::Bang)? {
            let op: &TokenKind = &self.previous().kind;
            let value: Instruction = self.primary()?;

            type_checking::check_unary_types(
                op,
                value.get_type(),
                (self.previous().line, self.previous().span),
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

            if let Instruction::Integer(kind, _, is_signed) = &mut value {
                if op.is_minus_operator() {
                    *kind = kind.reverse_to_signed_integer_type();
                    *is_signed = true;
                }
            }

            if let Instruction::Float(_, _, is_signed) = &mut value {
                if op.is_minus_operator() {
                    *is_signed = true;
                }
            }

            if let Instruction::LocalRef { kind, .. } = &mut value {
                if kind.is_integer_type() && op.is_minus_operator() {
                    *kind = kind.reverse_to_signed_integer_type();
                }
            }

            let kind: Type = *value.get_type();

            type_checking::check_unary_types(
                op,
                &kind,
                (self.previous().line, self.previous().span),
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                value: Box::from(value),
                kind,
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

                let expression: Instruction = Instruction::UnaryOp {
                    op,
                    value: Box::from(value),
                    kind: Type::Void,
                    is_pre: true,
                };

                return Ok(expression);
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

                let expression: Instruction = Instruction::UnaryOp {
                    op,
                    value: Box::from(value),
                    kind: Type::Void,
                    is_pre: true,
                };

                return Ok(expression);
            }
            TokenKind::DataType(dt) => {
                let datatype: &Token = self.advance()?;

                let line: usize = datatype.line;
                let span: (usize, usize) = datatype.span;

                match dt {
                    dt if dt.is_integer_type() => Instruction::Type(*dt),
                    dt if dt.is_float_type() => Instruction::Type(*dt),
                    dt if dt.is_bool_type() => Instruction::Type(*dt),
                    dt if dt.is_raw_ptr() => Instruction::Type(*dt),
                    what_heck_dt => {
                        return Err(ThrushCompilerError::Error(
                            String::from("Syntax error"),
                            format!(
                                "The type '{}' cannot be a value during the compilation.",
                                what_heck_dt
                            ),
                            line,
                            Some(span),
                        ));
                    }
                }
            }
            TokenKind::LParen => {
                let lparen: &Token = self.advance()?;

                let instr: Instruction = self.expression()?;
                let kind: Type = *instr.get_type();

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
            TokenKind::NullT => {
                self.only_advance()?;
                Instruction::NullT
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

                    self.throw_if_is_unreacheable_code();

                    if self.match_token(TokenKind::Eq)? {
                        let object: FoundObject = self
                            .parser_objects
                            .get_object(object_name, (object_line, object_span))?;

                        let local: &Local = object.expected_local(object_line, object_span)?;
                        let local_type: Type = local.0;

                        let expr: Instruction = self.expression()?;

                        self.check_type_mismatch(local_type, *expr.get_type(), Some(&expr));

                        return Ok(Instruction::LocalMut {
                            name: object_name,
                            value: Box::new(expr),
                            kind: local_type,
                        });
                    }

                    if self.match_token(TokenKind::LBracket)? {
                        return self.build_gep(object_name, (object_line, object_span));
                    }

                    if self.match_token(TokenKind::LParen)? {
                        return self.build_function_call(object_name, (object_line, object_span));
                    }

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
                            kind: Type::Void,
                            is_pre: false,
                        };

                        return Ok(expr);
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
        let object: FoundObject = self.parser_objects.get_object(name, location)?;
        let local: &Local = object.expected_local(location.0, location.1)?;

        self.check_type_mismatch(Type::T, local.0, None);

        let index: Instruction = self.expression()?;

        if !index.is_unsigned_integer() {
            self.push_error(
                String::from("Syntax error"),
                format!(
                    "Expected unsigned integer type (u8, u16, u32, u64), not {}. ",
                    index.get_type(),
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
        let mut arguments_provided: Vec<Instruction> = Vec::with_capacity(10);

        let object: FoundObject = self.parser_objects.get_object(name, location)?;
        let function: Function = object.expected_function(location.0, location.1)?.clone();

        let maximun_function_arguments: usize = function.1.len();

        while self.peek().kind != TokenKind::RParen {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            let instruction: Instruction = self.expression()?;

            self.throw_if_is_structure_initializer(&instruction);

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
                    .map(|parameter| parameter.get_type().to_string())
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
                let argument_type: Type = *argument.get_type();
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

    fn throw_if_is_unreacheable_code(&mut self) {
        if self.in_unreacheable_code == self.scope && self.scope != 0 {
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

            if target_type != structure_type {
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

    fn push_error(&mut self, error_title: String, help: String) {
        self.errors.push(ThrushCompilerError::Error(
            error_title,
            help,
            self.previous().line,
            Some(self.previous().span),
        ));
    }

    fn optional_consume(&mut self, tokenkind: TokenKind) -> Result<(), ThrushCompilerError> {
        if self.check_type(tokenkind) {
            self.consume(
                tokenkind,
                String::from("Syntax error"),
                format!("Expected '{}'.", tokenkind),
            )?;

            return Ok(());
        }

        Ok(())
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
        self.in_function_type.0.is_struct_type() || self.in_local_type.is_struct_type()
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
