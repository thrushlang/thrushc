use {
    super::{
        super::{
            backend::{compiler::misc::ThrushFile, instruction::Instruction},
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
        let mut globals: Globals = HashMap::new();

        globals.insert(
            String::from("sizeof"),
            (
                DataTypes::I64,
                Vec::from([DataTypes::Ptr]),
                Vec::new(),
                true,
                false,
            ),
        );

        Self {
            stmts: Vec::new(),
            errors: Vec::new(),
            tokens,
            current: 0,
            inside_function: false,
            at_typed_function: (DataTypes::Void, String::new()),
            at_typed_variable: DataTypes::Void,
            scope: 0,
            has_entry_point: false,
            scoper: ThrushScoper::new(file),
            diagnostic: Diagnostic::new(file),
            parser_objects: ParserObjects::new(globals),
        }
    }

    pub fn start(&mut self) -> &[Instruction<'instr>] {
        self.start_predeclaration();

        while !self.end() {
            if let TokenKind::Import = self.peek().kind {
                if let Err(e) = self.import() {
                    self.errors.push(e);
                    self.sync();
                }

                continue;
            }

            if let TokenKind::Public = self.peek().kind {
                let _ = self.only_advance();

                if self.check_kind(TokenKind::Struct) {
                    if let Err(e) = self.check_struct() {
                        self.errors.push(e);
                        self.sync();
                    }

                    continue;
                } else {
                    self.current -= 1;
                }
            }

            if let TokenKind::Struct = self.peek().kind {
                if let Err(e) = self.check_struct() {
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
                self.diagnostic.report(error, LogType::ERROR, false);
            });
            process::exit(1);
        }

        self.scoper.analyze();

        self.stmts.as_slice()
    }

    fn parse(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.parser_objects.decrease_local_references(self.scope);

        match &self.peek().kind {
            TokenKind::Fn => Ok(self.build_function(false)?),
            TokenKind::LBrace => Ok(self.build_block(&mut [], true)?),
            TokenKind::Return => Ok(self.build_return()?),
            TokenKind::Public => Ok(self.build_public_qualifier()?),
            TokenKind::Extern => Ok(self.build_external_qualifier()?),
            TokenKind::Var => Ok(self.build_local_variable(false)?),
            TokenKind::For => Ok(self.build_for_loop()?),
            TokenKind::New => Ok(self.build_struct_initializer()?),
            _ => Ok(self.expression()?),
        }
    }

    fn build_struct_initializer(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Expected struct reference"),
            String::from("Write the struct name: \"new --> name <-- { ... };\"."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        let struct_found: HashMap<String, DataTypes> = self
            .parser_objects
            .get_struct(name.lexeme.as_ref().unwrap(), line)?;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
            line,
        )?;

        let mut fields: StructFields = Vec::new();
        let mut count: u32 = 0;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_name: String = self.previous().lexeme.clone().unwrap();

                if count as usize >= struct_found.len() {
                    return Err(ThrushError::Error(
                        String::from("Too many fields in struct"),
                        String::from("There are more fields in the structure than normal, they must be the exact amount."),
                        line,
                        String::new(),
                    ));
                }

                if !struct_found.contains_key(&field_name) {
                    return Err(ThrushError::Error(
                        String::from("Field name not found"),
                        String::from("Write valid field name in the struct initialization."),
                        line,
                        String::new(),
                    ));
                }

                let line: usize = self.previous().line;

                let expr: Instruction = self.expression()?;
                let field_type: DataTypes = expr.get_data_type();

                let target_type: &DataTypes = struct_found.get(&field_name).unwrap();

                self.check_possible_type_mismatch(*target_type, field_type, Some(&expr), line);

                fields.push((field_name, expr, *target_type, count));

                count += 1;

                continue;
            }

            self.only_advance()?;
        }

        self.consume(
            TokenKind::RBrace,
            String::from("Syntax error"),
            String::from("Expected '}'."),
            line,
        )?;

        Ok(Instruction::Struct {
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
                String::new(),
            ));
        }

        self.only_advance()?;

        let line: usize = self.previous().line;

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
            line,
        )?;

        let path: &str = self
            .consume(
                TokenKind::Str,
                String::from("Syntax error"),
                String::from("Expected a String literal for @import(\"PATH\")."),
                line,
            )?
            .lexeme
            .as_ref()
            .unwrap();

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
            line,
        )?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
            line,
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
                    String::new(),
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
            let imports: (Vec<Instruction<'_>>, ParserObjects<'_>) =
                Import::generate(tokens, &file);

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
                    String::from("@import(\"dir/to/file.th\");"),
                ));
                return Ok(());
            }

            if path_converted.extension().is_none() {
                self.errors.push(ThrushError::Error(
                    String::from("Import error"),
                    String::from("The file should contain a extension (*.th)."),
                    line,
                    String::from("@import(\"only/th/extension/file.th\");"),
                ));
                return Ok(());
            }

            if path_converted.extension().unwrap() != "th" {
                self.errors.push(ThrushError::Error(
                    String::from("Import error"),
                    String::from("Only files with extension (*.th) are allowed to been imported."),
                    line,
                    String::from("@import(\"only/thrush/files/file.th\");"),
                ));
                return Ok(());
            }

            if path_converted.file_name().is_none() {
                self.errors.push(ThrushError::Error(
                    String::from("Import error"),
                    String::from("The file should contain a name."),
                    line,
                    String::from("@import(\"only/valid/files/file.th\");"),
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
            let imports: (Vec<Instruction<'_>>, ParserObjects<'_>) =
                Import::generate(tokens, &file);

            self.stmts.extend_from_slice(&imports.0);
            self.parser_objects.merge_globals(imports.1);

            return Ok(());
        }

        Err(ThrushError::Error(
            String::from("Import not found"),
            String::from("The import not found in the system or std or core library."),
            line,
            String::new(),
        ))
    }

    fn build_external_qualifier(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let line: usize = self.previous().line;

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
            line,
        )?;

        let name: &Token = self.consume(
            TokenKind::Str,
            String::from("Syntax error"),
            String::from("Expected a string literal for @extern(\"NAME\")."),
            line,
        )?;

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
            line,
        )?;

        let instr: Instruction<'instr> = match self.peek().kind {
            TokenKind::Fn => self.build_function(true)?,
            what => {
                return Err(ThrushError::Error(
                    String::from("Syntax error"),
                    format!("External qualifier is not applicable for \"{}\" .", what),
                    self.peek().line,
                    String::new(),
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

        let line: usize = self.previous().line;

        let variable: Instruction<'instr> = self.build_local_variable(false)?;

        let cond: Instruction<'instr> = self.expression()?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
            line,
        )?;

        let actions: Instruction<'instr> = self.expression()?;

        let mut variable_clone: Instruction<'instr> = variable.clone();

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
            line,
        )?;

        if !self.check_kind(TokenKind::LBrace) {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Expected for loop body \"{ ... }\"."),
                line,
                String::from("{ ... }"),
            ));
        }

        let body: Instruction<'instr> = self.build_block(&mut [variable_clone], true)?;

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

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Expected name"),
            String::from("Write the name for the variable."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::Colon,
            String::from("Expected type indicator"),
            String::from("Write the type indicator \":\"."),
            line,
        )?;

        let kind: (DataTypes, String) = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance()?;
                (*kind, String::new())
            }

            TokenKind::Identifier => {
                if self
                    .parser_objects
                    .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                    .is_ok()
                {
                    self.only_advance()?;

                    (DataTypes::Struct, self.previous().lexeme.clone().unwrap())
                } else {
                    return Err(ThrushError::Error(
                        String::from("Expected type"),
                        String::from("Write the type for the variable."),
                        line,
                        format!("var {}: str = \"\";", name.lexeme.as_ref().unwrap()),
                    ));
                }
            }

            _ => {
                return Err(ThrushError::Error(
                    String::from("Expected variable type"),
                    String::from("Write the type: \"var name: --> type <-- = value;\"."),
                    line,
                    format!("var {}: str = \"\";", name.lexeme.as_ref().unwrap()),
                ));
            }
        };

        if self.peek().kind == TokenKind::SemiColon && kind.0 == DataTypes::Void {
            self.only_advance()?;

            self.errors.push(ThrushError::Error(
                String::from("Missing Type"),
                String::from("Type is undefined. Write some type available."),
                line,
                format!("var {}: str;", name.lexeme.as_ref().unwrap()),
            ));
        } else if self.peek().kind == TokenKind::SemiColon {
            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
                line,
            )?;

            self.parser_objects.insert_new_local(
                self.scope,
                name.lexeme.as_ref().unwrap(),
                (kind.0, true, false, false, 0, kind.1),
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
            name.line,
        )?;

        self.at_typed_variable = kind.0;

        let value: Instruction<'instr> = self.expression()?;

        self.check_possible_type_mismatch(kind.0, value.get_data_type(), Some(&value), name.line);

        if self.at_typed_variable == DataTypes::Struct && value.get_data_type() == DataTypes::Struct
        {
            if let Instruction::Struct {
                name: struct_name, ..
            } = &value
            {
                self.check_possible_type_mismatch_for_struct_type(&kind.1, struct_name, line)?;
            }
        }

        self.parser_objects.insert_new_local(
            self.scope,
            name.lexeme.as_ref().unwrap(),
            (kind.0, false, false, false, 0, kind.1),
        );

        let var: Instruction<'_> = Instruction::Var {
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
            line,
        )?;

        Ok(var)
    }

    fn build_public_qualifier(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        match &self.peek().kind {
            TokenKind::Fn => Ok(self.build_function(true)?),
            TokenKind::Extern => Ok(self.build_external_qualifier()?),
            what => Err(ThrushError::Error(
                String::from("Syntax error"),
                format!("Public qualifier is not applicable for \"{}\".", what),
                self.peek().line,
                String::new(),
            )),
        }
    }

    fn build_return(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let line: usize = self.previous().line;

        if !self.inside_function {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "Return statement outside of function body. Invoke this, in function body.",
                ),
                line,
                String::from("fn mybuild_function(): i32 {\n return 0;\n}"),
            ));
        }

        if self.peek().kind == TokenKind::SemiColon {
            self.consume(
                TokenKind::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
                line,
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

        let value: Instruction<'instr> = self.expression()?;

        /* if self.at_typed_function.0 == DataTypes::Void && value.get_data_type() != DataTypes::Void {
            self.errors.push(ThrushError::Error(

                String::from("Syntax error"),
                format!("Missing function type indicator with type '{}', you should add a correct function type indicator with type '{}'.", value.get_data_type(), value.get_data_type()),
                line,
                String::new()
            ));
        }*/

        self.check_possible_type_mismatch(
            self.at_typed_function.0,
            value.get_data_type(),
            Some(&value),
            line,
        );

        if self.at_typed_function.0 == DataTypes::Struct
            && value.get_data_type() == DataTypes::Struct
        {
            if let Instruction::Struct {
                name: struct_name, ..
            } = &value
            {
                let mut inside_function_type_clone: String = String::new();

                inside_function_type_clone.clone_from(&self.at_typed_function.1);

                self.check_possible_type_mismatch_for_struct_type(
                    &inside_function_type_clone,
                    struct_name,
                    line,
                )?;
            }
        }

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
            line,
        )?;

        Ok(Instruction::Return(
            Box::new(value),
            self.at_typed_function.0,
        ))
    }

    fn build_block(
        &mut self,
        with_instrs: &mut [Instruction<'instr>],
        begin_scope: bool,
    ) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        if begin_scope {
            self.parser_objects.begin_local_scope();
        }

        let mut stmts: Vec<Instruction> = Vec::new();
        let mut was_emited_deallocators: bool = false;

        for instr in with_instrs.iter_mut() {
            stmts.push(mem::take(instr));
        }

        while !self.match_token(TokenKind::RBrace)? {
            let instr: Instruction<'instr> = self.parse()?;

            if instr.is_return() {
                let deallocators: Vec<Instruction<'_>> =
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

    fn build_function(&mut self, is_public: bool) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        if self.scope != 0 {
            self.errors.push(ThrushError::Error(

                String::from("Syntax error"),
                String::from(
                    "The build_functions must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                String::new(),
            ));
        }

        self.inside_function = true;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Expected function name"),
            String::from("Expected a name to the function."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        if name.lexeme.as_ref().unwrap() == "main" {
            if self.has_entry_point {
                self.errors.push(ThrushError::Error(
                    String::from("Duplicated EntryPoint"),
                    String::from("The language not support two entrypoints, remove one."),
                    line,
                    String::from("fn main() { ... }"),
                ));
            }

            self.consume(
                TokenKind::LParen,
                String::from("Syntax error"),
                String::from("Expected '('."),
                line,
            )?;

            self.consume(
                TokenKind::RParen,
                String::from("Syntax error"),
                String::from("Expected ')'."),
                line,
            )?;

            if self.peek().kind != TokenKind::LBrace {
                self.errors.push(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Expected '{'."),
                    line,
                    String::new(),
                ));
            }

            if self.peek().kind == TokenKind::LBrace {
                self.has_entry_point = true;

                return Ok(Instruction::EntryPoint {
                    body: Box::new(self.build_block(&mut [], true)?),
                });
            } else {
                self.errors.push(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Expected block \"({ ... })\" for the function body."),
                    self.peek().line,
                    String::new(),
                ));
            }
        }

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
            name.line,
        )?;

        let mut params: Vec<Instruction<'instr>> = Vec::new();

        self.parser_objects.begin_local_scope();

        let mut position: u32 = 0;

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Pass)? {
                continue;
            }

            if !self.match_token(TokenKind::Identifier)? {
                self.errors.push(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Expected argument name."),
                    line,
                    String::from("hello :: type, "),
                ));
            }

            let ident: &str = self.previous().lexeme.as_ref().unwrap();

            if !self.match_token(TokenKind::ColonColon)? {
                self.errors.push(ThrushError::Error(
                    String::from("Syntax error"),
                    String::from("Expected '::'."),
                    line,
                    format!("{} :: type, ", ident),
                ));
            }

            let kind: DataTypes = match &self.peek().kind {
                TokenKind::DataType(kind) => {
                    self.only_advance()?;
                    *kind
                }
                TokenKind::Identifier => {
                    if self
                        .parser_objects
                        .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                        .is_ok()
                    {
                        self.only_advance()?;
                        DataTypes::Struct
                    } else {
                        self.errors.push(ThrushError::Error(
                            String::from("Syntax error"),
                            format!(
                                "Expected valid argument type not \"{}\".",
                                self.peek().lexeme.as_ref().unwrap()
                            ),
                            line,
                            format!("{} :: str, ", ident),
                        ));

                        self.only_advance()?;
                        continue;
                    }
                }

                kind => {
                    self.errors.push(ThrushError::Error(
                        String::from("Syntax error"),
                        format!("Expected valid argument type not \"{}\".", kind),
                        line,
                        format!("{} :: str, ", ident),
                    ));

                    self.only_advance()?;

                    continue;
                }
            };

            self.parser_objects.insert_new_local(
                self.scope,
                ident,
                (kind, false, false, false, 0, String::new()),
            );

            params.push(Instruction::Param {
                name: ident.to_string(),
                kind,
                position,
                line,
            });

            position += 1;
        }

        if self.peek().kind == TokenKind::Colon {
            self.consume(
                TokenKind::Colon,
                String::from("Syntax error"),
                String::from(
                    "Missing return type indicator. Expected ':' followed by return type.",
                ),
                name.line,
            )?;
        }

        let return_type: Option<(DataTypes, String)> = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance()?;
                Some((*kind, String::new()))
            }

            TokenKind::Identifier => {
                if self
                    .parser_objects
                    .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                    .is_ok()
                {
                    self.only_advance()?;

                    Some((DataTypes::Struct, self.previous().lexeme.clone().unwrap()))
                } else {
                    None
                }
            }
            _ => None,
        };

        let return_type: (DataTypes, String) =
            return_type.unwrap_or((DataTypes::Void, String::new()));

        self.at_typed_function = return_type.clone();

        let mut function: Instruction<'_> = Instruction::Function {
            name: name.lexeme.clone().unwrap(),
            params: params.clone(),
            body: None,
            return_type: return_type.0,
            is_public,
        };

        if self.match_token(TokenKind::SemiColon)? {
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
        let mut instr: Instruction<'_> = self.and()?;

        while self.match_token(TokenKind::Or)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction<'instr> = self.and()?;

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
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
        let mut instr: Instruction<'_> = self.equality()?;

        while self.match_token(TokenKind::And)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction<'_> = self.equality()?;

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
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
        let mut instr: Instruction<'_> = self.comparison()?;

        while self.match_token(TokenKind::BangEq)? || self.match_token(TokenKind::EqEq)? {
            let op: &TokenKind = &self.previous().kind;
            let line: usize = self.previous().line;
            let right: Instruction<'_> = self.comparison()?;

            let left_type: DataTypes = instr.get_data_type_recursive();
            let right_type: DataTypes = right.get_data_type_recursive();

            type_checking::check_binary_instr(op, &left_type, &right_type, self.previous().line)?;

            instr.is_chained(&right, line)?;

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
        let mut instr: Instruction<'_> = self.term()?;

        while self.match_token(TokenKind::Greater)?
            || self.match_token(TokenKind::GreaterEq)?
            || self.match_token(TokenKind::Less)?
            || self.match_token(TokenKind::LessEq)?
        {
            let op: &TokenKind = &self.previous().kind;
            let line: usize = self.previous().line;
            let right: Instruction<'_> = self.term()?;

            let left_type: DataTypes = instr.get_data_type_recursive();
            let right_type: DataTypes = right.get_data_type_recursive();

            type_checking::check_binary_instr(op, &left_type, &right_type, self.previous().line)?;

            instr.is_chained(&right, line)?;

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
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction = self.factor()?;

            let left_type: DataTypes = instr.get_data_type_recursive();
            let right_type: DataTypes = right.get_data_type_recursive();

            let kind: DataTypes = if left_type.is_integer() && right_type.is_integer() {
                left_type.calculate_integer_datatype(right_type)
            } else if left_type.is_float() && right_type.is_float() {
                left_type.calculate_float_datatype(right_type)
            } else {
                self.at_typed_variable
            };

            type_checking::check_binary_instr(op, &left_type, &right_type, self.previous().line)?;

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
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

            let kind: DataTypes = if left_type.is_integer() && right_type.is_integer() {
                left_type.calculate_integer_datatype(right_type)
            } else if left_type.is_float() && right_type.is_float() {
                left_type.calculate_float_datatype(right_type)
            } else {
                self.at_typed_variable
            };

            type_checking::check_binary_instr(op, &left_type, &right_type, self.previous().line)?;

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

            type_checking::check_unary_instr(op, &value.get_data_type(), self.previous().line)?;

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

            let value_type: &DataTypes = &value.get_data_type();

            type_checking::check_unary_instr(op, value_type, self.previous().line)?;

            return Ok(Instruction::UnaryOp {
                op,
                value: Box::from(value),
                kind: *value_type,
            });
        }

        let instr: Instruction = self.primary()?;

        Ok(instr)
    }

    fn primary(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let primary: Instruction = match &self.peek().kind {
            TokenKind::New => self.build_struct_initializer()?,
            TokenKind::DataType(dt) => {
                let line: usize = self.advance()?.line;

                match dt {
                    dt if dt.is_integer() => Instruction::DataTypes(*dt),
                    dt if dt.is_float() => Instruction::DataTypes(*dt),
                    dt if dt == &DataTypes::Bool => Instruction::DataTypes(*dt),
                    dt if dt == &DataTypes::Ptr => Instruction::DataTypes(*dt),
                    what_heck_dt => {
                        return Err(ThrushError::Error(
                            String::from("Syntax error"),
                            format!(
                                "The type \"{}\" is not a machine-native type.",
                                what_heck_dt
                            ),
                            line,
                            String::new(),
                        ))
                    }
                }
            }
            TokenKind::LParen => {
                let line: usize = self.advance()?.line;
                let instr: Instruction = self.expression()?;
                let kind: DataTypes = instr.get_data_type();

                if !instr.is_binary() && !instr.is_group() {
                    self.errors.push(ThrushError::Error(
                        String::from("Syntax error"),
                        String::from(
                            "Group the expressions \"(...)\" is only allowed if contain binary expressions or other group expressions.",
                        ),
                        line,
                        String::from("(T + T) or ((T + T))")
                    ));
                }

                self.consume(
                    TokenKind::RParen,
                    String::from("Syntax error"),
                    String::from("Expected ')'."),
                    line,
                )?;

                return Ok(Instruction::Group {
                    instr: Box::new(instr),
                    kind,
                });
            }
            TokenKind::Str => {
                Instruction::Str(self.advance()?.lexeme.as_ref().unwrap().to_string())
            }
            TokenKind::Char => {
                Instruction::Char(self.advance()?.lexeme.as_ref().unwrap().as_bytes()[0])
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
                    let identifier_lexeme: Option<&String> = self.peek().lexeme.as_ref();
                    let identifier_type: TokenKind = self.peek().kind;
                    let line: usize = self.peek().line;

                    let object: FoundObject = self
                        .parser_objects
                        .get_object(identifier_lexeme.unwrap(), line)?;

                    let name: &str = identifier_lexeme.unwrap();

                    self.only_advance()?;

                    if self.match_token(TokenKind::Eq)? {
                        let expr: Instruction = self.expression()?;

                        self.check_possible_type_mismatch(
                            object.0,
                            expr.get_data_type(),
                            Some(&expr),
                            line,
                        );

                        self.consume(
                            TokenKind::SemiColon,
                            String::from("Syntax error"),
                            String::from("Expected ';'."),
                            line,
                        )?;

                        self.parser_objects.insert_new_local(
                            self.scope,
                            name,
                            (object.0, false, false, false, 0, String::new()),
                        );

                        return Ok(Instruction::MutVar {
                            name,
                            value: Box::new(expr),
                            kind: object.0,
                        });
                    } else if self.match_token(TokenKind::LParen)? {
                        return self.call(name, object, line);
                    }

                    if object.1 {
                        self.errors.push(ThrushError::Error(
                            String::from("Undefined variable usage"),
                            format!(
                                "Variable `{}` is not defined for are use it. Define the variable with value before of the use.",
                                name,
                            ),
                            line,
                            String::new()
                        ));
                    }

                    let refvar: Instruction = Instruction::RefVar {
                        name,
                        line,
                        kind: object.0,
                    };

                    if self.match_token(TokenKind::PlusPlus)?
                        | self.match_token(TokenKind::MinusMinus)?
                    {
                        let op: &TokenKind = &self.previous().kind;

                        type_checking::check_unary_instr(
                            &identifier_type,
                            &refvar.get_data_type(),
                            line,
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
                    self.only_advance()?;

                    let previous: &Token = self.previous();

                    return Err(ThrushError::Error(
                        String::from("Syntax error"),
                        format!(
                            "Statement \"{}\" don't allowed.",
                            previous.lexeme.as_ref().unwrap(),
                        ),
                        previous.line,
                        String::new(),
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
        line: usize,
    ) -> Result<Instruction<'instr>, ThrushError> {
        if !object.3 {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "The object called is don't a function. Call is only allowed for functions.",
                ),
                line,
                String::from("fn mybuild_function() { ... }\n fn main() { mybuild_function() }"),
            ));
        }

        let mut args: Vec<Instruction> = Vec::new();

        while self.peek().kind != TokenKind::RParen {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            args.push(self.expression()?);
        }

        self.consume(
            TokenKind::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
            line,
        )?;

        if args.len() > object.5.len() && !object.4 {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Function called with more arguments than expected."),
                line,
                String::new(),
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
                line,
                String::new(),
            ));
        }

        if !object.4 {
            for (index, argument) in args.iter().enumerate() {
                let argument_type: DataTypes = argument.get_data_type();
                let target: DataTypes = object.5[index];

                self.check_possible_type_mismatch(target, argument_type, Some(argument), line);

                if target == DataTypes::Struct {
                    if let Instruction::Struct { name, .. } = argument {
                        let original_struct_type: &(String, usize) =
                            object.6.iter().find(|obj| obj.1 == index).unwrap();
                        self.check_possible_type_mismatch_for_struct_type(
                            &original_struct_type.0,
                            name,
                            line,
                        )?;
                    }

                    if let Instruction::RefVar { name, .. } = argument {
                        let original_struct_type: &(String, usize) =
                            object.6.iter().find(|obj| obj.1 == index).unwrap();

                        let refvar_struct_type: FoundObject =
                            self.parser_objects.get_object(name, line).unwrap();

                        self.check_possible_type_mismatch_for_struct_type(
                            &original_struct_type.0,
                            &refvar_struct_type.7,
                            line,
                        )?;
                    }
                }
            }
        }

        Ok(Instruction::Call {
            name,
            args,
            kind: object.0,
        })
    }

    /* ######################################################################


        PARSER - FUNCTIONS & STRUCTS PRE-DECLARATION


    ########################################################################*/

    #[inline]
    fn is_external_qualifier(&mut self) -> bool {
        if self.current < 4 {
            false
        } else {
            self.tokens[self.current - 4].kind == TokenKind::Extern
        }
    }

    #[inline]
    fn is_public_qualifier(&mut self) -> bool {
        if self.current < 2 {
            false
        } else {
            self.tokens[self.current - 2].kind == TokenKind::Extern
        }
    }

    fn start_predeclaration(&mut self) {
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
            .filter(|(_, kind)| *kind == TokenKind::Struct)
            .for_each(|(pos, _)| {
                let _ = self.predeclare_struct(*pos);
            });

        declarations
            .iter()
            .filter(|(_, kind)| *kind == TokenKind::Fn)
            .for_each(|(pos, _)| {
                let _ = self.predeclare_function(*pos);
            });
    }

    fn predeclare_struct(&mut self, position: usize) -> Result<(), ThrushError> {
        self.current = position;

        let is_public_qualifer: bool = self.is_public_qualifier();

        if is_public_qualifer {
            while self.peek().kind != TokenKind::Struct {
                self.only_advance()?;
            }
        }

        self.only_advance()?;

        if self.scope != 0 {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "The structs must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                String::new(),
            ));
        }

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Expected struct name"),
            String::from("Write the struct name: \"struct --> name <-- { ... };\"."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
            line,
        )?;

        let mut fields_types: HashMap<String, DataTypes> = HashMap::new();

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_name: String = self.previous().lexeme.clone().unwrap();
                let line: usize = self.previous().line;

                let field_type: DataTypes = match &self.peek().kind {
                    TokenKind::DataType(kind) => {
                        self.only_advance()?;
                        *kind
                    }
                    TokenKind::Identifier => {
                        if self
                            .parser_objects
                            .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                            .is_ok()
                            || name.lexeme.as_ref().unwrap() == self.peek().lexeme.as_ref().unwrap()
                        {
                            self.only_advance()?;
                            DataTypes::Struct
                        } else {
                            return Err(ThrushError::Error(
                                String::from("Expected type of field"),
                                format!("Write the field type: \"{} --> i64 <--\".", field_name),
                                line,
                                format!("struct {} {{\n   {} i64\n  }}", field_name, field_name),
                            ));
                        }
                    }
                    _ => {
                        return Err(ThrushError::Error(
                            String::from("Expected type of field"),
                            format!("Write the field type: \"{} --> i64 <--\".", field_name),
                            line,
                            format!("struct {} {{\n   {} i64\n  }}", field_name, field_name),
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
            line,
        )?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
            line,
        )?;

        self.current = 0;

        if self
            .parser_objects
            .structs
            .contains_key(name.lexeme.as_ref().unwrap())
        {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from("The struct already exists."),
                line,
                String::new(),
            ));
        }

        self.parser_objects
            .insert_new_struct(name.lexeme.clone().unwrap(), fields_types);

        Ok(())
    }

    fn predeclare_function(&mut self, position: usize) -> Result<(), ThrushError> {
        self.current = position;

        let is_external_qualifier: bool = self.is_external_qualifier();

        if is_external_qualifier {
            while self.peek().kind != TokenKind::Fn {
                self.only_advance()?;
            }
        }

        let mut ignore_more_params: bool = false;

        self.only_advance()?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Expected function name"),
            String::from("Expected fn < name >."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::LParen,
            String::from("Syntax error"),
            String::from("Expected '('."),
            line,
        )?;

        let mut params: Vec<DataTypes> = Vec::new();
        let mut parameters_struct_types: Vec<(String, usize)> = Vec::new();
        let mut parameter_position: usize = 0;

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Pass)? {
                ignore_more_params = true;
                continue;
            }

            self.match_token(TokenKind::Identifier)?;
            self.match_token(TokenKind::ColonColon)?;

            let kind: DataTypes = match &self.peek().kind {
                TokenKind::DataType(kind) => {
                    self.only_advance()?;
                    *kind
                }
                TokenKind::Identifier => {
                    if self
                        .parser_objects
                        .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                        .is_ok()
                    {
                        parameters_struct_types
                            .push((self.peek().lexeme.clone().unwrap(), parameter_position));
                        self.only_advance()?;
                        DataTypes::Struct
                    } else {
                        self.only_advance()?;
                        continue;
                    }
                }
                _ => {
                    self.only_advance()?;
                    continue;
                }
            };

            params.push(kind);

            parameter_position += 1;
        }

        if ignore_more_params && !is_external_qualifier {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "Ignore statement \"...\" in functions is only allowed for external functions.",
                ),
                line,
                String::new(),
            ));
        }

        if self.peek().kind == TokenKind::Colon {
            self.consume(
                TokenKind::Colon,
                String::from("Syntax error"),
                String::from("Missing return type. Expected ':' followed by return type."),
                line,
            )?;
        }

        let return_type: DataTypes = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance()?;
                *kind
            }
            _ => DataTypes::Void,
        };

        self.current = 0;

        self.parser_objects.insert_new_global(
            name.lexeme.clone().unwrap(),
            (
                return_type,
                params,
                parameters_struct_types,
                true,
                ignore_more_params,
            ),
        );

        Ok(())
    }

    /* ######################################################################


        PARSER - HELPERS


    ########################################################################*/

    fn check_struct(&mut self) -> Result<(), ThrushError> {
        self.only_advance()?;

        if self.scope != 0 {
            self.errors.push(ThrushError::Error(
                String::from("Syntax error"),
                String::from(
                    "The structs must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                String::new(),
            ));
        }

        let name: &Token = self.consume(
            TokenKind::Identifier,
            String::from("Expected struct name"),
            String::from("Write the struct name: \"struct --> name <-- { ... };\"."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::LBrace,
            String::from("Syntax error"),
            String::from("Expected '{'."),
            line,
        )?;

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if self.match_token(TokenKind::Identifier)? {
                let field_name: String = self.previous().lexeme.clone().unwrap();
                let line: usize = self.previous().line;

                match &self.peek().kind {
                    TokenKind::DataType(kind) => {
                        self.only_advance()?;
                        *kind
                    }
                    TokenKind::Identifier => {
                        if self
                            .parser_objects
                            .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                            .is_ok()
                            || name.lexeme.as_ref().unwrap() == self.peek().lexeme.as_ref().unwrap()
                        {
                            self.only_advance()?;
                            DataTypes::Struct
                        } else {
                            return Err(ThrushError::Error(
                                String::from("Expected type of field"),
                                format!("Write the field type: \"{} --> i64 <--\".", field_name),
                                line,
                                format!("struct {} {{\n   {} i64\n  }}", field_name, field_name),
                            ));
                        }
                    }
                    _ => {
                        return Err(ThrushError::Error(
                            String::from("Expected type of field"),
                            format!("Write the field type: \"{} --> i64 <--\".", field_name),
                            line,
                            format!("struct {} {{\n   {} i64\n  }}", field_name, field_name),
                        ));
                    }
                };

                continue;
            }

            self.only_advance()?;
        }

        self.consume(
            TokenKind::RBrace,
            String::from("Syntax error"),
            String::from("Expected '}'."),
            line,
        )?;

        self.consume(
            TokenKind::SemiColon,
            String::from("Syntax error"),
            String::from("Expected ';'."),
            line,
        )?;

        Ok(())
    }

    fn check_possible_type_mismatch_for_struct_type(
        &self,
        target: &str,
        from: &str,
        line: usize,
    ) -> Result<(), ThrushError> {
        if target.trim().to_lowercase() != from.trim().to_lowercase() {
            return Err(ThrushError::Error(
                String::from("Mismatched Types"),
                format!("Struct '{}' and '{}' are not the same type.", target, from),
                line,
                String::new(),
            ));
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
                    line,
                    String::from("Mismatched types"),
                    format!(
                        "Mismatched type. Expected '{}' but found '{}'.",
                        target, from
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
            line,
            String::from("Mismatched types"),
            format!(
                "Mismatched type. Expected '{}' but found '{}'.",
                target, from
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
        line: usize,
    ) -> Result<&'instr Token, ThrushError> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushError::Error(error_title, help, line, String::new()))
    }

    fn check_kind(&self, other_type: TokenKind) -> bool {
        if self.end() {
            return false;
        }

        self.peek().kind == other_type
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
            self.previous().line,
            String::new(),
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
            self.previous().line,
            String::new(),
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

    #[inline]
    fn peek(&self) -> &'instr Token {
        &self.tokens[self.current]
    }

    #[inline]
    fn previous(&self) -> &'instr Token {
        &self.tokens[self.current - 1]
    }

    #[inline]
    fn end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}
