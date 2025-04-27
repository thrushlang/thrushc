use crate::common::misc::CompilerFile;
use crate::middle::instruction::Instruction;
use crate::middle::statement::traits::{
    AttributesExtensions, ConstructorExtensions, CustomTypeFieldsExtensions, EnumExtensions,
    EnumFieldsExtensions, FoundSymbolEither, FoundSymbolExtension, StructFieldsExtensions,
    StructureExtensions, TokenLexemeExtensions,
};
use crate::middle::statement::{
    Constructor, CustomType, CustomTypeFields, EnumField, EnumFields, StructFields,
    ThrushAttributes,
};
use crate::middle::symbols::traits::{ConstantExtensions, LocalExtensions};
use crate::middle::symbols::types::{Constant, Function, Functions, Local, Struct};

use super::lexer::Span;
use super::utils;

use super::{
    super::middle::types::*,
    lexer::Token,
    scoper::ThrushScoper,
    symbols::{FoundSymbolId, SymbolsTable},
    type_checking,
};

use super::super::{
    backend::llvm::compiler::{attributes::LLVMAttribute, builtins, conventions::CallConvention},
    common::{
        constants::MINIMAL_ERROR_CAPACITY, diagnostic::Diagnostician, error::ThrushCompilerError,
        logging,
    },
    logging::LoggingType,
};

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;
use std::process;
use std::rc::Rc;
use std::sync::Arc;

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

    // Token control
    current: usize,
    // Scope control
    scope: usize,

    // Lift locals
    lift_locals: Vec<Instruction<'instr>>,

    // Trigger flags
    entry_point: bool,
    rec_structure_ref: bool,
    inside_a_function: bool,
    inside_a_loop: bool,
    unreacheable_code: usize,

    scoper: ThrushScoper<'instr>,
    diagnostician: Diagnostician,
    symbols: SymbolsTable<'instr>,
}

impl<'instr> Parser<'instr> {
    pub fn new(tokens: &'instr Vec<Token<'instr>>, file: &'instr CompilerFile) -> Self {
        let mut functions: Functions = HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY);

        builtins::include(&mut functions);

        Self {
            stmts: Vec::with_capacity(MINIMAL_STATEMENT_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            lift_locals: Vec::with_capacity(10),
            tokens,
            current: 0,
            inside_a_function: false,
            inside_a_loop: false,
            rec_structure_ref: false,
            unreacheable_code: 0,
            scope: 0,
            entry_point: false,
            scoper: ThrushScoper::new(file),
            diagnostician: Diagnostician::new(file),
            symbols: SymbolsTable::with_functions(functions),
        }
    }

    pub fn start(&mut self) -> &[Instruction<'instr>] {
        let mut type_ctx: TypeContext = TypeContext::new(Type::Void);

        self.declare(&mut type_ctx);

        while !self.is_eof() {
            match self.parse(&mut type_ctx) {
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
                self.diagnostician
                    .build_diagnostic(error, LoggingType::Error);
            });

            process::exit(1);
        }

        println!("{:?}", self.stmts);

        self.scoper.check();
        self.stmts.as_slice()
    }

    fn parse(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        match &self.peek().kind {
            TokenKind::Type => Ok(self.build_custom_type(false)?),
            TokenKind::Struct => Ok(self.build_struct(false)?),
            TokenKind::Enum => Ok(self.build_enum(false, type_ctx)?),
            TokenKind::Fn => Ok(self.build_function(false, type_ctx)?),

            TokenKind::LBrace => Ok(self.build_block(type_ctx)?),
            TokenKind::Return => Ok(self.build_return(type_ctx)?),
            TokenKind::Const => Ok(self.build_const(false, type_ctx)?),
            TokenKind::Local => Ok(self.build_local(false, type_ctx)?),
            TokenKind::For => Ok(self.build_for_loop(type_ctx)?),
            TokenKind::New => Ok(self.build_constructor(type_ctx)?),
            TokenKind::If => Ok(self.build_if_elif_else(type_ctx)?),
            TokenKind::Match => Ok(self.build_match(type_ctx)?),
            TokenKind::While => Ok(self.build_while_loop(type_ctx)?),
            TokenKind::Continue => Ok(self.build_continue()?),
            TokenKind::Break => Ok(self.build_break()?),
            TokenKind::Loop => Ok(self.build_loop(type_ctx)?),

            _ => Ok(self.expression(type_ctx)?),
        }
    }

    fn build_entry_point(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        if self.entry_point {
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

        self.entry_point = true;

        let body: Rc<Instruction> = Rc::new(self.build_block(type_ctx)?);

        self.inside_a_function = false;

        Ok(Instruction::EntryPoint { body })
    }

    fn build_for_loop(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::For,
            String::from("Syntax error"),
            String::from("Expected 'for'."),
        )?;

        let local: Instruction = self.build_local(false, type_ctx)?;
        let cond: Instruction = self.expression(type_ctx)?;

        self.check_type_mismatch(&Type::Bool, cond.get_type(), cond.get_span(), Some(&cond));

        let actions: Instruction = self.expression(type_ctx)?;

        let mut local_clone: Instruction = local.clone();

        if let Instruction::Local { comptime, .. } = &mut local_clone {
            *comptime = true;
        }

        self.add_lift_local(local_clone);

        self.inside_a_loop = true;

        let body: Instruction = self.build_block(type_ctx)?;

        self.inside_a_loop = false;

        Ok(Instruction::ForLoop {
            variable: Rc::new(local),
            cond: Rc::new(cond),
            actions: Rc::new(actions),
            block: Rc::new(body),
        })
    }

    fn build_loop(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::Loop,
            String::from("Syntax error"),
            String::from("Expected 'loop'."),
        )?;

        self.throw_unreacheable_code();

        self.inside_a_loop = true;

        let block: Instruction = self.build_block(type_ctx)?;

        if !block.has_break() && !block.has_return() && !block.has_continue() {
            self.unreacheable_code = self.scope;
        }

        self.inside_a_loop = false;

        Ok(Instruction::Loop {
            block: Rc::new(block),
        })
    }

    fn build_while_loop(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::While,
            String::from("Syntax error"),
            String::from("Expected 'while'."),
        )?;

        self.throw_unreacheable_code();

        let conditional: Instruction = self.expr(type_ctx)?;

        self.check_type_mismatch(
            &Type::Bool,
            conditional.get_type(),
            conditional.get_span(),
            Some(&conditional),
        );

        let block: Instruction = self.build_block(type_ctx)?;

        Ok(Instruction::WhileLoop {
            cond: Rc::new(conditional),
            block: Rc::new(block),
        })
    }

    fn build_continue(&mut self) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::Continue,
            String::from("Syntax error"),
            String::from("Expected 'continue'."),
        )?;

        self.throw_unreacheable_code();

        self.unreacheable_code = self.scope;

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

        self.unreacheable_code = self.scope;

        self.throw_outside_loop();

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Break)
    }

    fn build_match(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.consume(
            TokenKind::Match,
            String::from("Syntax error"),
            String::from("Expected 'match'."),
        )?;

        self.throw_unreacheable_code();

        let mut start_pattern: Instruction = self.expr(type_ctx)?;
        let mut start_block: Instruction = Instruction::Block { stmts: Vec::new() };

        let mut patterns: Vec<Instruction> = Vec::with_capacity(10);
        let mut patterns_stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

        let mut position: u32 = 0;

        while self.match_token(TokenKind::Pattern)? {
            self.scope += 1;
            self.symbols.begin_local_scope();

            let pattern: Instruction = self.expr(type_ctx)?;

            self.check_type_mismatch(
                &Type::Bool,
                pattern.get_type(),
                pattern.get_span(),
                Some(&pattern),
            );

            self.consume(
                TokenKind::ColonColon,
                String::from("Syntax error"),
                String::from("Expected '::'."),
            )?;

            while !self.match_token(TokenKind::Break)? {
                patterns_stmts.push(self.parse(type_ctx)?);
            }

            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            self.scope -= 1;
            self.symbols.end_local_scope();

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
            self.check_type_mismatch(
                &Type::Bool,
                start_pattern.get_type(),
                start_pattern.get_span(),
                Some(&start_pattern),
            );
        }

        let otherwise: Option<Rc<Instruction>> = if self.match_token(TokenKind::Else)? {
            self.consume(
                TokenKind::ColonColon,
                String::from("Syntax error"),
                String::from("Expected '::'."),
            )?;

            let mut stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

            while !self.match_token(TokenKind::Break)? {
                stmts.push(self.parse(type_ctx)?);
            }

            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            if stmts.is_empty() {
                None
            } else {
                Some(Rc::new(Instruction::Else {
                    block: Rc::new(Instruction::Block { stmts }),
                }))
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

    fn build_if_elif_else(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
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

        let if_condition: Instruction = self.expr(type_ctx)?;

        if !if_condition.get_type().is_bool_type() {
            self.push_error(
                String::from("Syntax error"),
                String::from("Condition must be type boolean."),
            );
        }

        let if_body: Rc<Instruction> = Rc::new(self.build_block(type_ctx)?);

        let mut elfs: Vec<Instruction> = Vec::with_capacity(10);

        while self.match_token(TokenKind::Elif)? {
            let elif_condition: Instruction = self.expr(type_ctx)?;

            self.check_type_mismatch(
                &Type::Bool,
                elif_condition.get_type(),
                elif_condition.get_span(),
                Some(&elif_condition),
            );

            let elif_body: Instruction = self.build_block(type_ctx)?;

            if !elif_body.has_instruction() {
                continue;
            }

            elfs.push(Instruction::Elif {
                cond: Rc::new(elif_condition),
                block: Rc::new(elif_body),
            });
        }

        let mut otherwise: Option<Rc<Instruction>> = None;

        if self.match_token(TokenKind::Else)? {
            let else_body: Instruction = self.build_block(type_ctx)?;

            if else_body.has_instruction() {
                otherwise = Some(Rc::new(Instruction::Else {
                    block: Rc::new(else_body),
                }));
            }
        }

        Ok(Instruction::If {
            cond: Rc::new(if_condition),
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

        let span: Span = name.span;
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
            self.symbols.new_custom_type(
                custom_type_name,
                (custom_type_fields, custom_type_attributes),
                span,
            )?;
        }

        Ok(Instruction::Null)
    }

    fn build_enum(
        &mut self,
        declare: bool,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
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

        let span: Span = name.span;

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
                let field_tk: &Token = self.previous();
                let name: &str = field_tk.lexeme.to_str();
                let span: Span = field_tk.span;

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
                        span,
                    ));
                }

                if self.match_token(TokenKind::SemiColon)? {
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

                self.consume(
                    TokenKind::Eq,
                    String::from("Syntax error"),
                    String::from("Expected '='."),
                )?;

                let expression: Instruction = self.expr(type_ctx)?;

                expression.throw_attemping_use_jit(span)?;

                self.consume(
                    TokenKind::SemiColon,
                    String::from("Syntax error"),
                    String::from("Expected ';'."),
                )?;

                self.check_type_mismatch(
                    &field_type,
                    expression.get_type(),
                    expression.get_span(),
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
            self.symbols
                .new_enum(enum_name, (enum_fields, enum_attributes), span)?;
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

        let span: Span = name.span;

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
                self.previous().span,
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
            self.symbols
                .new_struct(struct_name, (fields_types, struct_attributes), span)?;
        }

        Ok(Instruction::Null)
    }

    fn build_constructor(
        &mut self,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
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

        let span: Span = name.span;
        let struct_name: &str = name.lexeme.to_str();

        let struct_found: Struct = self.symbols.get_struct(struct_name, span)?;

        let fields_required: usize = struct_found.get_fields().len();

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut arguments: Constructor = Vec::with_capacity(10);

        let mut amount: usize = 0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_tk: &Token = self.previous();
                let field_name: &str = field_tk.lexeme.to_str();

                if !struct_found.contains_field(field_name) {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Expected existing structure field name."),
                        span,
                    ));
                }

                if amount >= fields_required {
                    return Err(ThrushCompilerError::Error(
                        String::from("Too many fields in structure"),
                        format!("Expected '{}' fields, not '{}'.", fields_required, amount),
                        span,
                    ));
                }

                let expression: Instruction = self.expr(type_ctx)?;

                self.throw_constructor(&expression);

                let expression_type: &Type = expression.get_type();

                if let Some(target_type) = struct_found.get_field_type(field_name) {
                    self.check_type_mismatch(
                        &target_type,
                        expression_type,
                        expression.get_span(),
                        Some(&expression),
                    );

                    arguments.push((field_name, expression, target_type, amount as u32));
                }

                amount += 1;
                continue;
            }

            self.only_advance()?;
        }

        let amount_fields: usize = arguments.len();

        if amount_fields != fields_required {
            return Err(ThrushCompilerError::Error(
                String::from("Missing fields in structure"),
                format!(
                    "Expected '{}' arguments, but '{}' was gived.",
                    fields_required, amount_fields
                ),
                span,
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
            span,
        })
    }

    fn build_const(
        &mut self,
        declare: bool,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        if !self.is_main_scope() {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Constants should be contained at global scope."),
                self.previous().span,
            ));
        }

        self.consume(
            TokenKind::Const,
            String::from("Syntax error"),
            String::from("Expected 'const'."),
        )?;

        let const_tk: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name."),
        )?;

        let name: &str = const_tk.lexeme.to_str();
        let span: Span = const_tk.span;

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

        let value: Instruction = self.expr(type_ctx)?;

        value.throw_attemping_use_jit(span)?;

        self.check_type_mismatch(
            &const_type,
            value.get_type(),
            value.get_span(),
            Some(&value),
        );

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        if declare {
            self.symbols
                .new_constant(name, (const_type, const_attributes), span)?;

            return Ok(Instruction::Null);
        }

        Ok(Instruction::Const {
            name,
            kind: const_type,
            value: Rc::new(value),
            attributes: const_attributes,
            span,
        })
    }

    fn build_local(
        &mut self,
        comptime: bool,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
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

        let local_tk: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name."),
        )?;

        let local_name: &str = local_tk.lexeme.to_str();
        let span: Span = local_tk.span;

        self.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let local_type: Type = self.build_type(None)?;

        if self.match_token(TokenKind::SemiColon)? {
            self.symbols
                .new_local(self.scope, local_name, (local_type.clone(), true), span)?;

            return Ok(Instruction::Local {
                name: local_name,
                kind: local_type,
                value: Rc::new(Instruction::Null),
                span,
                comptime,
            });
        }

        self.symbols
            .new_local(self.scope, local_name, (local_type.clone(), false), span)?;

        self.consume(
            TokenKind::Eq,
            String::from("Syntax error"),
            String::from("Expected '='."),
        )?;

        let value: Instruction = self.expr(type_ctx)?;

        self.check_type_mismatch(
            &local_type,
            value.get_type(),
            value.get_span(),
            Some(&value),
        );

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        let local: Instruction = Instruction::Local {
            name: local_name,
            kind: local_type,
            value: Rc::new(value),
            span,
            comptime,
        };

        Ok(local)
    }

    fn build_return(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
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
            if type_ctx.function_type.is_void_type() {
                return Ok(Instruction::Null);
            }

            self.check_type_mismatch(
                &Type::Void,
                &type_ctx.function_type,
                self.previous().span,
                None,
            );

            return Ok(Instruction::Return(Type::Void, Rc::new(Instruction::Null)));
        }

        let value: Instruction = self.expr(type_ctx)?;

        self.check_type_mismatch(
            &type_ctx.function_type,
            value.get_type(),
            value.get_span(),
            Some(&value),
        );

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Return(
            type_ctx.function_type.clone(),
            Rc::new(value),
        ))
    }

    fn build_block(
        &mut self,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        self.scope += 1;
        self.symbols.begin_local_scope();

        self.symbols
            .lift_locals(self.scope, &mut self.lift_locals)?;

        let mut stmts: Vec<Instruction> = Vec::with_capacity(MINIMAL_SCOPE_CAPACITY);

        while !self.match_token(TokenKind::RBrace)? {
            let instruction: Instruction = self.parse(type_ctx)?;
            stmts.push(instruction)
        }

        self.symbols.end_local_scope();

        self.scoper.add_scope(stmts.clone());
        self.scope -= 1;

        Ok(Instruction::Block { stmts })
    }

    fn build_function(
        &mut self,
        declare: bool,
        type_ctx: &mut TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        self.throw_unreacheable_code();

        if self.scope != 0 {
            self.errors.push(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Functions are only defined globally."),
                self.previous().span,
            ));
        }

        self.consume(
            TokenKind::Fn,
            String::from("Syntax error"),
            String::from("Expected 'fn'."),
        )?;

        self.inside_a_function = true;

        let function_name_tk: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name to the function."),
        )?;

        let name: &str = function_name_tk.lexeme.to_str();
        let span: Span = function_name_tk.span;

        if name == "main" {
            if declare {
                return Ok(Instruction::Null);
            }

            return self.build_entry_point(type_ctx);
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
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            let parameter_tk: &Token = self.consume(
                TokenKind::Identifier,
                String::from("Syntax error"),
                String::from("Expected parameter name."),
            )?;

            let parameter_name: &str = parameter_tk.lexeme.to_str();
            let parameter_span: Span = parameter_tk.span;

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
                span: parameter_span,
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

        type_ctx.function_type = return_type.clone();

        let mut function: Instruction = Instruction::Function {
            name,
            params: params.clone(),
            body: None,
            return_type: return_type.clone(),
            attributes: function_attributes,
        };

        if function_has_ffi || declare {
            if declare {
                self.symbols.new_function(
                    name,
                    (return_type, parameters_types, function_has_ignore),
                    span,
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

        params.iter().cloned().for_each(|param| {
            self.add_lift_local(param);
        });

        let function_body: Rc<Instruction> = Rc::new(self.build_block(type_ctx)?);

        if !return_type.is_void_type() && !function_body.has_return() {
            self.push_error(
                String::from("Syntax error"),
                format!("Missing return with type '{}'.", return_type),
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

        let convention_tk: &Token = self.consume(
            TokenKind::Str,
            String::from("Syntax error"),
            String::from("Expected a string for @convention(\"CONVENTION NAME\")."),
        )?;

        let span: Span = convention_tk.span;
        let name: &[u8] = convention_tk.lexeme;

        if let Some(call_convention) = CALL_CONVENTIONS.get(name) {
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
            span,
        ))
    }

    /* ######################################################################


        PARSER EXPRESSIONS


    ########################################################################*/

    fn expression(
        &mut self,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let instruction: Instruction = self.or(type_ctx)?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(instruction)
    }

    fn expr(&mut self, type_ctx: &TypeContext) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let instruction: Instruction = self.or(type_ctx)?;
        Ok(instruction)
    }

    fn or(&mut self, type_ctx: &TypeContext) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.and(type_ctx)?;

        while self.match_token(TokenKind::Or)? {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let right: Instruction = self.and(type_ctx)?;

            type_checking::check_binary_types(
                &operator,
                expression.get_type(),
                right.get_type(),
                span,
            )?;

            expression = Instruction::BinaryOp {
                left: Rc::new(expression),
                operator,
                right: Rc::new(right),
                kind: Type::Bool,
                span,
            }
        }

        Ok(expression)
    }

    fn and(&mut self, type_ctx: &TypeContext) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.equality(type_ctx)?;

        while self.match_token(TokenKind::And)? {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let right: Instruction = self.equality(type_ctx)?;

            type_checking::check_binary_types(
                &operator,
                expression.get_type(),
                right.get_type(),
                span,
            )?;

            expression = Instruction::BinaryOp {
                left: Rc::new(expression),
                operator,
                right: Rc::new(right),
                kind: Type::Bool,
                span,
            }
        }

        Ok(expression)
    }

    fn equality(
        &mut self,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.comparison(type_ctx)?;

        if self.match_token(TokenKind::BangEq)? || self.match_token(TokenKind::EqEq)? {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let right: Instruction = self.comparison(type_ctx)?;

            type_checking::check_binary_types(
                &operator,
                expression.get_type(),
                right.get_type(),
                span,
            )?;

            expression = Instruction::BinaryOp {
                left: Rc::from(expression),
                operator,
                right: Rc::from(right),
                kind: Type::Bool,
                span,
            }
        }

        Ok(expression)
    }

    fn comparison(
        &mut self,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.term(type_ctx)?;

        if self.match_token(TokenKind::Greater)?
            || self.match_token(TokenKind::GreaterEq)?
            || self.match_token(TokenKind::Less)?
            || self.match_token(TokenKind::LessEq)?
        {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let right: Instruction = self.term(type_ctx)?;

            type_checking::check_binary_types(
                &operator,
                expression.get_type(),
                right.get_type(),
                span,
            )?;

            expression = Instruction::BinaryOp {
                left: Rc::from(expression),
                operator,
                right: Rc::from(right),
                kind: Type::Bool,
                span,
            };
        }

        Ok(expression)
    }

    fn term(&mut self, type_ctx: &TypeContext) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.factor(type_ctx)?;

        while self.match_token(TokenKind::Plus)?
            || self.match_token(TokenKind::Minus)?
            || self.match_token(TokenKind::LShift)?
            || self.match_token(TokenKind::RShift)?
        {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let right: Instruction = self.factor(type_ctx)?;

            let left_type: &Type = expression.get_type();
            let right_type: &Type = right.get_type();

            type_checking::check_binary_types(&operator, left_type, right_type, span)?;

            let kind: &Type = left_type.precompute_type(right_type);

            expression = Instruction::BinaryOp {
                left: Rc::from(expression.clone()),
                operator,
                right: Rc::from(right),
                kind: kind.clone(),
                span,
            };
        }

        Ok(expression)
    }

    fn factor(
        &mut self,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut expression: Instruction = self.unary(type_ctx)?;

        while self.match_token(TokenKind::Slash)? || self.match_token(TokenKind::Star)? {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let right: Instruction = self.unary(type_ctx)?;

            let left_type: &Type = expression.get_type();
            let right_type: &Type = right.get_type();

            type_checking::check_binary_types(
                &operator,
                left_type,
                right_type,
                self.previous().span,
            )?;

            let kind: &Type = left_type.precompute_type(right_type);

            expression = Instruction::BinaryOp {
                left: Rc::from(expression.clone()),
                operator,
                right: Rc::from(right),
                kind: kind.clone(),
                span,
            };
        }

        Ok(expression)
    }

    fn unary(
        &mut self,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        if self.match_token(TokenKind::Bang)? {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let expression: Instruction = self.primary(type_ctx)?;

            type_checking::check_unary_types(
                &operator,
                expression.get_type(),
                self.previous().span,
            )?;

            return Ok(Instruction::UnaryOp {
                operator,
                expression: Rc::from(expression),
                kind: Type::Bool,
                is_pre: false,
                span,
            });
        }

        if self.match_token(TokenKind::Minus)? {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            let mut expression: Instruction = self.primary(type_ctx)?;

            expression.cast_signess(operator);

            let expression_type: &Type = expression.get_type();

            type_checking::check_unary_types(&operator, expression_type, self.previous().span)?;

            return Ok(Instruction::UnaryOp {
                operator,
                expression: Rc::from(expression.clone()),
                kind: expression_type.clone(),
                is_pre: false,
                span,
            });
        }

        let instr: Instruction = self.primary(type_ctx)?;

        Ok(instr)
    }

    fn primary(
        &mut self,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let primary: Instruction = match &self.peek().kind {
            TokenKind::Take => {
                self.only_advance()?;

                if self.match_token(TokenKind::Identifier)? {
                    let identifier_tk: &Token = self.previous();
                    let name: &str = identifier_tk.lexeme.to_str();

                    return self.build_ref(name, identifier_tk.span, true);
                }

                return Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("Take the value is only allowed by references."),
                    self.previous().span,
                ));
            }

            TokenKind::Carry => {
                let carry_tk: &Token = self.advance()?;
                let span: Span = carry_tk.span;

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
                    let identifier_tk: &Token = self.consume(
                        TokenKind::Identifier,
                        String::from("Syntax error"),
                        String::from("Expected 'identifier'."),
                    )?;

                    let name: &str = identifier_tk.lexeme.to_str();

                    self.build_ref(name, span, false)?;

                    return Ok(Instruction::Carry {
                        name,
                        expression: None,
                        carry_type,
                        span,
                    });
                }

                let expression: Instruction = self.expr(type_ctx)?;
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
                    expression: Some(Rc::new(expression)),
                    carry_type,
                    span,
                }
            }

            TokenKind::Write => {
                let write_tk: &Token = self.advance()?;
                let span: Span = write_tk.span;

                self.consume(
                    TokenKind::LBracket,
                    String::from("Syntax error"),
                    String::from("Expected '['."),
                )?;

                let write_type: Type = self.build_type(None)?;

                self.consume(
                    TokenKind::RBracket,
                    String::from("Syntax error"),
                    String::from("Expected ']'."),
                )?;

                let value: Instruction = self.expr(type_ctx)?;

                self.check_type_mismatch(
                    &write_type,
                    value.get_type(),
                    value.get_span(),
                    Some(&value),
                );

                self.consume(
                    TokenKind::Arrow,
                    String::from("Syntax error"),
                    String::from("Expected '->'."),
                )?;

                if self.check(TokenKind::Identifier) {
                    let identifier_tk: &Token = self.consume(
                        TokenKind::Identifier,
                        String::from("Syntax error"),
                        String::from("Expected 'identifier'."),
                    )?;

                    let name: &str = identifier_tk.lexeme.to_str();

                    self.build_ref(name, span, false)?;

                    return Ok(Instruction::Write {
                        write_to: (name, None),
                        write_value: Rc::new(value),
                        write_type,
                        span,
                    });
                }

                let expression: Instruction = self.expr(type_ctx)?;
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
                    write_to: ("", Some(Rc::new(expression))),
                    write_value: Rc::new(value),
                    write_type,
                    span,
                }
            }

            TokenKind::Address => {
                self.only_advance()?;

                let identifier_tk: &Token = self.consume(
                    TokenKind::Identifier,
                    String::from("Syntax error"),
                    String::from("Expected 'identifier'."),
                )?;

                let name: &str = identifier_tk.lexeme.to_str();
                let span: Span = identifier_tk.span;

                self.consume(
                    TokenKind::LBracket,
                    String::from("Syntax error"),
                    String::from("Expected '['."),
                )?;

                return self.build_address(name, span, type_ctx);
            }

            TokenKind::New => self.build_constructor(type_ctx)?,

            TokenKind::PlusPlus => {
                let operator_tk: &Token = self.advance()?;
                let operator: TokenKind = operator_tk.kind;
                let span: Span = operator_tk.span;

                let expression: Instruction = self.expr(type_ctx)?;

                if !expression.is_local_ref() {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Only local references can be pre-incremented."),
                        self.previous().span,
                    ));
                }

                let unaryop: Instruction = Instruction::UnaryOp {
                    operator,
                    expression: Rc::from(expression),
                    kind: Type::Void,
                    is_pre: true,
                    span,
                };

                return Ok(unaryop);
            }

            TokenKind::MinusMinus => {
                let operator_tk: &Token = self.advance()?;
                let operator: TokenKind = operator_tk.kind;
                let span: Span = operator_tk.span;

                let expression: Instruction = self.expr(type_ctx)?;

                if !expression.is_local_ref() {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Only local references can be pre-decremented."),
                        self.previous().span,
                    ));
                }

                let unaryop: Instruction = Instruction::UnaryOp {
                    operator,
                    expression: Rc::from(expression),
                    kind: Type::Void,
                    is_pre: true,
                    span,
                };

                return Ok(unaryop);
            }

            TokenKind::LParen => {
                let span: Span = self.advance()?.span;

                let expression: Instruction = self.expression(type_ctx)?;
                let kind: &Type = expression.get_type();

                if !expression.is_binary() && !expression.is_group() {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from(
                            "Grouping '(...)' is only allowed with binary expressions or other grouped expressions.",
                        ),
                        span,
                    ));
                }

                self.consume(
                    TokenKind::RParen,
                    String::from("Syntax error"),
                    String::from("Expected ')'."),
                )?;

                return Ok(Instruction::Group {
                    expression: Rc::new(expression.clone()),
                    kind: kind.clone(),
                    span,
                });
            }

            TokenKind::Str => {
                let str_tk: &Token = self.advance()?;
                let str: &[u8] = str_tk.lexeme;
                let span: Span = str_tk.span;

                Instruction::Str(Type::Str, str.parse_scapes(span)?, span)
            }

            TokenKind::Char => {
                let char: &Token = self.advance()?;
                let span: Span = char.span;

                Instruction::Char(Type::Char, char.lexeme[0], span)
            }

            kind => match kind {
                TokenKind::Integer => {
                    let integer_tk: &Token = self.advance()?;
                    let integer: &str = integer_tk.lexeme.to_str();
                    let span: Span = integer_tk.span;

                    let parsed_integer: (Type, f64) = utils::parse_number(integer, span)?;

                    let integer_type: Type = parsed_integer.0;
                    let integer_value: f64 = parsed_integer.1;

                    Instruction::Integer(integer_type, integer_value, false, span)
                }

                TokenKind::Float => {
                    let float_tk: &Token = self.advance()?;
                    let float: &str = float_tk.lexeme.to_str();
                    let span: Span = float_tk.span;

                    let parsed_float: (Type, f64) = utils::parse_number(float, span)?;

                    let float_type: Type = parsed_float.0;
                    let float_value: f64 = parsed_float.1;

                    Instruction::Float(float_type, float_value, false, span)
                }

                TokenKind::Identifier => {
                    let identifier_tk: &Token = self.advance()?;

                    let name: &str = identifier_tk.lexeme.to_str();
                    let span: Span = identifier_tk.span;

                    self.throw_unreacheable_code();

                    /*if self.rec_structure_ref {
                        return Ok(Instruction::ComplexType(
                            Type::Struct(Vec::new()),
                            object_name,
                            None,
                        ));
                    }*/

                    let object: FoundSymbolId = self.symbols.get_symbols_id(name, span)?;

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
                            .symbols
                            .get_custom_type_by_id(custom_id, location)?;

                        let custom_type_fields: CustomTypeFields = custom.0;

                        return Ok(custom_type_fields.get_type());
                    }*/

                    if self.match_token(TokenKind::Eq)? {
                        let object: FoundSymbolId = self.symbols.get_symbols_id(name, span)?;

                        let local_position: (&str, usize) = object.expected_local(span)?;

                        let local: &Local = self.symbols.get_local_by_id(
                            local_position.0,
                            local_position.1,
                            span,
                        )?;

                        let local_type: Type = local.0.clone();

                        let expression: Instruction = self.expression(type_ctx)?;

                        self.check_type_mismatch(
                            &local_type.clone(),
                            expression.get_type(),
                            expression.get_span(),
                            Some(&expression),
                        );

                        return Ok(Instruction::LocalMut {
                            name,
                            value: Rc::new(expression),
                            kind: local_type,
                            span,
                        });
                    }

                    if self.match_token(TokenKind::Arrow)? {
                        return self.build_enum_field(name, span);
                    }

                    if self.match_token(TokenKind::LParen)? {
                        return self.build_function_call(name, span, type_ctx);
                    }

                    if object.is_enum() {
                        return Err(ThrushCompilerError::Error(
                            String::from("Invalid type"),
                            String::from(
                                "Enums cannot be used as types; use properties instead with their types.",
                            ),
                            span,
                        ));
                    }

                    if object.is_function() {
                        return Err(ThrushCompilerError::Error(
                            String::from("Invalid type"),
                            String::from("Functions cannot be used as types; call it instead."),
                            span,
                        ));
                    }

                    self.build_ref(name, span, false)?
                }

                TokenKind::True => Instruction::Boolean(Type::Bool, true, self.advance()?.span),
                TokenKind::False => Instruction::Boolean(Type::Bool, false, self.advance()?.span),

                _ => {
                    let previous: &Token = self.advance()?;

                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        format!("Statement '{}' don't allowed.", previous.lexeme.to_str()),
                        previous.span,
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
                        tk.span,
                    )),
                }
            }

            TokenKind::Identifier => {
                let identifier_tk: &Token = self.advance()?;

                let name: &str = identifier_tk.lexeme.to_str();
                let span: Span = identifier_tk.span;

                /*if self.rec_structure_ref {
                    return Ok(Instruction::ComplexType(
                        Type::Struct(Vec::new()),
                        object_name,
                        None,
                    ));
                }*/

                let object: FoundSymbolId = self.symbols.get_symbols_id(name, span)?;

                if object.is_structure() {
                    let struct_id: &str = object.expected_struct(span)?;

                    let structure: Struct = self.symbols.get_struct_by_id(struct_id, span)?;

                    let fields: StructFields = structure.get_fields();

                    return Ok(fields.get_type());
                }

                if object.is_custom_type() {
                    let custom_id: &str = object.expected_custom_type(span)?;

                    let custom: CustomType = self.symbols.get_custom_type_by_id(custom_id, span)?;

                    let custom_type_fields: CustomTypeFields = custom.0;

                    return Ok(custom_type_fields.get_type());
                }

                Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    format!("Not found type '{}'.", name),
                    span,
                ))
            }

            what_heck => Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                format!("Expected type, not '{}'", what_heck),
                self.previous().span,
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
        span: Span,
        take_ptr: bool,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let object: FoundSymbolId = self.symbols.get_symbols_id(name, span)?;

        if object.is_constant() {
            let const_id: &str = object.expected_constant(span)?;
            let constant: Constant = self.symbols.get_const_by_id(const_id, span)?;
            let constant_type: Type = constant.get_type();

            return Ok(Instruction::ConstRef {
                name,
                kind: constant_type,
                take: take_ptr,
                span,
            });
        }

        let local_position: (&str, usize) = object.expected_local(span)?;

        let local: &Local =
            self.symbols
                .get_local_by_id(local_position.0, local_position.1, span)?;

        let mut local_type: Type = local.get_type();

        if local.is_undefined() {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                format!("Local reference '{}' is undefined.", name),
                span,
            ));
        }

        if take_ptr {
            local_type = Type::Ptr(Some(Arc::new(local_type)));
        }

        let localref: Instruction = Instruction::LocalRef {
            name,
            kind: local_type.clone(),
            take: take_ptr,
            span,
        };

        if self.match_token(TokenKind::PlusPlus)? | self.match_token(TokenKind::MinusMinus)? {
            let operator_tk: &Token = self.previous();
            let operator: TokenKind = operator_tk.kind;
            let span: Span = operator_tk.span;

            type_checking::check_unary_types(&operator, &local_type, span)?;

            let unaryop: Instruction = Instruction::UnaryOp {
                operator,
                expression: Rc::from(localref),
                kind: Type::Void,
                is_pre: false,
                span,
            };

            return Ok(unaryop);
        }

        Ok(localref)
    }

    fn build_enum_field(
        &mut self,
        name: &'instr str,
        span: Span,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let object: FoundSymbolId = self.symbols.get_symbols_id(name, span)?;
        let enum_id: &str = object.expected_enum(span)?;

        let union: EnumFields = self.symbols.get_enum_by_id(enum_id, span)?.get_fields();

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
                span,
            ));
        }

        let field: EnumField = union.get_field(field_name);
        let field_value: Instruction = field.1;

        Ok(field_value)
    }

    fn build_address(
        &mut self,
        name: &'instr str,
        span: Span,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let object: FoundSymbolId = self.symbols.get_symbols_id(name, span)?;
        let local_id: (&str, usize) = object.expected_local(span)?;

        let local: &Local = self.symbols.get_local_by_id(local_id.0, local_id.1, span)?;

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

        let index: Instruction = self.expr(type_ctx)?;

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
            let index: Instruction = self.expr(type_ctx)?;

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
            span,
        })
    }

    fn build_function_call(
        &mut self,
        name: &'instr str,
        span: Span,
        type_ctx: &TypeContext,
    ) -> Result<Instruction<'instr>, ThrushCompilerError> {
        let mut args_provided: Vec<Instruction> = Vec::with_capacity(10);

        let object: FoundSymbolId = self.symbols.get_symbols_id(name, span)?;

        let function_id: &str = object.expected_function(span)?;

        let function: Function = self.symbols.get_function_by_id(span, function_id)?;

        let function_type: Type = function.0;
        let ignore_extra_args: bool = function.2;

        let maximun_function_arguments: usize = function.1.len();

        while self.peek().kind != TokenKind::RParen {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            let instruction: Instruction = self.expr(type_ctx)?;

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
                span,
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
                span,
            ));
        }

        if !ignore_extra_args {
            for (position, argument) in args_provided.iter().enumerate() {
                let from_type: &Type = argument.get_type();
                let target_type: &Type = &function.1[position];

                self.check_type_mismatch(
                    target_type,
                    from_type,
                    argument.get_span(),
                    Some(argument),
                );
            }
        }

        Ok(Instruction::Call {
            name,
            args: args_provided,
            kind: function_type,
            span,
        })
    }

    /* ######################################################################


        PARSER - STRUCTS, ENUMS & FUNCTIONS DECLARATION


    ########################################################################*/

    fn declare(&mut self, type_ctx: &mut TypeContext) {
        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_type_keyword())
            .for_each(|(pos, _)| {
                let _ = self.declare_custom_type(pos);
                self.current = 0;
            });

        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_const_keyword())
            .for_each(|(pos, _)| {
                let _ = self.declare_const(pos, type_ctx);
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
                let _ = self.declare_enum(pos, type_ctx);
                self.current = 0;
            });

        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_function_keyword())
            .for_each(|(pos, _)| {
                let _ = self.declare_function(pos, type_ctx);
                self.current = 0;
            });
    }

    fn declare_custom_type(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_custom_type(true)?;

        Ok(())
    }

    fn declare_struct(&mut self, position: usize) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_struct(true)?;

        Ok(())
    }

    fn declare_enum(
        &mut self,
        position: usize,
        type_ctx: &mut TypeContext,
    ) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_enum(true, type_ctx)?;

        Ok(())
    }

    fn declare_const(
        &mut self,
        position: usize,
        type_ctx: &mut TypeContext,
    ) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_const(true, type_ctx)?;

        Ok(())
    }

    fn declare_function(
        &mut self,
        position: usize,
        type_ctx: &mut TypeContext,
    ) -> Result<(), ThrushCompilerError> {
        self.current = position;
        self.build_function(true, type_ctx)?;

        Ok(())
    }

    /* ######################################################################


        PARSER - HELPERS


    ########################################################################*/

    fn throw_unreacheable_code(&mut self) {
        if self.unreacheable_code == self.scope && !self.is_main_scope() {
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
        target: &Type,
        from: &Type,
        span: Span,
        expr: Option<&Instruction>,
    ) {
        let error: ThrushCompilerError = ThrushCompilerError::Error(
            String::from("Mismatched types"),
            format!("Expected '{}' but found '{}'.", target, from),
            span,
        );

        if expr.is_some_and(|expr| expr.is_binary() || expr.is_group()) {
            if let Err(error) = type_checking::check_type(target, &Type::Void, expr, None, error) {
                self.errors.push(error);
            }
        } else if let Err(error) = type_checking::check_type(target, from, None, None, error) {
            self.errors.push(error);
        }
    }

    fn add_lift_local(&mut self, instruction: Instruction<'instr>) {
        self.lift_locals.push(instruction);
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
            self.previous().span,
        ))
    }

    fn push_error(&mut self, title: String, help: String) {
        self.errors.push(ThrushCompilerError::Error(
            title,
            help,
            self.previous().span,
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
            self.peek().span,
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
            self.peek().span,
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
        self.scope == 0
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
                    self.peek().span.get_line()
                ),
            );
            unreachable!()
        })
    }
}
