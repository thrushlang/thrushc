use super::super::backend::compiler::types::EnumField;

use super::traits::CustomTypeFieldsExtensions;
use super::utils;
use super::{objects::Constant, traits::EnumExtensions};

use super::{
    lexer::{Token, TokenKind, Type},
    objects::{FoundObjectId, Function, Functions, Local, ParserObjects, Struct},
    scoper::ThrushScoper,
    traits::{
        EnumFieldsExtensions, FoundObjectEither, FoundObjectExtensions, StructureExtensions,
        TokenLexemeExtensions,
    },
    type_checking,
};

use super::super::{
    backend::compiler::{
        attributes::LLVMAttribute,
        builtins,
        conventions::CallConvention,
        instruction::Instruction,
        misc::CompilerFile,
        traits::{AttributesExtensions, ConstructorExtensions, StructFieldsExtensions},
        types::{
            CodeLocation, Constructor, CustomType, CustomTypeFields, EnumFields, StructFields,
            ThrushAttributes,
        },
    },
    common::{
        constants::MINIMAL_ERROR_CAPACITY, diagnostic::Diagnostician, error::ThrushCompilerError,
        logging,
    },
    logging::LoggingType,
};

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;
use std::sync::Arc;
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

    // Type management
    in_function_type: Type,
    in_local_type: Type,
    in_unreacheable_code: usize,

    // Scope control
    current: usize,
    scope_position: usize,

    // Trigger flags
    has_entry_point: bool,
    rec_structure_ref: bool,
    inside_a_function: bool,
    inside_a_loop: bool,

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
            in_function_type: Type::Void,
            in_local_type: Type::Void,
            rec_structure_ref: false,
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

        while !self.is_eof() {
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

        self.scoper.check();

        self.stmts.as_slice()
    }

    fn parse(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        match &self.peek().kind {
            TokenKind::Type => Ok(self.build_custom_type(false)?),
            TokenKind::Struct => Ok(self.build_struct(false)?),
            TokenKind::Enum => Ok(self.build_enum(false)?),
            TokenKind::Fn => Ok(self.build_function(false)?),

            TokenKind::LBrace => Ok(self.build_code_block(&mut [])?),
            TokenKind::Return => Ok(self.build_return()?),
            TokenKind::Const => Ok(self.build_const(false)?),
            TokenKind::Local => Ok(self.build_local(false)?),
            TokenKind::For => Ok(self.build_for_loop()?),
            TokenKind::New => Ok(self.build_constructor()?),
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

    fn build_for_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::For,
            String::from("Syntax error"),
            String::from("Expected 'for'."),
        )?;

        let variable: Instruction = self.build_local(false)?;

        let conditional: Instruction = self.expression()?;

        self.check_type_mismatch(Type::Bool, conditional.get_type().clone(), None);

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

    fn build_loop(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::Loop,
            String::from("Syntax error"),
            String::from("Expected 'loop'."),
        )?;

        self.throw_unreacheable_code();

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
        self.consume(
            TokenKind::While,
            String::from("Syntax error"),
            String::from("Expected 'while'."),
        )?;

        self.throw_unreacheable_code();

        let conditional: Instruction = self.expr()?;

        self.check_type_mismatch(
            Type::Bool,
            conditional.get_type().clone(),
            Some(&conditional),
        );

        let block: Instruction = self.build_code_block(&mut [])?;

        Ok(Instruction::WhileLoop {
            cond: Box::new(conditional),
            block: Box::new(block),
        })
    }

    fn build_continue(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::Continue,
            String::from("Syntax error"),
            String::from("Expected 'continue'."),
        )?;

        self.throw_unreacheable_code();

        self.in_unreacheable_code = self.scope_position;

        self.throw_outside_loop();

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Continue)
    }

    fn build_break(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::Break,
            String::from("Syntax error"),
            String::from("Expected 'break'."),
        )?;

        self.throw_unreacheable_code();

        self.in_unreacheable_code = self.scope_position;

        self.throw_outside_loop();

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Break)
    }

    fn build_match(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::Match,
            String::from("Syntax error"),
            String::from("Expected 'match'."),
        )?;

        self.throw_unreacheable_code();

        let mut start_pattern: Instruction = self.expr()?;
        let mut start_block: Instruction = Instruction::Block { stmts: Vec::new() };

        let mut patterns: Vec<Instruction> = Vec::with_capacity(10);
        let mut patterns_stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

        let mut position: u32 = 0;

        while self.match_token(TokenKind::Pattern)? {
            self.scope_position += 1;
            self.parser_objects.begin_local_scope();

            let pattern: Instruction = self.expr()?;

            self.check_type_mismatch(Type::Bool, pattern.get_type().clone(), Some(&pattern));

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

            if position != 0 {
                patterns.push(Instruction::Elif {
                    cond: Box::new(pattern),
                    block: Box::new(Instruction::Block {
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
            self.check_type_mismatch(
                Type::Bool,
                start_pattern.get_type().clone(),
                Some(&start_pattern),
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

        if !start_block.has_instruction() && patterns.is_empty() && otherwise.is_none() {
            return Ok(Instruction::Null);
        }

        Ok(Instruction::If {
            cond: Box::new(start_pattern),
            block: Box::new(start_block),
            elfs: patterns,
            otherwise,
        })
    }

    fn build_if_elif_else(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::If,
            String::from("Syntax error"),
            String::from("Expected 'if'."),
        )?;

        if !self.inside_a_function {
            self.push_error(
                String::from("Syntax error"),
                String::from("The if-elif-else must be placed inside a function."),
            );
        }

        self.throw_unreacheable_code();

        let if_condition: Instruction = self.expr()?;

        if !if_condition.get_type().is_bool_type() {
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
                Type::Bool,
                elif_condition.get_type().clone(),
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

    fn build_custom_type(
        &mut self,
        declare: bool,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::Type,
            String::from("Syntax error"),
            String::from("Expected 'type'."),
        )?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected type name."),
        )?;

        let custom_type_name: &str = name.lexeme.to_str();

        self.consume(
            TokenKind::Eq,
            String::from("Syntax error"),
            String::from("Expected '='."),
        )?;

        let custom_type_attributes: ThrushAttributes =
            self.build_compiler_attributes(&[TokenKind::LBrace])?;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut custom_type_fields: CustomTypeFields = Vec::with_capacity(10);

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            let kind: Type = self.build_type(Some(TokenKind::SemiColon))?;

            custom_type_fields.push(kind);
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
            self.parser_objects.insert_new_custom_type(
                custom_type_name,
                (custom_type_fields, custom_type_attributes),
                CodeLocation::new(self.diagnostician.get_instance(), name.line, name.span),
            )?;
        }

        Ok(Instruction::Null)
    }

    fn build_enum(&mut self, declare: bool) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::Enum,
            String::from("Syntax error"),
            String::from("Expected 'enum'."),
        )?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected enum name."),
        )?;

        let enum_name: &str = name.lexeme.to_str();

        let enum_attributes: ThrushAttributes =
            self.build_compiler_attributes(&[TokenKind::LBrace])?;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut enum_fields: EnumFields = Vec::with_capacity(10);
        let mut index: f64 = 0.0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let name: &str = self.previous().lexeme.to_str();

                self.consume(
                    TokenKind::Colon,
                    String::from("Syntax error"),
                    String::from("Expected ':'."),
                )?;

                let field_type: Type = self.build_type(None)?;

                if !field_type.is_integer_type()
                    && !field_type.is_float_type()
                    && !field_type.is_bool_type()
                {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Expected integer, boolean or floating-point types."),
                        self.previous().line,
                        Some(self.previous().span),
                    ));
                }

                if self.match_token(TokenKind::SemiColon)? {
                    let field_value: Instruction = if field_type.is_float_type() {
                        Instruction::Float(field_type.clone(), index, false)
                    } else if field_type.is_bool_type() {
                        Instruction::Boolean(Type::Bool, index != 0.0)
                    } else {
                        Instruction::Integer(field_type.clone(), index, false)
                    };

                    enum_fields.push((name, field_value));
                    index += 1.0;

                    continue;
                }

                self.consume(
                    TokenKind::Eq,
                    String::from("Syntax error"),
                    String::from("Expected '='."),
                )?;

                let expression: Instruction = self.expr()?;

                expression.throw_attemping_use_jit(CodeLocation::new(
                    self.diagnostician.get_instance(),
                    self.previous().line,
                    self.previous().span,
                ))?;

                self.consume(
                    TokenKind::SemiColon,
                    String::from("Syntax error"),
                    String::from("Expected ';'."),
                )?;

                self.check_type_mismatch(
                    field_type,
                    expression.get_type().clone(),
                    Some(&expression),
                );

                enum_fields.push((name, expression));

                continue;
            }

            self.only_advance()?;

            self.push_error(
                String::from("Syntax error"),
                String::from("Expected identifier in enum field."),
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
            self.parser_objects.insert_new_enum(
                enum_name,
                (enum_fields, enum_attributes),
                CodeLocation::new(self.diagnostician.get_instance(), name.line, name.span),
            )?;
        }

        Ok(Instruction::Null)
    }

    fn build_struct(&mut self, declare: bool) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::Struct,
            String::from("Syntax error"),
            String::from("Expected 'struct'."),
        )?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected structure name."),
        )?;

        let struct_name: &str = name.lexeme.to_str();

        let struct_attributes: ThrushAttributes =
            self.build_compiler_attributes(&[TokenKind::LBrace])?;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut fields_types: StructFields = Vec::with_capacity(10);
        let mut field_position: u32 = 0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_name: &str = self.previous().lexeme.to_str();

                if self.peek().lexeme.to_str() == struct_name {
                    self.rec_structure_ref = true;
                }

                let field_type: Type = self.build_type(Some(TokenKind::SemiColon))?;

                self.rec_structure_ref = false;

                fields_types.push((field_name, field_type, field_position));
                field_position += 1;

                continue;
            }

            self.only_advance()?;

            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Expected identifier in structure field."),
                self.previous().line,
                Some(self.previous().span),
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

        if declare {
            self.parser_objects.insert_new_struct(
                name.lexeme.to_str(),
                (fields_types, struct_attributes),
                CodeLocation::new(self.diagnostician.get_instance(), name.line, name.span),
            )?;
        }

        Ok(Instruction::Null)
    }

    fn build_constructor(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::New,
            String::from("Syntax error"),
            String::from("Expected 'new'."),
        )?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected structure reference."),
        )?;

        let struct_name: &str = name.lexeme.to_str();

        let line: usize = name.line;
        let span: (usize, usize) = name.span;

        let struct_found: Struct = self.parser_objects.get_struct(
            struct_name,
            CodeLocation::new(self.diagnostician.get_instance(), line, span),
        )?;

        let fields_required: usize = struct_found.get_fields().len();

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

                if !struct_found.contains_field(field_name) {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Expected existing structure field name."),
                        self.previous().line,
                        Some(self.previous().span),
                    ));
                }

                if field_index as usize >= fields_required {
                    return Err(ThrushCompilerError::Error(
                        String::from("Too many fields in structure"),
                        format!(
                            "Expected '{}' fields, not '{}'.",
                            fields_required, field_index
                        ),
                        self.previous().line,
                        Some(self.previous().span),
                    ));
                }

                let expression: Instruction = self.expr()?;

                self.throw_constructor(&expression);

                let expression_type: &Type = expression.get_type();

                if let Some(target_type) = struct_found.get_field_type(field_name) {
                    self.check_type_mismatch(
                        target_type.clone(),
                        expression_type.clone(),
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

        if fields_size != fields_required {
            return Err(ThrushCompilerError::Error(
                String::from("Missing fields in structure"),
                format!(
                    "Expected '{}' arguments, but '{}' was gived.",
                    fields_required, fields_size
                ),
                self.previous().line,
                None,
            ));
        }

        self.consume(
            TokenKind::RBrace,
            String::from("Syntax error"),
            String::from("Expected '}'."),
        )?;

        Ok(Instruction::InitStruct {
            arguments: arguments.clone(),
            kind: arguments.get_type(),
        })
    }

    fn build_const(&mut self, declare: bool) -> Result<Instruction<'instr>, ThrushCompilerError> {
        if !self.is_main_scope() {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Constants should be contained at global scope."),
                self.peek().line,
                None,
            ));
        }

        self.consume(
            TokenKind::Const,
            String::from("Syntax error"),
            String::from("Expected 'const'."),
        )?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name."),
        )?;

        let const_name: &str = name.lexeme.to_str();

        self.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let const_type: Type = self.build_type(None)?;

        let const_attributes: ThrushAttributes =
            self.build_compiler_attributes(&[TokenKind::Eq])?;

        self.consume(
            TokenKind::Eq,
            String::from("Syntax error"),
            String::from("Expected '='."),
        )?;

        let const_value: Instruction = self.expr()?;

        const_value.throw_attemping_use_jit(CodeLocation::new(
            self.diagnostician.get_instance(),
            self.previous().line,
            self.previous().span,
        ))?;

        self.check_type_mismatch(
            const_type.clone(),
            const_value.get_type().clone(),
            Some(&const_value),
        );

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        if declare {
            self.parser_objects.insert_new_constant(
                const_name,
                (const_type, const_attributes),
                CodeLocation::new(self.diagnostician.get_instance(), name.line, name.span),
            )?;

            return Ok(Instruction::Null);
        }

        Ok(Instruction::Const {
            name: const_name,
            kind: const_type,
            value: Box::new(const_value),
            attributes: const_attributes,
        })
    }

    fn build_local(&mut self, comptime: bool) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        if self.is_main_scope() {
            self.push_error(
                String::from("Syntax error"),
                String::from("Locals variables should be contained at local scope."),
            );
        }

        self.consume(
            TokenKind::Local,
            String::from("Syntax error"),
            String::from("Expected 'local'."),
        )?;

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

        let local_type: Type = self.build_type(None)?;

        self.parser_objects.insert_new_local(
            self.scope_position,
            name.lexeme.to_str(),
            (local_type.clone(), false, false),
            CodeLocation::new(self.diagnostician.get_instance(), name.line, name.span),
        )?;

        if self.match_token(TokenKind::SemiColon)? {
            return Ok(Instruction::Local {
                name: name.lexeme.to_str(),
                kind: local_type,
                value: Box::new(Instruction::Null),
                location: CodeLocation::new(
                    self.diagnostician.get_instance(),
                    name.line,
                    name.span,
                ),
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

        self.check_type_mismatch(
            local_type.clone(),
            local_value.get_type().clone(),
            Some(&local_value),
        );

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        let local: Instruction = Instruction::Local {
            name: name.lexeme.to_str(),
            kind: local_type,
            value: Box::new(local_value),
            location: CodeLocation::new(self.diagnostician.get_instance(), name.line, name.span),
            comptime,
        };

        Ok(local)
    }

    fn build_return(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::Return,
            String::from("Syntax error"),
            String::from("Expected 'return'."),
        )?;

        if !self.inside_a_function {
            self.push_error(
                String::from("Syntax error"),
                String::from("Return outside of function body."),
            );
        }

        if self.match_token(TokenKind::SemiColon)? {
            if self.in_function_type.is_void_type() {
                return Ok(Instruction::Null);
            }

            self.check_type_mismatch(Type::Void, self.in_function_type.clone(), None);

            return Ok(Instruction::Return(Type::Void, Box::new(Instruction::Null)));
        }

        let value: Instruction = self.expr()?;

        self.check_type_mismatch(
            self.in_function_type.clone(),
            value.get_type().clone(),
            Some(&value),
        );

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Return(
            self.in_function_type.clone(),
            Box::new(value),
        ))
    }

    fn build_code_block(
        &mut self,
        with_instrs: &mut [Instruction<'instr>],
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        self.scope_position += 1;
        self.parser_objects.begin_local_scope();

        let mut stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

        for instruction in with_instrs.iter_mut() {
            if let Instruction::FunctionParameter {
                name,
                kind,
                location,
                ..
            } = instruction
            {
                self.parser_objects.insert_new_local(
                    self.scope_position,
                    name,
                    ((*kind).clone(), false, false),
                    location.clone(),
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
        self.throw_unreacheable_code();

        if self.scope_position != 0 {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Functions are only defined globally."),
                self.previous().line,
                Some(self.previous().span),
            ));
        }

        self.consume(
            TokenKind::Fn,
            String::from("Syntax error"),
            String::from("Expected 'fn'."),
        )?;

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
        let mut parameters_types: Vec<Type> = Vec::with_capacity(10);

        let mut position: u32 = 0;

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

            let parameter_type: Type = self.build_type(None)?;

            parameters_types.push(parameter_type.clone());

            params.push(Instruction::FunctionParameter {
                name: parameter_name,
                kind: parameter_type,
                position,
                location: CodeLocation::new(
                    self.diagnostician.get_instance(),
                    parameter_line,
                    parameter_span,
                ),
            });

            position += 1;
        }

        let return_type: Type = self.build_type(None)?;

        let function_attributes: ThrushAttributes =
            self.build_compiler_attributes(&[TokenKind::SemiColon, TokenKind::LParen])?;

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
            return_type: return_type.clone(),
            attributes: function_attributes,
        };

        if function_has_ffi || declare {
            if declare {
                self.parser_objects.insert_new_function(
                    function_name,
                    (return_type, parameters_types, function_has_ignore),
                    CodeLocation::new(self.diagnostician.get_instance(), name.line, name.span),
                )?;
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
                format!("Missing return type with type '{}'.", return_type),
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
        limits: &[TokenKind],
    ) -> Result<ThrushAttributes<'instr>, ThrushCompilerError> {
        let mut compiler_attributes: ThrushAttributes = Vec::with_capacity(10);

        while !limits.contains(&self.peek().kind) {
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

        if self.match_token(TokenKind::BangEq)? || self.match_token(TokenKind::EqEq)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.comparison()?;

            type_checking::check_binary_types(
                op,
                expression.get_type(),
                right.get_type(),
                (self.previous().line, self.previous().span),
            )?;

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

        if self.match_token(TokenKind::Greater)?
            || self.match_token(TokenKind::GreaterEq)?
            || self.match_token(TokenKind::Less)?
            || self.match_token(TokenKind::LessEq)?
        {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.term()?;

            type_checking::check_binary_types(
                op,
                expression.get_type(),
                right.get_type(),
                (self.previous().line, self.previous().span),
            )?;

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

            let left_type: &Type = expression.get_type();
            let right_type: &Type = right.get_type();

            type_checking::check_binary_types(&op.kind, left_type, right_type, (op.line, op.span))?;

            let kind: &Type = left_type.precompute_type(right_type);

            expression = Instruction::BinaryOp {
                left: Box::from(expression.clone()),
                op: &op.kind,
                right: Box::from(right),
                kind: kind.clone(),
            };
        }

        Ok(expression)
    }

    fn factor(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.unary()?;

        while self.match_token(TokenKind::Slash)? || self.match_token(TokenKind::Star)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.unary()?;

            let left_type: &Type = expression.get_type();
            let right_type: &Type = right.get_type();

            type_checking::check_binary_types(
                op,
                left_type,
                right_type,
                (self.previous().line, self.previous().span),
            )?;

            let kind: &Type = left_type.precompute_type(right_type);

            expression = Instruction::BinaryOp {
                left: Box::from(expression.clone()),
                op,
                right: Box::from(right),
                kind: kind.clone(),
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
                expression.get_type(),
                (self.previous().line, self.previous().span),
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                expression: Box::from(expression),
                kind: Type::Bool,
                is_pre: false,
            });
        }

        if self.match_token(TokenKind::Minus)? {
            let op: &TokenKind = &self.previous().kind;

            let mut expression: Instruction = self.primary()?;

            expression.cast_signess(*op);

            let expression_type: &Type = expression.get_type();

            type_checking::check_unary_types(
                op,
                expression_type,
                (self.previous().line, self.previous().span),
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                expression: Box::from(expression.clone()),
                kind: expression_type.clone(),
                is_pre: false,
            });
        }

        let instr: Instruction = self.primary()?;

        Ok(instr)
    }

    fn primary(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let primary: Instruction = match &self.peek().kind {
            TokenKind::Take => {
                self.only_advance()?;

                if self.match_token(TokenKind::Identifier)? {
                    let token: &Token = self.previous();
                    let name: &str = token.lexeme.to_str();

                    let location: CodeLocation = CodeLocation::new(
                        self.diagnostician.get_instance(),
                        token.line,
                        token.span,
                    );

                    return self.build_ref(name, location, true);
                }

                return Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("Take the value is only allowed by references."),
                    self.previous().line,
                    Some(self.previous().span),
                ));
            }

            TokenKind::Carry => {
                self.only_advance()?;

                self.consume(
                    TokenKind::LBracket,
                    String::from("Syntax error"),
                    String::from("Expected '['."),
                )?;

                let carry_type: Type = self.build_type(None)?;

                self.consume(
                    TokenKind::RBracket,
                    String::from("Syntax error"),
                    String::from("Expected ']'."),
                )?;

                if self.check(TokenKind::Identifier) {
                    let object: &Token = self.consume(
                        TokenKind::Identifier,
                        String::from("Syntax error"),
                        String::from("Expected 'identifier'."),
                    )?;

                    let name: &str = object.lexeme.to_str();

                    self.build_ref(
                        name,
                        CodeLocation::new(
                            self.diagnostician.get_instance(),
                            object.line,
                            object.span,
                        ),
                        false,
                    )?;

                    return Ok(Instruction::Carry {
                        name,
                        expression: None,
                        carry_type,
                    });
                }

                let expression: Instruction = self.expr()?;
                let expression_type: &Type = expression.get_type();

                if !expression_type.is_ptr_type() && !expression_type.is_address_type() {
                    self.push_error(
                        String::from("Attemping to access an invalid pointer"),
                        format!(
                            "Carry is only allowed for pointer types or memory address, not '{}'. ",
                            expression_type
                        ),
                    );
                }

                Instruction::Carry {
                    name: "",
                    expression: Some(Box::new(expression)),
                    carry_type,
                }
            }

            TokenKind::Write => {
                self.only_advance()?;

                self.consume(
                    TokenKind::LBracket,
                    String::from("Syntax error"),
                    String::from("Expected '['."),
                )?;

                let kind: Type = self.build_type(None)?;

                self.consume(
                    TokenKind::RBracket,
                    String::from("Syntax error"),
                    String::from("Expected ']'."),
                )?;

                let value: Instruction = self.expr()?;

                self.check_type_mismatch(kind.clone(), value.get_type().clone(), Some(&value));

                self.consume(
                    TokenKind::Arrow,
                    String::from("Syntax error"),
                    String::from("Expected '->'."),
                )?;

                if self.check(TokenKind::Identifier) {
                    let object: &Token = self.consume(
                        TokenKind::Identifier,
                        String::from("Syntax error"),
                        String::from("Expected 'identifier'."),
                    )?;

                    let name: &str = object.lexeme.to_str();

                    self.build_ref(
                        name,
                        CodeLocation::new(
                            self.diagnostician.get_instance(),
                            object.line,
                            object.span,
                        ),
                        false,
                    )?;

                    return Ok(Instruction::Write {
                        write_to: (name, None),
                        write_value: Box::new(value),
                        write_type: kind,
                    });
                }

                let expression: Instruction = self.expr()?;
                let expression_type: &Type = expression.get_type();

                if !expression_type.is_ptr_type() && !expression_type.is_address_type() {
                    self.push_error(
                        String::from("Attemping to access an invalid pointer"),
                        format!(
                            "Write is only allowed for pointer types or memory address, not '{}'. ",
                            expression_type
                        ),
                    );
                }

                Instruction::Write {
                    write_to: ("", Some(Box::new(expression))),
                    write_value: Box::new(value),
                    write_type: kind,
                }
            }

            TokenKind::Address => {
                self.only_advance()?;

                let object: &Token = self.consume(
                    TokenKind::Identifier,
                    String::from("Syntax error"),
                    String::from("Expected 'identifier'."),
                )?;

                let name: &str = object.lexeme.to_str();
                let location: CodeLocation =
                    CodeLocation::new(self.diagnostician.get_instance(), object.line, object.span);

                self.consume(
                    TokenKind::LBracket,
                    String::from("Syntax error"),
                    String::from("Expected '['."),
                )?;

                return self.build_address(name, location);
            }

            TokenKind::New => self.build_constructor()?,

            TokenKind::PlusPlus => {
                let op: &TokenKind = &self.advance()?.kind;

                let expression: Instruction = self.expr()?;

                if !expression.is_local_ref() {
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
                    kind: Type::Void,
                    is_pre: true,
                };

                return Ok(unaryop);
            }

            TokenKind::MinusMinus => {
                let op: &TokenKind = &self.advance()?.kind;

                let expression: Instruction = self.expr()?;

                if !expression.is_local_ref() {
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
                    kind: Type::Void,
                    is_pre: true,
                };

                return Ok(unaryop);
            }

            // tk_kind if tk_kind.is_type() => self.build_type()?,
            TokenKind::LParen => {
                let lparen: &Token = self.advance()?;

                let expression: Instruction = self.expression()?;
                let expression_type: &Type = expression.get_type();

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
                    expression: Box::new(expression.clone()),
                    kind: expression_type.clone(),
                });
            }

            TokenKind::Str => {
                let token: &Token = self.advance()?;
                let token_lexeme: &[u8] = token.lexeme;

                Instruction::Str(
                    Type::Str,
                    token_lexeme.parse_scapes(token.line, token.span)?,
                )
            }

            TokenKind::Char => {
                let char: &Token = self.advance()?;

                Instruction::Char(Type::Char, char.lexeme[0])
            }

            kind => match kind {
                TokenKind::Integer => {
                    let integer_tk: &Token = self.advance()?;
                    let integer: &str = integer_tk.lexeme.to_str();

                    let parsed_integer: (Type, f64) = utils::parse_number(
                        integer,
                        CodeLocation::new(
                            self.diagnostician.get_instance(),
                            integer_tk.line,
                            integer_tk.span,
                        ),
                    )?;

                    let integer_type: Type = parsed_integer.0;
                    let integer_value: f64 = parsed_integer.1;

                    Instruction::Integer(integer_type, integer_value, false)
                }

                TokenKind::Float => {
                    let float_tk: &Token = self.advance()?;
                    let float: &str = float_tk.lexeme.to_str();

                    let parsed_float: (Type, f64) = utils::parse_number(
                        float,
                        CodeLocation::new(
                            self.diagnostician.get_instance(),
                            float_tk.line,
                            float_tk.span,
                        ),
                    )?;

                    let float_type: Type = parsed_float.0;
                    let float_value: f64 = parsed_float.1;

                    Instruction::Float(float_type, float_value, false)
                }

                TokenKind::Identifier => {
                    let object_token: &Token = self.advance()?;

                    let name: &str = object_token.lexeme.to_str();

                    let location: CodeLocation = CodeLocation::new(
                        self.diagnostician.get_instance(),
                        object_token.line,
                        object_token.span,
                    );

                    let line: usize = location.line;
                    let span: (usize, usize) = location.span;

                    self.throw_unreacheable_code();

                    /*if self.rec_structure_ref {
                        return Ok(Instruction::ComplexType(
                            Type::Struct(Vec::new()),
                            object_name,
                            None,
                        ));
                    }*/

                    let object: FoundObjectId =
                        self.parser_objects.get_object_id(name, location.clone())?;

                    /*if object.is_structure() {
                        return Ok(Instruction::ComplexType(
                            Type::Struct(Vec::new()),
                            object_name,
                            None,
                        ));
                    }*/

                    /*if object.is_custom_type() {
                        let custom_id: &str = object.expected_custom_type(location)?;

                        let custom: CustomType = self
                            .parser_objects
                            .get_custom_type_by_id(custom_id, location)?;

                        let custom_type_fields: CustomTypeFields = custom.0;

                        return Ok(custom_type_fields.get_type());
                    }*/

                    if self.match_token(TokenKind::Eq)? {
                        let object: FoundObjectId =
                            self.parser_objects.get_object_id(name, location.clone())?;

                        let local_position: (&str, usize) =
                            object.expected_local(location.clone())?;

                        let local: &Local = self.parser_objects.get_local_by_id(
                            local_position.0,
                            local_position.1,
                            location,
                        )?;

                        let local_type: Type = local.0.clone();

                        let expression: Instruction = self.expression()?;

                        self.check_type_mismatch(
                            local_type.clone(),
                            expression.get_type().clone(),
                            Some(&expression),
                        );

                        return Ok(Instruction::LocalMut {
                            name,
                            value: Box::new(expression),
                            kind: local_type,
                        });
                    }

                    if self.match_token(TokenKind::Arrow)? {
                        return self.build_enum_field(name, location);
                    }

                    if self.match_token(TokenKind::LParen)? {
                        return self.build_function_call(name, location);
                    }

                    if object.is_enum() {
                        return Err(ThrushCompilerError::Error(
                            String::from("Invalid type"),
                            String::from(
                                "Enums cannot be used as types; use properties instead with their types.",
                            ),
                            line,
                            Some(span),
                        ));
                    }

                    if object.is_function() {
                        return Err(ThrushCompilerError::Error(
                            String::from("Invalid type"),
                            String::from("Functions cannot be used as types; call it instead."),
                            line,
                            Some(span),
                        ));
                    }

                    self.build_ref(name, location, false)?
                }

                TokenKind::True => {
                    self.only_advance()?;
                    Instruction::Boolean(Type::Bool, true)
                }

                TokenKind::False => {
                    self.only_advance()?;
                    Instruction::Boolean(Type::Bool, false)
                }

                _ => {
                    let previous: &Token = self.advance()?;

                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        format!("Statement '{}' don't allowed.", previous.lexeme.to_str()),
                        previous.line,
                        Some(previous.span),
                    ));
                }
            },
        };

        Ok(primary)
    }

    fn build_type(&mut self, consume: Option<TokenKind>) -> Result<Type, ThrushCompilerError> {
        let builded_type: Result<Type, ThrushCompilerError> = match self.peek().kind {
            tk_kind if tk_kind.is_type() => {
                let tk: &Token = self.advance()?;

                match tk_kind.as_type() {
                    ty if ty.is_integer_type() => Ok(ty),
                    ty if ty.is_float_type() => Ok(ty),
                    ty if ty.is_bool_type() => Ok(ty),
                    ty if ty.is_ptr_type() && self.check(TokenKind::LBracket) => {
                        Ok(self.build_recursive_type(Type::Ptr(None))?)
                    }
                    ty if ty.is_ptr_type() => Ok(ty),
                    ty if ty.is_void_type() => Ok(ty),
                    ty if ty.is_str_type() => Ok(ty),

                    what_heck => Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        format!(
                            "The type '{}' cannot be a value during the compile time.",
                            what_heck
                        ),
                        tk.line,
                        Some(tk.span),
                    )),
                }
            }

            TokenKind::Identifier => {
                let tk: &Token = self.advance()?;

                let name: &str = tk.lexeme.to_str();

                let location: CodeLocation =
                    CodeLocation::new(self.diagnostician.get_instance(), tk.line, tk.span);

                /*if self.rec_structure_ref {
                    return Ok(Instruction::ComplexType(
                        Type::Struct(Vec::new()),
                        object_name,
                        None,
                    ));
                }*/

                let object: FoundObjectId =
                    self.parser_objects.get_object_id(name, location.clone())?;

                if object.is_structure() {
                    let struct_id: &str = object.expected_struct(location.clone())?;

                    let structure: Struct = self
                        .parser_objects
                        .get_struct_by_id(struct_id, location.clone())?;

                    let fields: StructFields = structure.get_fields();

                    return Ok(fields.get_type());
                }

                if object.is_custom_type() {
                    let custom_id: &str = object.expected_custom_type(location.clone())?;

                    let custom: CustomType = self
                        .parser_objects
                        .get_custom_type_by_id(custom_id, location.clone())?;

                    let custom_type_fields: CustomTypeFields = custom.0;

                    return Ok(custom_type_fields.get_type());
                }

                Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    format!("Not found type '{}'.", name),
                    self.previous().line,
                    Some(self.previous().span),
                ))
            }

            what_heck => Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                format!("Expected type, not '{}'", what_heck),
                self.peek().line,
                Some(self.peek().span),
            )),
        };

        if let Some(tk_kind) = consume {
            self.consume(
                tk_kind,
                String::from("Syntax error"),
                format!("Expected '{}'.", tk_kind),
            )?;
        }

        builded_type
    }

    fn build_recursive_type(&mut self, mut before_type: Type) -> Result<Type, ThrushCompilerError> {
        self.consume(
            TokenKind::LBracket,
            String::from("Syntax error"),
            String::from("Expected '['."),
        )?;

        if let Type::Ptr(_) = &mut before_type {
            let mut inner_type: Type = self.build_type(None)?;

            while self.peek().kind == TokenKind::LBracket {
                inner_type = self.build_recursive_type(inner_type)?;
            }

            self.consume(
                TokenKind::RBracket,
                String::from("Syntax error"),
                String::from("Expected ']'."),
            )?;

            return Ok(Type::Ptr(Some(Arc::new(inner_type))));
        }

        unreachable!()
    }

    fn build_ref(
        &mut self,
        name: &'instr str,
        location: CodeLocation,
        take_ptr: bool,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = location.line;
        let span: (usize, usize) = location.span;

        let object: FoundObjectId = self.parser_objects.get_object_id(name, location.clone())?;

        if object.is_constant() {
            let const_id: &str = object.expected_constant(location.clone())?;

            let constant: Constant = self
                .parser_objects
                .get_const_by_id(const_id, location.clone())?;

            return Ok(Instruction::ConstRef {
                name,
                kind: constant.0,
                take: take_ptr,
                location: location.clone(),
            });
        }

        let local_position: (&str, usize) = object.expected_local(location.clone())?;

        let local: &Local = self.parser_objects.get_local_by_id(
            local_position.0,
            local_position.1,
            location.clone(),
        )?;

        let mut local_type: Type = local.0.clone();

        if local.2 {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                format!("Local reference '{}' is undefined.", name),
                line,
                Some(span),
            ));
        }

        if take_ptr {
            local_type = Type::Ptr(Some(Arc::new(local_type)));
        }

        let localref: Instruction = Instruction::LocalRef {
            name,
            kind: local_type.clone(),
            take: take_ptr,
            location,
        };

        if self.match_token(TokenKind::PlusPlus)? | self.match_token(TokenKind::MinusMinus)? {
            let op: &TokenKind = &self.previous().kind;

            type_checking::check_unary_types(op, &local_type, (line, span))?;

            let unaryop: Instruction = Instruction::UnaryOp {
                op,
                expression: Box::from(localref),
                kind: Type::Void,
                is_pre: false,
            };

            return Ok(unaryop);
        }

        Ok(localref)
    }

    fn build_enum_field(
        &mut self,
        name: &'instr str,
        location: CodeLocation,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = location.line;
        let span: (usize, usize) = location.span;

        let object: FoundObjectId = self.parser_objects.get_object_id(name, location.clone())?;
        let enum_id: &str = object.expected_enum(location.clone())?;

        let union: EnumFields = self
            .parser_objects
            .get_enum_by_id(enum_id, location)?
            .get_fields();

        let field: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected enum field identifier."),
        )?;

        let field_name: &str = field.lexeme.to_str();

        if !union.contain_field(field_name) {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                format!("Not found '{}' field in '{}' enum.", name, field_name),
                line,
                Some(span),
            ));
        }

        let field: EnumField = union.get_field(field_name);

        let field_value: Instruction = field.1;

        Ok(field_value)
    }

    fn build_address(
        &mut self,
        name: &'instr str,
        location: CodeLocation,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let object: FoundObjectId = self.parser_objects.get_object_id(name, location.clone())?;
        let local_id: (&str, usize) = object.expected_local(location.clone())?;

        let local: &Local = self
            .parser_objects
            .get_local_by_id(local_id.0, local_id.1, location)?;

        let local_type: Type = local.0.clone();

        if !local_type.is_ptr_type() && !local_type.is_struct_type() && !local_type.is_str_type() {
            self.push_error(
                String::from("Syntax error"),
                format!(
                    "Indexe is only allowed for pointers and structs, not '{}'. ",
                    local_type
                ),
            );
        }

        let mut indexes: Vec<Instruction> = Vec::with_capacity(10);

        let index: Instruction = self.expr()?;

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

        indexes.push(index);

        while self.match_token(TokenKind::LBracket)? {
            let index: Instruction = self.expr()?;

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

            indexes.push(index);
        }

        Ok(Instruction::Address {
            name,
            indexes,
            kind: local_type,
        })
    }

    fn build_function_call(
        &mut self,
        name: &'instr str,
        location: CodeLocation,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let line: usize = location.line;
        let span: (usize, usize) = location.span;

        let mut args_provided: Vec<Instruction> = Vec::with_capacity(10);

        let object: FoundObjectId = self.parser_objects.get_object_id(name, location.clone())?;

        let function_id: &str = object.expected_function(location.clone())?;

        let function: Function = self
            .parser_objects
            .get_function_by_id(location, function_id)?;

        let function_type: Type = function.0;
        let ignore_extra_args: bool = function.2;

        let maximun_function_arguments: usize = function.1.len();

        while self.peek().kind != TokenKind::RParen {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            let instruction: Instruction = self.expr()?;

            self.throw_constructor(&instruction);

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
                line,
                Some(span),
            ));
        }

        if amount_arguments_provided != function.1.len() && !ignore_extra_args {
            let display_args_types: String = if !args_provided.is_empty() {
                args_provided
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
                line,
                Some(span),
            ));
        }

        if !ignore_extra_args {
            for (position, argument) in args_provided.iter().enumerate() {
                let from_type: &Type = argument.get_type();
                let target_type: &Type = &function.1[position];

                self.check_type_mismatch(target_type.clone(), from_type.clone(), Some(argument));
            }
        }

        Ok(Instruction::Call {
            name,
            args: args_provided,
            kind: function_type,
        })
    }

    /* ######################################################################


        PARSER - STRUCTS, ENUMS & FUNCTIONS DECLARATION


    ########################################################################*/

    fn declare(&mut self) {
        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_type_keyword())
            .for_each(|(pos, _)| {
                let _ = self.declare_type(pos);
                self.current = 0;
            });

        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_const_keyword())
            .for_each(|(pos, _)| {
                let _ = self.declare_const(pos);
                self.current = 0;
            });

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
            .filter(|(_, token)| token.kind.is_enum_keyword())
            .for_each(|(pos, _)| {
                let _ = self.declare_enum(pos);
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

    fn declare_type(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_custom_type(true)?;

        Ok(())
    }

    fn declare_struct(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_struct(true)?;

        Ok(())
    }

    fn declare_enum(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_enum(true)?;

        Ok(())
    }

    fn declare_const(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_const(true)?;

        Ok(())
    }

    fn declare_function(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_function(true)?;

        Ok(())
    }

    /* ######################################################################


        PARSER - HELPERS


    ########################################################################*/

    fn throw_unreacheable_code(&mut self) {
        if self.in_unreacheable_code == self.scope_position && self.scope_position != 0 {
            self.push_error(
                String::from("Syntax error"),
                String::from("Unreacheable code."),
            );
        }
    }

    fn throw_outside_loop(&mut self) {
        if !self.inside_a_loop {
            self.push_error(
                String::from("Syntax error"),
                String::from("The flow changer of a loop must go inside one."),
            )
        }
    }

    fn throw_constructor(&mut self, instruction: &Instruction) {
        if matches!(instruction, Instruction::InitStruct { .. }) {
            self.push_error(
                String::from("Syntax error"),
                String::from("A constructor should be stored a local variable."),
            );
        }
    }

    fn check_type_mismatch(
        &mut self,
        target_type: Type,
        from_type: Type,
        expression: Option<&Instruction>,
    ) {
        if expression.is_some_and(|expression| expression.is_binary() || expression.is_group()) {
            if let Err(error) = type_checking::check_type(
                target_type.clone(),
                Type::Void,
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
        }

        if let Err(error) = type_checking::check_type(
            target_type.clone(),
            from_type.clone(),
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
        title: String,
        help: String,
    ) -> Result<&'instr Token<'instr>, ThrushCompilerError> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushCompilerError::Error(
            title,
            help,
            self.previous().line,
            Some(self.previous().span),
        ))
    }

    fn push_error(&mut self, title: String, help: String) {
        self.errors.push(ThrushCompilerError::Error(
            title,
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
        if !self.is_eof() {
            self.current += 1;
            return Ok(());
        }

        Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            self.peek().line,
            None,
        ))
    }

    fn advance(&mut self) -> Result<&'instr Token<'instr>, ThrushCompilerError> {
        if !self.is_eof() {
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
        self.rec_structure_ref = false;

        while !self.is_eof() {
            match self.peek().kind {
                kind if kind.is_keyword() => return,
                _ => {}
            }

            self.current += 1;
        }
    }

    #[inline]
    fn check(&self, kind: TokenKind) -> bool {
        if self.is_eof() {
            return false;
        }

        self.peek().kind == kind
    }

    #[inline(always)]
    const fn is_main_scope(&self) -> bool {
        self.scope_position == 0
    }

    #[must_use]
    #[inline(always)]
    fn is_eof(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    #[must_use]
    #[inline(always)]
    fn peek(&self) -> &'instr Token<'instr> {
        self.tokens.get(self.current).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                "Attempting to get token in invalid current position.",
            );

            unreachable!()
        })
    }

    #[must_use]
    #[inline(always)]
    fn previous(&self) -> &'instr Token<'instr> {
        self.tokens.get(self.current - 1).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Attempting to get token in invalid previous position in line '{}'.",
                    self.peek().line
                ),
            );
            unreachable!()
        })
    }
}
