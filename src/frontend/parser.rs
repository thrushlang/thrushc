use {
    super::{
        super::{
            backend::{compiler::misc::ThrushFile, instruction::Instruction},
            constants::MINIMAL_ERROR_CAPACITY,
            diagnostic::Diagnostic,
            error::ThrushError,
            logging::LogType,
            CORE_LIBRARY_PATH,
        },
        lexer::{DataTypes, Lexer, Token, TokenKind},
        objects::{FoundObject, Globals, ParserObjects},
        preproccesadors::Import,
        scoper::ThrushScoper,
        type_checking,
        types::StructFields,
    },
    ahash::AHashMap as HashMap,
    std::{
        fs, mem,
        path::{Path, PathBuf},
        process,
    },
};

const MINIMAL_STATEMENT_CAPACITY: usize = 100_000;
const MINIMAL_GLOBAL_CAPACITY: usize = 2024;

pub struct Parser<'instr> {
    stmts: Vec<Instruction<'instr>>,
    errors: Vec<ThrushError>,
    tokens: &'instr [Token],
    inside_function: bool,
    at_typed_function: (DataTypes, String),
    at_typed_variable: DataTypes,
    current: usize,
    scope: usize,
    has_entry_point: bool,
    scoper: ThrushScoper<'instr>,
    diagnostic: Diagnostic,
    parser_objects: ParserObjects<'instr>,
}

impl<'instr> Parser<'instr> {
    pub fn new(tokens: &'instr [Token], file: &'instr ThrushFile) -> Self {
        let mut globals: Globals = HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY);

        globals.insert(
            String::from("sizeof"),
            (
                DataTypes::I64,
                Vec::from([DataTypes::Ptr]),
                Vec::new(),
                true,
                false,
                String::new(),
            ),
        );

        Self {
            stmts: Vec::with_capacity(MINIMAL_STATEMENT_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            tokens,
            current: 0,
            inside_function: false,
            at_typed_function: (DataTypes::Void, String::new()),
            at_typed_variable: DataTypes::Void,
            scope: 0,
            has_entry_point: false,
            scoper: ThrushScoper::new(file),
            diagnostic: Diagnostic::new(file),
            parser_objects: ParserObjects::with_globals(globals),
        }
    }

    pub fn start(&mut self) -> &[Instruction<'instr>] {
        self.predefine();

        while !self.end() {
            if let TokenKind::Import = self.peek().kind {
                if let Err(e) = self.import() {
                    self.errors.push(e);
                    self.sync();
                }

                continue;
            }

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
            self.errors.iter().for_each(|error: &ThrushError| {
                self.diagnostic.report(error, LogType::ERROR);
            });
            process::exit(1);
        }

        self.scoper.analyze();

        self.stmts.as_slice()
    }

    fn parse(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        match &self.peek().kind {
            TokenKind::Struct => Ok(self.build_struct()?),
            TokenKind::Fn => Ok(self.build_function(false, false, false)?),
            TokenKind::LBrace => Ok(self.build_block(&mut [], true)?),
            TokenKind::Return => Ok(self.build_return()?),
            TokenKind::Public => Ok(self.build_public_qualifier()?),
            TokenKind::Extern => Ok(self.build_external_qualifier()?),
            TokenKind::Var => Ok(self.build_local_variable(false)?),
            TokenKind::For => Ok(self.build_for_loop()?),
            TokenKind::New => Ok(self.build_struct_initializer()?),
            TokenKind::If => Ok(self.build_if_elif_else()?),
            _ => Ok(self.expression()?),
        }
    }

    fn build_entry_point(&mut self, line: usize) -> Result<Instruction<'instr>, ThrushError> {
        if self.has_entry_point {
            self.errors.push(ThrushError::Error(
                String::from("Duplicated Entrypoint"),
                String::from("The language not support two entrypoints, remove one."),
                line,
                None,
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

        if !self.check_type(TokenKind::LBrace) {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Expected '{'."),
                line,
                Some(self.peek().span),
            ));
        }

        self.has_entry_point = true;

        let body: Box<Instruction<'instr>> = Box::new(self.build_block(&mut [], true)?);

        self.inside_function = false;

        Ok(Instruction::EntryPoint { body })
    }

    fn build_if_elif_else(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let if_keyword: &Token = self.advance()?;
        let line: usize = if_keyword.line;
        let span: (usize, usize) = if_keyword.span;

        if !self.inside_function {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from("The if-elif-else must go inside the a function."),
                line,
                Some(span),
            ));
        }

        let if_condition: Instruction<'instr> = self.expression()?;

        if if_condition.get_data_type() != DataTypes::Bool {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Condition must be type boolean."),
                line,
                Some(span),
            ));
        }

        if !self.check_type(TokenKind::LBrace) {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Expected '{'."),
                line,
                Some(self.peek().span),
            ));
        }

        let if_body: Box<Instruction> = Box::new(self.build_block(&mut [], true)?);

        let mut elfs: Option<Vec<Instruction>> = None;

        if self.check_type(TokenKind::Elif) {
            elfs = Some(Vec::with_capacity(100));
        }

        while self.check_type(TokenKind::Elif) {
            let line: usize = self.advance()?.line;

            let elif_condition: Instruction<'instr> = self.expression()?;

            if elif_condition.get_data_type() != DataTypes::Bool {
                return Err(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Condition must be type boolean."),
                    line,
                    Some(self.peek().span),
                ));
            }

            if !self.check_type(TokenKind::LBrace) {
                return Err(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Expected '{'."),
                    line,
                    Some(self.peek().span),
                ));
            }

            let elif_body: Instruction<'instr> = self.build_block(&mut [], true)?;

            elfs.as_mut().unwrap().push(Instruction::Elif {
                cond: Box::new(elif_condition),
                block: Box::new(elif_body),
            });
        }

        let mut otherwise: Option<Box<Instruction<'instr>>> = None;

        if self.check_type(TokenKind::Else) {
            let line: usize = self.advance()?.line;

            if !self.check_type(TokenKind::LBrace) {
                return Err(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Expected '{'."),
                    line,
                    Some(self.peek().span),
                ));
            }

            let else_body: Instruction<'instr> = self.build_block(&mut [], true)?;

            otherwise = Some(Box::new(Instruction::Else {
                block: Box::new(else_body),
            }));
        }

        Ok(Instruction::If {
            cond: Box::new(if_condition),
            block: if_body,
            elfs,
            otherwise,
        })
    }

    fn build_struct(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name for the struct."),
        )?;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut fields_types: Vec<(String, DataTypes, u32)> = Vec::with_capacity(10);
        let mut field_position: u32 = 0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let line: usize = self.previous().line;
                let span: (usize, usize) = self.previous().span;

                match &self.peek().kind {
                    TokenKind::DataType(kind) => {
                        fields_types.push((String::new(), *kind, field_position))
                    }
                    ident
                        if *ident == TokenKind::Identifier
                            && self
                                .parser_objects
                                .get_struct(self.peek().lexeme.as_ref().unwrap(), (line, span))
                                .is_ok()
                            || name.lexeme.as_ref().unwrap()
                                == self.peek().lexeme.as_ref().unwrap() =>
                    {
                        fields_types.push((
                            self.peek().lexeme.clone().unwrap(),
                            DataTypes::Struct,
                            field_position,
                        ))
                    }
                    what => {
                        return Err(ThrushError::Error(
                            String::from("Undeterminated type"),
                            format!("Type '{}' not exist.", what),
                            line,
                            Some(span),
                        ));
                    }
                };

                self.only_advance()?;
                field_position += 1;
                continue;
            }

            self.only_advance()?;
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

        Ok(Instruction::Struct {
            name: name.lexeme.clone().unwrap(),
            types: fields_types,
        })
    }

    fn build_struct_initializer(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Expected struct reference"),
            String::from("Write the struct name: \"new --> name <-- { ... }\"."),
        )?;

        let line: usize = name.line;
        let span: (usize, usize) = name.span;

        let struct_found: HashMap<String, DataTypes> = self
            .parser_objects
            .get_struct(name.lexeme.as_ref().unwrap(), (line, span))?;

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
                let mut field_name: String = self.previous().lexeme.clone().unwrap();
                let span: (usize, usize) = self.previous().span;

                if field_index as usize >= struct_found.len() {
                    return Err(ThrushError::Error(
                        String::from("Too many fields in struct"),
                        String::from("There are more fields in the structure than normal, they must be the exact amount."),
                        line,
                        Some(span)
                    ));
                }

                if !struct_found.contains_key(&field_name) {
                    return Err(ThrushError::Error(
                        String::from("Syntax error"),
                        String::from("Write valid field name in the struct initialization."),
                        line,
                        Some(span),
                    ));
                }

                let line: usize = self.previous().line;

                let instruction: Instruction = self.expression()?;

                self.throw_if_is_struct_initializer(&instruction, line)?;

                let field_type: DataTypes = instruction.get_data_type();
                let target_type: &DataTypes = struct_found.get(&field_name).unwrap();

                self.check_possible_type_mismatch(
                    *target_type,
                    field_type,
                    Some(&instruction),
                    line,
                );

                if *target_type == DataTypes::Struct {
                    field_name = self.get_struct_type_from_initializer(&instruction);
                }

                fields.push((field_name, instruction, *target_type, field_index));

                field_index += 1;
                continue;
            }

            self.only_advance()?;
        }

        let fields_size: usize = fields.len();
        let fields_needed_size: usize = struct_found.len();

        if fields_size != fields_needed_size {
            return Err(ThrushError::Error(
                String::from("Missing fields"),
                format!(
                    "Expected {} fields, but {} was gived.",
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
            name: name.lexeme.clone().unwrap(),
            fields,
            kind: DataTypes::Struct,
        })
    }

    fn import(&mut self) -> Result<(), ThrushError> {
        if self.scope != 0 {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from("The imports must go in the global scope."),
                self.previous().line,
                None,
            ));
        }

        self.only_advance()?;

        let line: usize = self.previous().line;

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
        )?;

        let path: &str = self
            .consume(
                TokenKind::Str,
                String::from("Syntax error"),
                String::from("Expected a string literal for @import(\"PATH\")."),
            )?
            .lexeme
            .as_ref()
            .unwrap();

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        let path_converted: &Path = Path::new(path);

        if path.starts_with("core")
            && path.split("").filter(|c| *c == ".").count() >= 1
            && !path_converted.exists()
        {
            if !CORE_LIBRARY_PATH.contains_key(path) {
                self.errors.push(ThrushError::Error(
                    String::from("Import error"),
                    String::from("This module not exist in core library."),
                    line,
                    None,
                ));
                return Ok(());
            }

            let library: &(String, String) = CORE_LIBRARY_PATH.get(path).unwrap();

            let mut name: String = String::new();
            let mut path: String = String::new();

            name.clone_from(&library.0);
            path.clone_from(&library.1);

            let file: ThrushFile = ThrushFile {
                name,
                path: PathBuf::from(path),
            };

            let content: String = fs::read_to_string(&file.path).unwrap();

            let tokens: Vec<Token> = Lexer::lex(content.as_bytes(), &file);
            let imports: (Vec<Instruction>, ParserObjects) = Import::generate(tokens, &file);

            self.stmts.extend_from_slice(&imports.0);
            self.parser_objects.merge_globals(imports.1);

            return Ok(());
        }

        if path_converted.exists() {
            if path_converted.is_dir() {
                self.errors.push(ThrushError::Error(
                    String::from("Import error"),
                    String::from("A path to directory is not able to import."),
                    line,
                    None,
                ));

                return Ok(());
            }

            if path_converted.extension().is_none() {
                self.errors.push(ThrushError::Error(
                    String::from("Import error"),
                    String::from("The file should contain a extension (*.th)."),
                    line,
                    None,
                ));

                return Ok(());
            }

            if path_converted.extension().unwrap() != "th" {
                self.errors.push(ThrushError::Error(
                    String::from("Import error"),
                    String::from("Only files with extension (*.th) are allowed to been imported."),
                    line,
                    None,
                ));

                return Ok(());
            }

            if path_converted.file_name().is_none() {
                self.errors.push(ThrushError::Error(
                    String::from("Import error"),
                    String::from("The file should contain a name."),
                    line,
                    None,
                ));

                return Ok(());
            }

            let file: ThrushFile = ThrushFile {
                name: path_converted
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                path: path_converted.to_path_buf(),
            };

            let content: String = fs::read_to_string(&file.path).unwrap();

            let tokens: Vec<Token> = Lexer::lex(content.as_bytes(), &file);
            let imports: (Vec<Instruction>, ParserObjects) = Import::generate(tokens, &file);

            self.stmts.extend_from_slice(&imports.0);
            self.parser_objects.merge_globals(imports.1);

            return Ok(());
        }

        Err(ThrushError::Error(
            String::from("Import not found"),
            String::from("The import not found in the system or std or core library."),
            line,
            None,
        ))
    }

    fn build_external_qualifier(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
        )?;

        let name: &Token = self.consume(
            TokenKind::Str,
            String::from("Syntax error"),
            String::from("Expected a string literal for @extern(\"NAME\")."),
        )?;

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        let instr: Instruction<'instr> = match self.peek().kind {
            TokenKind::Fn => self.build_function(true, true, false)?,
            what => {
                return Err(ThrushError::Error(
                    String::from("Syntax error"),
                    format!("External qualifier is not applicable for \"{}\" .", what),
                    self.peek().line,
                    Some(self.peek().span),
                ))
            }
        };

        Ok(Instruction::Extern {
            name: name.lexeme.clone().unwrap(),
            instr: Box::new(instr),
            kind: TokenKind::Fn,
        })
    }

    fn build_for_loop(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let variable: Instruction = self.build_local_variable(false)?;

        let cond: Instruction = self.expression()?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        let actions: Instruction = self.expression()?;

        let mut variable_clone: Instruction = variable.clone();

        if let Instruction::Var {
            exist_only_comptime,
            ..
        } = &mut variable_clone
        {
            *exist_only_comptime = true;
        }

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        if !self.check_type(TokenKind::LBrace) {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Expected for loop body \"{ ... }\"."),
                self.peek().line,
                Some(self.peek().span),
            ));
        }

        let body: Instruction = self.build_block(&mut [variable_clone], true)?;

        Ok(Instruction::ForLoop {
            variable: Box::new(variable),
            cond: Box::new(cond),
            actions: Box::new(actions),
            block: Box::new(body),
        })
    }

    fn build_local_variable(
        &mut self,
        exist_only_comptime: bool,
    ) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        if self.scope == 0 {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Locals variables should be contained at local scope."),
                self.previous().line,
                Some(self.previous().span),
            ));
        }

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name for the variable."),
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected \":\"."),
        )?;

        let kind: (DataTypes, String) = match &self.peek().kind {
            TokenKind::DataType(kind) => (*kind, String::new()),
            ident
                if *ident == TokenKind::Identifier
                    && self
                        .parser_objects
                        .get_struct(
                            self.peek().lexeme.as_ref().unwrap(),
                            (self.peek().line, self.peek().span),
                        )
                        .is_ok() =>
            {
                (DataTypes::Struct, self.peek().lexeme.clone().unwrap())
            }
            _ => {
                return Err(ThrushError::Error(
                    String::from("Undeterminated type"),
                    format!(
                        "Type \"{}\" not exist.",
                        self.peek().lexeme.as_ref().unwrap()
                    ),
                    line,
                    Some(self.peek().span),
                ));
            }
        };

        self.only_advance()?;

        if self.check_type(TokenKind::SemiColon) && kind.0 == DataTypes::Void {
            let previous: &Token = self.advance()?;

            let line: usize = previous.line;
            let span: (usize, usize) = previous.span;

            self.errors.push(ThrushError::Error(
                String::from("Expected type"),
                String::from("Expected explicit type."),
                line,
                Some(span),
            ));
        } else if self.check_type(TokenKind::SemiColon) {
            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            self.parser_objects.insert_new_local(
                self.scope,
                name.lexeme.as_ref().unwrap(),
                (kind.0, true, false, false, kind.1),
            );

            return Ok(Instruction::Var {
                name: name.lexeme.as_ref().unwrap(),
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

        self.check_possible_type_mismatch(kind.0, value.get_data_type(), Some(&value), line);

        self.check_struct_type_mismatch(&kind.1, &value, line)?;

        self.parser_objects.insert_new_local(
            self.scope,
            name.lexeme.as_ref().unwrap(),
            (kind.0, false, false, false, kind.1),
        );

        let local_variable: Instruction = Instruction::Var {
            name: name.lexeme.as_ref().unwrap(),
            kind: kind.0,
            value: Box::new(value),
            line,
            exist_only_comptime,
        };

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(local_variable)
    }

    fn build_public_qualifier(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        match &self.peek().kind {
            TokenKind::Fn => Ok(self.build_function(true, false, false)?),
            TokenKind::Extern => Ok(self.build_external_qualifier()?),
            TokenKind::Struct => Ok(self.build_struct()?),
            what => Err(ThrushError::Error(
                String::from("Syntax error"),
                format!("Public qualifier is not applicable for \"{}\".", what),
                self.peek().line,
                Some(self.peek().span),
            )),
        }
    }

    fn build_return(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let line: usize = self.advance()?.line;

        if !self.inside_function {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "Return statement outside of function body. Invoke this, in function body.",
                ),
                line,
                None,
            ));
        }

        if self.peek().kind == TokenKind::SemiColon {
            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            self.check_possible_type_mismatch(
                DataTypes::Void,
                self.at_typed_function.0,
                None,
                line,
            );

            return Ok(Instruction::Return(
                Box::new(Instruction::Null),
                DataTypes::Void,
            ));
        }

        let value: Instruction = self.expression()?;

        self.check_possible_type_mismatch(
            self.at_typed_function.0,
            value.get_data_type(),
            Some(&value),
            line,
        );

        self.check_struct_type_mismatch(&self.at_typed_function.1, &value, line)?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Return(
            Box::new(value),
            self.at_typed_function.0,
        ))
    }

    fn build_block(
        &mut self,
        with_instrs: &mut [Instruction<'instr>],
        build_begin_scope: bool,
    ) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        if build_begin_scope {
            self.parser_objects.begin_local_scope();
        }

        let mut stmts: Vec<Instruction> = Vec::new();
        let mut was_emited_deallocators: bool = false;

        with_instrs.iter_mut().for_each(|instruction| {
            stmts.push(mem::take(instruction));
        });

        while !self.match_token(TokenKind::RBrace)? {
            let instr: Instruction = self.parse()?;

            if instr.is_return() {
                if let Some(name) = instr.return_with_ptr() {
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

        Ok(Instruction::Block { stmts })
    }

    fn build_function(
        &mut self,
        is_public: bool,
        is_external: bool,
        only_define: bool,
    ) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        if self.scope != 0 {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "The functions must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                None,
            ));
        }

        self.inside_function = true;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name to the function."),
        )?;

        let line: usize = name.line;

        if name.lexeme.as_ref().unwrap() == "main" {
            if only_define {
                return Err(ThrushError::Error(String::new(), String::new(), 0, None));
            }

            return self.build_entry_point(line);
        }

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
        )?;

        let mut params: Vec<Instruction<'instr>> = Vec::with_capacity(10);
        let mut params_types: Vec<DataTypes> = Vec::with_capacity(10);
        let mut struct_types_params: Vec<(String, usize)> = Vec::with_capacity(10);

        let mut parameter_position: u32 = 0;
        let mut ignore_more_params: bool = false;

        self.parser_objects.begin_local_scope();

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Pass)? {
                ignore_more_params = true;
                continue;
            }

            if !self.match_token(TokenKind::Identifier)? {
                self.errors.push(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Expected argument name."),
                    self.peek().line,
                    Some(self.peek().span),
                ));
            }

            let ident: &str = self.previous().lexeme.as_ref().unwrap();

            if !self.match_token(TokenKind::ColonColon)? {
                self.errors.push(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Expected '::'."),
                    self.peek().line,
                    Some(self.peek().span),
                ));
            }

            let kind: (DataTypes, String) = match self.peek().kind {
                TokenKind::DataType(kind) => (kind, String::new()),
                ident
                    if ident == TokenKind::Identifier
                        && self
                            .parser_objects
                            .get_struct(
                                self.peek().lexeme.as_ref().unwrap(),
                                (self.peek().line, self.peek().span),
                            )
                            .is_ok() =>
                {
                    struct_types_params.push((
                        self.peek().lexeme.clone().unwrap(),
                        parameter_position as usize,
                    ));

                    (DataTypes::Struct, self.peek().lexeme.clone().unwrap())
                }
                what => {
                    return Err(ThrushError::Error(
                        String::from("Undeterminated type"),
                        format!("Type \"{}\" not exist.", what),
                        self.peek().line,
                        Some(self.peek().span),
                    ));
                }
            };

            self.only_advance()?;

            params_types.push(kind.0);

            self.parser_objects.insert_new_local(
                self.scope,
                ident,
                (kind.0, false, false, true, kind.1),
            );

            params.push(Instruction::Param {
                name: ident.to_string(),
                kind: kind.0,
                position: parameter_position,
                line,
            });

            parameter_position += 1;
        }

        self.consume(
            TokenKind::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let return_type: (DataTypes, String) = match &self.peek().kind {
            TokenKind::DataType(kind) => (*kind, String::new()),
            ident
                if *ident == TokenKind::Identifier
                    && self
                        .parser_objects
                        .get_struct(
                            self.peek().lexeme.as_ref().unwrap(),
                            (self.peek().line, self.peek().span),
                        )
                        .is_ok() =>
            {
                (DataTypes::Struct, self.peek().lexeme.clone().unwrap())
            }
            what => {
                return Err(ThrushError::Error(
                    String::from("Undeterminated type"),
                    format!("Type \"{}\" not exist.", what),
                    self.peek().line,
                    Some(self.peek().span),
                ))
            }
        };

        self.only_advance()?;

        self.at_typed_function = return_type.clone();

        let mut function: Instruction = Instruction::Function {
            name: name.lexeme.clone().unwrap(),
            params: params.clone(),
            body: None,
            return_type: return_type.0,
            is_public,
        };

        if ignore_more_params && !is_external {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "Ignore statement \"...\" in functions is only allowed for external functions.",
                ),
                self.peek().line,
                None,
            ));
        }

        if only_define {
            self.parser_objects.insert_new_global(
                name.lexeme.clone().unwrap(),
                (
                    return_type.0,
                    params_types,
                    struct_types_params,
                    true,
                    ignore_more_params,
                    return_type.1,
                ),
            );

            if is_external {
                self.consume(
                    TokenKind::SemiColon,
                    String::from("Syntax error"),
                    String::from("Expected ';'."),
                )?;
            }

            self.inside_function = false;

            return Ok(function);
        }

        if is_external {
            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            self.inside_function = false;
            return Ok(function);
        }

        let body: Box<Instruction> = Box::new(self.build_block(&mut params, false)?);

        self.inside_function = false;

        if let Instruction::Function { body: body_fn, .. } = &mut function {
            *body_fn = Some(body);
        }

        Ok(function)
    }

    /* ######################################################################


        PARSER EXPRESSIONS


    ########################################################################*/

    fn expression(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let instr: Instruction = self.or()?;
        Ok(instr)
    }

    fn or(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction = self.and()?;

        while self.match_token(TokenKind::Or)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.and()?;

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                (self.previous().line, self.previous().span),
            )?;

            instr = Instruction::BinaryOp {
                left: Box::new(instr),
                op,
                right: Box::new(right),
                kind: DataTypes::Bool,
            }
        }

        Ok(instr)
    }

    fn and(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction = self.equality()?;

        while self.match_token(TokenKind::And)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.equality()?;

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                (self.previous().line, self.previous().span),
            )?;

            instr = Instruction::BinaryOp {
                left: Box::new(instr),
                op,
                right: Box::new(right),
                kind: DataTypes::Bool,
            }
        }

        Ok(instr)
    }

    fn equality(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction = self.comparison()?;

        while self.match_token(TokenKind::BangEq)? || self.match_token(TokenKind::EqEq)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.comparison()?;

            let left_type: DataTypes = instr.get_data_type_recursive();
            let right_type: DataTypes = right.get_data_type_recursive();

            type_checking::check_binary_instr(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            instr.is_chained(&right, (self.previous().line, self.previous().span))?;

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind: DataTypes::Bool,
            }
        }

        Ok(instr)
    }

    fn comparison(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction = self.term()?;

        while self.match_token(TokenKind::Greater)?
            || self.match_token(TokenKind::GreaterEq)?
            || self.match_token(TokenKind::Less)?
            || self.match_token(TokenKind::LessEq)?
        {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.term()?;

            let left_type: DataTypes = instr.get_data_type_recursive();
            let right_type: DataTypes = right.get_data_type_recursive();

            type_checking::check_binary_instr(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            instr.is_chained(&right, (self.previous().line, self.previous().span))?;

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind: DataTypes::Bool,
            };
        }

        Ok(instr)
    }

    fn term(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction = self.factor()?;

        while self.match_token(TokenKind::Plus)? || self.match_token(TokenKind::Minus)? {
            let op: &Token = self.previous();
            let right: Instruction = self.factor()?;

            let left_type: DataTypes = instr.get_data_type_recursive();
            let right_type: DataTypes = right.get_data_type_recursive();

            type_checking::check_binary_instr(
                &op.kind,
                &left_type,
                &right_type,
                (op.line, op.span),
            )?;

            let kind: DataTypes = if left_type.is_integer_type() && right_type.is_integer_type() {
                left_type.calculate_integer_datatype(right_type)
            } else if left_type.is_float_type() && right_type.is_float_type() {
                left_type.calculate_float_datatype(right_type)
            } else {
                self.at_typed_variable
            };

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op: &op.kind,
                right: Box::from(right),
                kind,
            };
        }

        Ok(instr)
    }

    fn factor(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction = self.unary()?;

        while self.match_token(TokenKind::Slash)? || self.match_token(TokenKind::Star)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.unary()?;

            let left_type: DataTypes = instr.get_data_type_recursive();
            let right_type: DataTypes = right.get_data_type_recursive();

            type_checking::check_binary_instr(
                op,
                &left_type,
                &right_type,
                (self.previous().line, self.previous().span),
            )?;

            let kind: DataTypes = if left_type.is_integer_type() && right_type.is_integer_type() {
                left_type.calculate_integer_datatype(right_type)
            } else if left_type.is_float_type() && right_type.is_float_type() {
                left_type.calculate_float_datatype(right_type)
            } else {
                self.at_typed_variable
            };

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind,
            };
        }

        Ok(instr)
    }

    fn unary(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        if self.match_token(TokenKind::Bang)? {
            let op: &TokenKind = &self.previous().kind;
            let value: Instruction = self.primary()?;

            type_checking::check_unary_instr(
                op,
                &value.get_data_type(),
                (self.previous().line, self.previous().span),
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                value: Box::from(value),
                kind: DataTypes::Bool,
            });
        } else if self.match_token(TokenKind::PlusPlus)?
            | self.match_token(TokenKind::MinusMinus)?
            | self.match_token(TokenKind::Minus)?
        {
            let op: &TokenKind = &self.previous().kind;
            let mut value: Instruction = self.primary()?;

            if let Instruction::Integer(_, _, is_signed) = &mut value {
                if *op == TokenKind::Minus {
                    *is_signed = true;
                    return Ok(value);
                }
            }

            if let Instruction::Float(_, _, is_signed) = &mut value {
                if *op == TokenKind::Minus {
                    *is_signed = true;
                    return Ok(value);
                }
            }

            let value_type: DataTypes = value.get_data_type();

            type_checking::check_unary_instr(
                op,
                &value_type,
                (self.previous().line, self.previous().span),
            )?;

            return Ok(Instruction::UnaryOp {
                op,
                value: Box::from(value),
                kind: value_type,
            });
        }

        let instr: Instruction = self.primary()?;

        Ok(instr)
    }

    fn primary(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let primary: Instruction = match &self.peek().kind {
            TokenKind::New => self.build_struct_initializer()?,
            TokenKind::DataType(dt) => {
                let datatype: &Token = self.advance()?;
                let line: usize = datatype.line;
                let span: (usize, usize) = datatype.span;

                match dt {
                    dt if dt.is_integer_type() => Instruction::DataTypes(*dt),
                    dt if dt.is_float_type() => Instruction::DataTypes(*dt),
                    dt if dt.is_bool_type() => Instruction::DataTypes(*dt),
                    dt if dt == &DataTypes::Ptr => Instruction::DataTypes(*dt),
                    what_heck_dt => {
                        return Err(ThrushError::Error(
                            String::from("Syntax error"),
                            format!(
                                "The type \"{}\" is not a machine-native type.",
                                what_heck_dt
                            ),
                            line,
                            Some(span),
                        ))
                    }
                }
            }
            TokenKind::LParen => {
                let lparen: &Token = self.advance()?;
                let lparen_line: usize = lparen.line;

                let instr: Instruction = self.expression()?;
                let kind: DataTypes = instr.get_data_type();

                if !instr.is_binary() && !instr.is_group() {
                    return Err(ThrushError::Error(
                        String::from("Syntax error"),
                        String::from(
                            "Group the expressions \"(...)\" is only allowed if contain binary expressions or other group expressions.",
                        ),
                        lparen_line,
                        Some((lparen.span.0, self.peek().span.1))
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
                let str: &Token = self.advance()?;
                let mut str_buffered: String = String::with_capacity(256);

                str_buffered.clone_from(str.lexeme.as_ref().unwrap());

                Instruction::Str(str_buffered)
            }
            TokenKind::Char => {
                let char: &Token = self.advance()?;
                Instruction::Char(char.lexeme.as_ref().unwrap().as_bytes()[0])
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
                    let object_identifier_token: &Token = self.advance()?;
                    let object_name: &str = object_identifier_token.lexeme.as_ref().unwrap();
                    let object_type: TokenKind = object_identifier_token.kind;
                    let object_location: (usize, (usize, usize)) =
                        (object_identifier_token.line, object_identifier_token.span);

                    let object: FoundObject = self
                        .parser_objects
                        .get_object(object_name, object_location)?;

                    if self.match_token(TokenKind::Eq)? {
                        let expr: Instruction = self.expression()?;

                        self.check_possible_type_mismatch(
                            object.0,
                            expr.get_data_type(),
                            Some(&expr),
                            self.previous().line,
                        );

                        self.consume(
                            TokenKind::SemiColon,
                            String::from("Syntax error"),
                            String::from("Expected ';'."),
                        )?;

                        self.parser_objects.insert_new_local(
                            self.scope,
                            object_name,
                            (object.0, false, false, false, String::new()),
                        );

                        return Ok(Instruction::MutVar {
                            name: object_name,
                            value: Box::new(expr),
                            kind: object.0,
                        });
                    } else if self.match_token(TokenKind::LParen)? {
                        return self.call(object_name, object, object_location);
                    }

                    if object.1 {
                        return Err(ThrushError::Error(
                            String::from("Syntax error"),
                            format!(
                                "Variable `{}` is not defined for are use it. Define the variable with value before of the use.",
                                object_name,
                            ),
                            object_location.0,
                            Some(object_location.1)
                        ));
                    }

                    let refvar: Instruction = Instruction::RefVar {
                        name: object_name,
                        line: object_location.0,
                        kind: object.0,
                        struct_type: object.7,
                    };

                    if self.match_token(TokenKind::PlusPlus)?
                        | self.match_token(TokenKind::MinusMinus)?
                    {
                        let op: &TokenKind = &self.previous().kind;

                        type_checking::check_unary_instr(
                            &object_type,
                            &refvar.get_data_type(),
                            object_location,
                        )?;

                        let expr: Instruction = Instruction::UnaryOp {
                            op,
                            value: Box::from(refvar),
                            kind: DataTypes::I64,
                        };

                        return Ok(expr);
                    }

                    refvar
                }
                TokenKind::Pass => {
                    self.only_advance()?;
                    Instruction::Pass
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

                    return Err(ThrushError::Error(
                        String::from("Syntax error"),
                        format!(
                            "Statement \"{}\" don't allowed.",
                            previous.lexeme.as_ref().unwrap(),
                        ),
                        previous.line,
                        Some(previous.span),
                    ));
                }
            },
        };

        Ok(primary)
    }

    fn call(
        &mut self,
        name: &'instr str,
        object: FoundObject,
        location: (usize, (usize, usize)),
    ) -> Result<Instruction<'instr>, ThrushError> {
        if !object.3 {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Call is only allowed for functions."),
                location.0,
                Some(location.1),
            ));
        }

        let mut args: Vec<Instruction> = Vec::new();

        while self.peek().kind != TokenKind::RParen {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            let instruction: Instruction<'instr> = self.expression()?;

            self.throw_if_is_struct_initializer(&instruction, location.0)?;

            args.push(instruction);
        }

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        if args.len() > object.5.len() && !object.4 {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Function called with more arguments than expected."),
                location.0,
                None,
            ));
        }

        if object.5.len() != args.len() && !object.4 {
            let represented_args_types: String = if !args.is_empty() {
                args.iter()
                    .map(|param| param.get_data_type().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                DataTypes::Void.to_string()
            };

            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                format!(
                    "Function called expected all arguments with types ({}) don't ({}).",
                    object
                        .5
                        .iter()
                        .map(|param| param.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    represented_args_types,
                ),
                location.0,
                None,
            ));
        }

        if !object.4 {
            for (index, argument) in args.iter().enumerate() {
                let argument_type: DataTypes = argument.get_data_type();
                let target: DataTypes = object.5[index];

                self.check_possible_type_mismatch(
                    target,
                    argument_type,
                    Some(argument),
                    location.0,
                );

                if target == DataTypes::Struct {
                    let original_struct_type: &(String, usize) =
                        object.6.iter().find(|obj| obj.1 == index).unwrap();

                    self.check_struct_type_mismatch(&original_struct_type.0, argument, location.0)?;
                }
            }
        }

        Ok(Instruction::Call {
            name,
            args,
            kind: object.0,
            struct_type: object.7,
        })
    }

    /* ######################################################################


        PARSER - FUNCTIONS & STRUCTS PRE-DECLARATION


    ########################################################################*/

    fn predefine(&mut self) {
        let mut declarations: Vec<(usize, TokenKind)> = Vec::new();

        for (pos, token) in self.tokens.iter().enumerate() {
            match token.kind {
                TokenKind::Fn => declarations.push((pos, TokenKind::Fn)),
                TokenKind::Struct => declarations.push((pos, TokenKind::Struct)),
                _ => continue,
            }
        }

        declarations
            .iter()
            .filter(|(_, kind)| kind == &TokenKind::Struct)
            .for_each(|(pos, _)| {
                let _ = self.predefine_struct(*pos);
                self.current = 0;
            });

        declarations
            .iter()
            .filter(|(_, kind)| *kind == TokenKind::Fn)
            .for_each(|(pos, _)| {
                let _ = self.declare_function(*pos);
                self.current = 0;
            });
    }

    fn predefine_struct(&mut self, position: usize) -> Result<(), ThrushError> {
        self.current = position;

        if self.scope != 0 {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "The structs must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                None,
            ));
        }

        let is_public_qualifer: bool = self.is_public_qualifier();

        if is_public_qualifer {
            while self.peek().kind != TokenKind::Struct {
                self.only_advance()?;
            }
        }

        self.only_advance()?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Syntax error"),
            String::from("Expected name for the struct."),
        )?;

        let line: usize = name.line;

        if self
            .parser_objects
            .contains_struct(name.lexeme.as_ref().unwrap())
        {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from("The struct already exists."),
                line,
                None,
            ));
        }

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
        )?;

        let mut fields_types: HashMap<String, DataTypes> = HashMap::new();

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let identifier: &Token = self.previous();

                let field_name: String = identifier.lexeme.clone().unwrap();
                let line: usize = identifier.line;

                let field_type: DataTypes = match &self.peek().kind {
                    TokenKind::DataType(kind) => {
                        self.only_advance()?;
                        *kind
                    }
                    ident
                        if *ident == TokenKind::Identifier
                            && self
                                .parser_objects
                                .get_struct(
                                    self.peek().lexeme.as_ref().unwrap(),
                                    (self.peek().line, self.peek().span),
                                )
                                .is_ok()
                            || name.lexeme.as_ref().unwrap()
                                == self.peek().lexeme.as_ref().unwrap() =>
                    {
                        self.only_advance()?;
                        DataTypes::Struct
                    }
                    _ => {
                        return Err(ThrushError::Error(
                            String::from("Undeterminated type"),
                            format!(
                                "Type \"{}\" not exist.",
                                self.peek().lexeme.as_ref().unwrap()
                            ),
                            line,
                            Some(self.peek().span),
                        ));
                    }
                };

                fields_types.insert(field_name, field_type);
                continue;
            }

            self.only_advance()?;
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

        self.parser_objects
            .insert_new_struct(name.lexeme.clone().unwrap(), fields_types);

        Ok(())
    }

    fn declare_function(&mut self, position: usize) -> Result<(), ThrushError> {
        self.current = position;

        let is_external_qualifier: bool = self.is_external_qualifier();

        if is_external_qualifier {
            while self.peek().kind != TokenKind::Fn {
                self.only_advance()?;
            }
        }

        self.build_function(false, is_external_qualifier, true)?;

        Ok(())
    }

    /* ######################################################################


        PARSER - HELPERS


    ########################################################################*/

    fn get_struct_type_from_initializer(&self, instruction: &Instruction) -> String {
        if let Instruction::RefVar { struct_type, .. } = instruction {
            let mut cloned_struct_type: String = String::with_capacity(struct_type.len());
            cloned_struct_type.clone_from(struct_type);

            return cloned_struct_type;
        }

        if let Instruction::NullPtr = instruction {
            return String::new();
        }

        unreachable!()
    }

    fn throw_if_is_struct_initializer(
        &self,
        instruction: &Instruction,
        line: usize,
    ) -> Result<(), ThrushError> {
        if let Instruction::InitStruct { .. } = instruction {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("A struct initializer should be stored a variable."),
                line,
                None,
            ));
        }

        Ok(())
    }

    fn check_struct_type_mismatch(
        &self,
        target_type: &str,
        value: &Instruction,
        line: usize,
    ) -> Result<(), ThrushError> {
        if self.at_typed_function.0 == DataTypes::Struct
            || self.at_typed_variable == DataTypes::Struct
        {
            let mut from_type: &str = "unreacheable";

            if let Instruction::InitStruct {
                name: struct_name, ..
            } = &value
            {
                from_type = struct_name;
            }

            if let Instruction::Call { struct_type, .. } = &value {
                from_type = struct_type;
            }

            if let Instruction::RefVar { struct_type, .. } = &value {
                from_type = struct_type;
            }

            if target_type.trim().to_lowercase() != from_type.trim().to_lowercase() {
                return Err(ThrushError::Error(
                    String::from("Mismatched Types"),
                    format!(
                        "'{}' and '{}' aren't same struct type.",
                        target_type, from_type
                    ),
                    line,
                    None,
                ));
            }
        }

        Ok(())
    }

    fn check_possible_type_mismatch(
        &mut self,
        target: DataTypes,
        from: DataTypes,
        value: Option<&Instruction>,
        line: usize,
    ) {
        if let Some(Instruction::BinaryOp { .. } | Instruction::Group { .. }) = value {
            if value.unwrap().is_binary() || value.unwrap().is_group() {
                if let Err(error) = type_checking::check_types(
                    target,
                    None,
                    value,
                    None,
                    ThrushError::Error(
                        String::from("Mismatched types"),
                        format!(
                            "Mismatched type. Expected '{}' but found '{}'.",
                            target, from
                        ),
                        line,
                        None,
                    ),
                ) {
                    self.errors.push(error);
                }
            }
        } else if let Err(error) = type_checking::check_types(
            target,
            Some(from),
            None,
            None,
            ThrushError::Error(
                String::from("Mismatched types"),
                format!(
                    "Mismatched type. Expected '{}' but found '{}'.",
                    target, from
                ),
                line,
                None,
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
    ) -> Result<&'instr Token, ThrushError> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushError::Error(
            error_title,
            help,
            self.peek().line,
            Some(self.peek().span),
        ))
    }

    fn match_token(&mut self, kind: TokenKind) -> Result<bool, ThrushError> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn only_advance(&mut self) -> Result<(), ThrushError> {
        if !self.end() {
            self.current += 1;
            return Ok(());
        }

        Err(ThrushError::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            self.peek().line,
            None,
        ))
    }

    fn advance(&mut self) -> Result<&'instr Token, ThrushError> {
        if !self.end() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(ThrushError::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            self.peek().line,
            None,
        ))
    }

    fn sync(&mut self) {
        while !self.end() {
            match self.peek().kind {
                TokenKind::Var | TokenKind::Fn => return,
                _ => {}
            }

            self.current += 1;
        }
    }

    #[inline(always)]
    fn is_external_qualifier(&self) -> bool {
        if self.current < 4 {
            false
        } else {
            self.tokens[self.current - 4].kind == TokenKind::Extern
        }
    }

    #[inline(always)]
    fn is_public_qualifier(&self) -> bool {
        if self.current < 2 {
            false
        } else {
            self.tokens[self.current - 2].kind == TokenKind::Extern
        }
    }

    #[inline(always)]
    fn check_type(&self, other_type: TokenKind) -> bool {
        if self.end() {
            return false;
        }

        self.peek().kind == other_type
    }

    #[inline(always)]
    const fn peek(&self) -> &'instr Token {
        &self.tokens[self.current]
    }

    #[inline(always)]
    const fn previous(&self) -> &'instr Token {
        &self.tokens[self.current - 1]
    }

    #[inline(always)]
    fn end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}
