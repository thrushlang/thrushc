use {
    super::{
        super::{
            backend::{compiler::options::ThrushFile, instruction::Instruction},
            diagnostic::Diagnostic,
            error::{ThrushError, ThrushErrorKind},
            logging::LogType,
            CORE_LIBRARY_PATH,
        },
        lexer::{DataTypes, Lexer, Token, TokenKind},
        objects::{Globals, ParserObjects},
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

type ParserObject = (
    DataTypes,      // Main Type
    bool,           // is null?
    bool,           // is freeded?
    bool,           // is function?
    bool,           // ignore the params if is a function?
    Vec<DataTypes>, // params types
    usize,          // Number the references
);

pub struct Parser<'instr> {
    stmts: Vec<Instruction<'instr>>,
    errors: Vec<ThrushError>,
    tokens: &'instr [Token],
    in_function: bool,
    in_type_function: (DataTypes, String),
    in_var_type: DataTypes,
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
            (DataTypes::I64, Vec::from([DataTypes::Ptr]), true, false),
        );

        Self {
            stmts: Vec::new(),
            errors: Vec::new(),
            tokens,
            current: 0,
            in_function: false,
            in_type_function: (DataTypes::Void, String::new()),
            in_var_type: DataTypes::Void,
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
            ThrushErrorKind::SyntaxError,
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
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
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
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Too many fields in struct"),
                        String::from("There are more fields in the structure than normal, they must be the exact amount."),
                        line,
                        String::new(),
                    ));
                }

                if !struct_found.contains_key(&field_name) {
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Struct field name not found"),
                        String::from("Write valid field name in the struct initialization."),
                        line,
                        String::new(),
                    ));
                }

                let line: usize = self.previous().line;

                let expr: Instruction = self.expression()?;
                let field_type: DataTypes = expr.get_data_type();

                let target_type: &DataTypes = struct_found.get(&field_name).unwrap();

                self.check_types(*target_type, field_type, &expr, line);

                fields.push((field_name, expr, *target_type, count));

                count += 1;

                continue;
            }

            self.only_advance()?;
        }

        self.consume(
            TokenKind::RBrace,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
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
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "The imports must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                String::new(),
            ));
        }

        self.only_advance()?;

        let line: usize = self.previous().line;

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
            line,
        )?;

        let path: &str = self
            .consume(
                TokenKind::Str,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected a String literal for @import(\"PATH\")."),
                line,
            )?
            .lexeme
            .as_ref()
            .unwrap();

        self.consume(
            TokenKind::RParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ')'."),
            line,
        )?;

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            line,
        )?;

        let path_converted: &Path = Path::new(path);

        if path.starts_with("core")
            && path.split("").filter(|c| *c == ".").count() >= 1
            && !path_converted.exists()
        {
            if !CORE_LIBRARY_PATH.contains_key(path) {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Import Error"),
                    String::from("This module not exist in Thrush Core Library."),
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
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Import Error"),
                    String::from("A path to directory is don't able to import."),
                    line,
                    String::from("@import(\"dir/to/file.th\");"),
                ));
                return Ok(());
            }

            if path_converted.extension().is_none() {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Import Error"),
                    String::from("The file should contain a extension (*.th)."),
                    line,
                    String::from("@import(\"only/th/extension/file.th\");"),
                ));
                return Ok(());
            }

            if path_converted.extension().unwrap() != "th" {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Import Error"),
                    String::from("Only thrush files (*.th) are allowed to been imported."),
                    line,
                    String::from("@import(\"only/thrush/files/file.th\");"),
                ));
                return Ok(());
            }

            if path_converted.file_name().is_none() {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Import Error"),
                    String::from("The file should contain a name (<mythfile>.th)."),
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

        Err(ThrushError::Parse(
            ThrushErrorKind::SyntaxError,
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
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
            line,
        )?;

        let name: &Token = self.consume(
            TokenKind::Str,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected a string literal for @extern(\"NAME\")."),
            line,
        )?;

        self.consume(
            TokenKind::RParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ')'."),
            line,
        )?;

        let instr: Instruction<'instr> = match self.peek().kind {
            TokenKind::Fn => self.build_function(true)?,
            what => {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
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

        let start_line: usize = self.previous().line;

        let variable: Instruction<'instr> = self.build_local_variable(false)?;

        let cond: Instruction<'instr> = self.expression()?;

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            start_line,
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

        if !self.check_kind(TokenKind::RBrace) {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected body \"{ ... }\" for the loop."),
                start_line,
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
            ThrushErrorKind::SyntaxError,
            String::from("Expected variable name"),
            String::from("Write the name: \"var --> name <-- : type = value;\"."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::Colon,
            ThrushErrorKind::SyntaxError,
            String::from("Expected variable type indicator"),
            String::from("Write the type indicator: \"var name --> : <-- type = value;\"."),
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
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Expected variable type"),
                        String::from("Write the type: \"var name: --> type <-- = value;\"."),
                        line,
                        format!("var {}: str = \"\";", name.lexeme.as_ref().unwrap()),
                    ));
                }
            }

            _ => {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Expected variable type"),
                    String::from("Write the type: \"var name: --> type <-- = value;\"."),
                    line,
                    format!("var {}: str = \"\";", name.lexeme.as_ref().unwrap()),
                ));
            }
        };

        if self.peek().kind == TokenKind::SemiColon && kind.0 == DataTypes::Void {
            self.only_advance()?;

            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Variable don't declared without type"),
                String::from(
                    "Variable type is undefined. Did you forget to specify the variable type to undefined variable? Like \"var thrush: --> String <--;\".",
                ),
                line,
                format!("var {}: str;", name.lexeme.as_ref().unwrap())
            ));
        } else if self.peek().kind == TokenKind::SemiColon {
            self.consume(
                TokenKind::SemiColon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ';'."),
                line,
            )?;

            self.parser_objects.insert_new_local(
                self.scope,
                name.lexeme.as_ref().unwrap(),
                (kind.0, true, false, false, 0),
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
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '='."),
            name.line,
        )?;

        self.in_var_type = kind.0;

        let value: Instruction<'instr> = self.expression()?;

        self.check_types(kind.0, value.get_data_type(), &value, name.line);

        if self.in_var_type == DataTypes::Struct && value.get_data_type() == DataTypes::Struct {
            if let Instruction::Struct {
                name: struct_name, ..
            } = &value
            {
                self.check_types_for_struct(kind.1, struct_name, line)?;
            }
        }

        self.parser_objects.insert_new_local(
            self.scope,
            name.lexeme.as_ref().unwrap(),
            (kind.0, false, false, false, 0),
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
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
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
            what => Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                format!("Public qualifier is not applicable for \"{}\" .", what),
                self.peek().line,
                String::new(),
            )),
        }
    }

    fn build_return(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let line: usize = self.previous().line;

        if !self.in_function {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Return statement outside of build_function. Invoke this keyword in scope of function."),
                line,
                String::from("fn mybuild_function(): i32 {\n return 0;\n}")
            ));
        }

        if self.peek().kind == TokenKind::SemiColon {
            self.consume(
                TokenKind::SemiColon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ';'."),
                line,
            )?;

            if self.in_type_function.0 != DataTypes::Void {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    format!("Missing return statement with correctly type '{}', you should rewrite for return with type '{}'.", self.in_type_function.0, self.in_type_function.0),
                    line,
                    String::new()
                ));
            }

            return Ok(Instruction::Return(
                Box::new(Instruction::Null),
                DataTypes::Void,
            ));
        }

        let value: Instruction<'instr> = self.expression()?;

        if self.in_type_function.0 == DataTypes::Void && value.get_data_type() != DataTypes::Void {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                format!("Missing function type indicator with type '{}', you should add a correct function type indicator with type '{}'.", value.get_data_type(), value.get_data_type()),
                line,
                String::new()
            ));
        }

        self.check_types(self.in_type_function.0, value.get_data_type(), &value, line);

        if self.in_type_function.0 == DataTypes::Struct
            && value.get_data_type() == DataTypes::Struct
        {
            if let Instruction::Struct {
                name: struct_name, ..
            } = &value
            {
                let mut in_function_type_clone: String = String::new();

                in_function_type_clone.clone_from(&self.in_type_function.1);

                self.check_types_for_struct(in_function_type_clone, struct_name, line)?;
            }
        }

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            line,
        )?;

        Ok(Instruction::Return(
            Box::new(value),
            self.in_type_function.0,
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
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "The build_functions must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                String::new(),
            ));
        }

        self.in_function = true;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            ThrushErrorKind::SyntaxError,
            String::from("Expected function name"),
            String::from("Expected a name to the function."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        if name.lexeme.as_ref().unwrap() == "main" {
            if self.has_entry_point {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Duplicated EntryPoint"),
                    String::from("The language not support two entrypoints, remove one."),
                    line,
                    String::from("fn main() { ... }"),
                ));
            }

            self.consume(
                TokenKind::LParen,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected '('."),
                line,
            )?;

            self.consume(
                TokenKind::RParen,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ')'."),
                line,
            )?;

            if self.peek().kind != TokenKind::LBrace {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
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
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected block \"({ ... })\" for the function body."),
                    self.peek().line,
                    String::new(),
                ));
            }
        }

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
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
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected argument name."),
                    line,
                    String::from("hello :: type, "),
                ));
            }

            let ident: &str = self.previous().lexeme.as_ref().unwrap();

            if !self.match_token(TokenKind::ColonColon)? {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
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
                kind => {
                    self.errors.push(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Syntax Error"),
                        format!("Expected valid argument type not \"{}\".", kind),
                        line,
                        format!("{} :: str, ", ident),
                    ));

                    self.only_advance()?;

                    continue;
                }
            };

            self.parser_objects
                .insert_new_local(self.scope, ident, (kind, false, false, false, 0));

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
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
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

        self.in_type_function = return_type.clone();

        let mut function: Instruction<'_> = Instruction::Function {
            name: name.lexeme.clone().unwrap(),
            params: params.clone(),
            body: None,
            return_type: return_type.0,
            is_public,
        };

        if self.match_token(TokenKind::SemiColon)? {
            self.in_function = false;
            return Ok(function);
        }

        let body: Box<Instruction> = Box::new(self.build_block(&mut params, false)?);

        self.in_function = false;

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
                self.in_var_type
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
                self.in_var_type
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
                self.only_advance()?;

                let line: usize = self.previous().line;

                match dt {
                    dt if dt.is_integer() => Instruction::DataTypes(*dt),
                    dt if dt.is_float() => Instruction::DataTypes(*dt),
                    dt if dt == &DataTypes::Bool => Instruction::DataTypes(*dt),
                    dt if dt == &DataTypes::Ptr => Instruction::DataTypes(*dt),
                    what_heck_dt => {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
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
                let line: usize = self.peek().line;

                self.only_advance()?;

                let instr: Instruction = self.expression()?;
                let kind: DataTypes = instr.get_data_type();

                if !instr.is_binary() && !instr.is_group() {
                    self.errors.push(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Syntax Error"),
                        String::from(
                            "Group the expressions \"(...)\" is only allowed if contain binary expressions or other group expressions.",
                        ),
                        line,
                        String::from("(T + T) or ((T + T))")
                    ));
                }

                self.consume(
                    TokenKind::RParen,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
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

                    let instr: Instruction = Instruction::Integer(*kind, *num, *is_signed);

                    if self.match_token(TokenKind::PlusPlus)?
                        | self.match_token(TokenKind::MinusMinus)?
                    {
                        type_checking::check_unary_instr(
                            &self.previous().kind,
                            kind,
                            self.previous().line,
                        )?;

                        return Ok(Instruction::UnaryOp {
                            op: &self.previous().kind,
                            value: Box::from(instr),
                            kind: *kind,
                        });
                    }

                    instr
                }

                TokenKind::Float(kind, num, is_signed) => {
                    self.only_advance()?;

                    let instr: Instruction = Instruction::Float(*kind, *num, *is_signed);

                    if self.match_token(TokenKind::PlusPlus)?
                        | self.match_token(TokenKind::MinusMinus)?
                    {
                        type_checking::check_unary_instr(
                            &self.previous().kind,
                            kind,
                            self.previous().line,
                        )?;

                        return Ok(Instruction::UnaryOp {
                            op: &self.previous().kind,
                            value: Box::from(instr),
                            kind: *kind,
                        });
                    }

                    instr
                }

                TokenKind::Identifier => {
                    let current: &Token = self.peek();
                    let line: usize = self.peek().line;

                    let object: ParserObject = self
                        .parser_objects
                        .get_object(current.lexeme.as_ref().unwrap(), line)?;

                    let name: &str = current.lexeme.as_ref().unwrap();

                    self.only_advance()?;

                    /* if self.peek().kind == TokenKind::LBracket {
                        self.consume(
                            TokenKind::LBracket,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected '['."),
                            line,
                        )?;

                        let expr: Instruction = self.primary()?;

                        self.consume(
                            TokenKind::RBracket,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ']'."),
                            line,
                        )?;

                        if object.1 {
                            self.errors.push(ThrushError::Parse(
                                ThrushErrorKind::VariableNotDeclared,
                                String::from("Variable Not Declared"),
                                format!(
                                    "Variable `{}` is not declared for are use it. Declare the variable before of the use.",
                                    self.previous().lexeme.as_ref().unwrap(),
                                ),
                                line,
                                String::new()
                            ));
                        }

                        let kind: DataTypes = if object.0 == DataTypes::String {
                            DataTypes::Char
                        } else {
                            todo!()
                        };

                        if let Instruction::Integer(_, num, _) = expr {
                            return Ok(Instruction::Indexe {
                                origin: name,
                                index: num as u64,
                                kind,
                            });
                        }

                        self.errors.push(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected unsigned number for the build an indexe."),
                            self.previous().line,
                            String::new(),
                        ));
                    } else */
                    if self.match_token(TokenKind::Eq)? {
                        let expr: Instruction = self.expression()?;

                        self.check_types(object.0, expr.get_data_type(), &expr, line);

                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                            line,
                        )?;

                        self.parser_objects.insert_new_local(
                            self.scope,
                            name,
                            (object.0, false, false, false, 0),
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
                        self.errors.push(ThrushError::Parse(
                            ThrushErrorKind::VariableNotDeclared,
                            String::from("Variable not declared"),
                            format!(
                                "Variable `{}` is not declared for are use it. Declare the variable before of the use.",
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
                            &current.kind,
                            &refvar.get_data_type(),
                            line,
                        )?;

                        let expr: Instruction = Instruction::UnaryOp {
                            op,
                            value: Box::from(refvar),
                            kind: DataTypes::I64,
                        };

                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                            line,
                        )?;

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

                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Syntax Error"),
                        format!(
                            "Statement \"{}\" don't allowed.",
                            self.previous().lexeme.as_ref().unwrap(),
                        ),
                        self.previous().line,
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
        object: ParserObject,
        line: usize,
    ) -> Result<Instruction<'instr>, ThrushError> {
        if !object.3 {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "The object called is don't a function. Call is only allowed for functions.",
                ),
                line,
                String::from("fn mybuild_function() { ... }\n fn main() { mybuild_function() }"),
            ));
        }

        let mut args: Vec<Instruction<'instr>> = Vec::new();

        while self.peek().kind != TokenKind::RParen {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            args.push(self.expression()?);
        }

        self.consume(
            TokenKind::RParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ')'."),
            line,
        )?;

        if args.len() > object.5.len() && !object.4 {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Function called with more arguments than expected."),
                line,
                String::new(),
            ));
        }

        if object.5.len() != args.len() && !object.4 {
            let args_types: String = if !args.is_empty() {
                args.iter()
                    .map(|param| param.get_data_type().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                DataTypes::Void.to_string()
            };

            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                format!(
                    "Function called expected all arguments with types '{}' don't '{}'.",
                    object
                        .5
                        .iter()
                        .map(|param| param.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    args_types,
                ),
                line,
                String::new(),
            ));
        }

        if !object.4 {
            let mut index: usize = 0;

            args.iter().for_each(|argument| {
                let argument_type: DataTypes = argument.get_data_type();

                self.check_types(args[index].get_data_type(), argument_type, argument, line);

                index += 1;
            });
        }

        Ok(Instruction::Call {
            name,
            args,
            kind: object.0,
        })
    }

    /* ######################################################################


        PARSER - FUNCTIONS and STRUCTS PREDECLARATION


    ########################################################################*/

    fn is_external_qualifier(&mut self) -> bool {
        if self.current < 4 {
            false
        } else {
            self.tokens[self.current - 4].kind == TokenKind::Extern
        }
    }

    fn is_public_qualifier(&mut self) -> bool {
        if self.current < 2 {
            false
        } else {
            self.tokens[self.current - 2].kind == TokenKind::Extern
        }
    }

    fn start_predeclaration(&mut self) {
        let mut functions_positions: Vec<usize> = Vec::new();
        let mut structs_positions: Vec<usize> = Vec::new();

        let mut position: usize = 0;

        self.tokens.iter().for_each(|tok| {
            if let TokenKind::Fn = tok.kind {
                functions_positions.push(position);
                position += 1;
            } else if let TokenKind::Struct = tok.kind {
                structs_positions.push(position);
                position += 1;
            } else {
                position += 1;
            }
        });

        structs_positions.iter().for_each(|position| {
            let _ = self.predeclare_struct(*position);
        });

        functions_positions.iter().for_each(|position| {
            let _ = self.predeclare_function(*position);
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
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "The structs must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                String::new(),
            ));
        }

        let name: &Token = self.consume(
            TokenKind::Identifier,
            ThrushErrorKind::SyntaxError,
            String::from("Expected struct name"),
            String::from("Write the struct name: \"struct --> name <-- { ... };\"."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::LBrace,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
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
                            return Err(ThrushError::Parse(
                                ThrushErrorKind::SyntaxError,
                                String::from("Expected type of field"),
                                format!("Write the field type: \"{} --> i64 <--\".", field_name),
                                line,
                                format!("struct {} {{\n   {} i64\n  }}", field_name, field_name),
                            ));
                        }
                    }
                    _ => {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
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
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '}'."),
            line,
        )?;

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            line,
        )?;

        self.current = 0;

        if self
            .parser_objects
            .structs
            .contains_key(name.lexeme.as_ref().unwrap())
        {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
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
            ThrushErrorKind::SyntaxError,
            String::from("Expected function name"),
            String::from("Expected fn < name >."),
            self.previous().line,
        )?;

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
            name.line,
        )?;

        let mut params: Vec<DataTypes> = Vec::new();

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
                _ => {
                    self.only_advance()?;

                    continue;
                }
            };

            params.push(kind)
        }

        if ignore_more_params && !is_external_qualifier {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "Ignore statement \"...\" in functions is only allowed for external functions.",
                ),
                name.line,
                String::new(),
            ));
        }

        if self.peek().kind == TokenKind::Colon {
            self.consume(
                TokenKind::Colon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Missing return type. Expected ':' followed by return type."),
                name.line,
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
            (return_type, params, true, ignore_more_params),
        );

        Ok(())
    }

    /* ######################################################################


        PARSER - HELPERS


    ########################################################################*/

    fn check_struct(&mut self) -> Result<(), ThrushError> {
        self.only_advance()?;

        if self.scope != 0 {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "The structs must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                String::new(),
            ));
        }

        let name: &Token = self.consume(
            TokenKind::Identifier,
            ThrushErrorKind::SyntaxError,
            String::from("Expected struct name"),
            String::from("Write the struct name: \"struct --> name <-- { ... };\"."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::LBrace,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
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
                            return Err(ThrushError::Parse(
                                ThrushErrorKind::SyntaxError,
                                String::from("Expected type of field"),
                                format!("Write the field type: \"{} --> i64 <--\".", field_name),
                                line,
                                format!("struct {} {{\n   {} i64\n  }}", field_name, field_name),
                            ));
                        }
                    }
                    _ => {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
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
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '}'."),
            line,
        )?;

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            line,
        )?;

        Ok(())
    }

    fn check_types_for_struct(
        &mut self,
        target: String,
        from: &str,
        line: usize,
    ) -> Result<(), ThrushError> {
        if target.trim().to_lowercase() != from.trim().to_lowercase() {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Mismatched Types"),
                format!("Structs '{}' and '{}' are not the same.", target, from),
                line,
                String::new(),
            ));
        }

        Ok(())
    }

    fn check_types(
        &mut self,
        target: DataTypes,
        value_type: DataTypes,
        value: &Instruction,
        line: usize,
    ) {
        if value.is_binary() || value.is_group() {
            if let Err(err) = type_checking::check_types(
                target,
                None,
                Some(value),
                None,
                line,
                String::from("Type Mismatch"),
                format!(
                    "Type mismatch. Expected '{}' but found '{}'.",
                    target, value_type
                ),
            ) {
                self.errors.push(err);
                return;
            }
        }

        if let Err(e) = type_checking::check_types(
            target,
            Some(value_type),
            None,
            None,
            line,
            String::from("Type Mismatch"),
            format!(
                "Type mismatch. Expected '{}' but found '{}'.",
                target, value_type
            ),
        ) {
            self.errors.push(e);
        }
    }

    fn consume(
        &mut self,
        kind: TokenKind,
        error_kind: ThrushErrorKind,
        error_title: String,
        help: String,
        line: usize,
    ) -> Result<&'instr Token, ThrushError> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushError::Parse(
            error_kind,
            error_title,
            help,
            line,
            String::new(),
        ))
    }

    fn check_kind(&self, other_type: TokenKind) -> bool {
        if self.end() {
            return false;
        }

        self.peek().kind == other_type
    }

    fn match_token(&mut self, kind: TokenKind) -> Result<bool, ThrushError> {
        if self.end() {
            return Ok(false);
        } else if self.peek().kind == kind {
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

        Err(ThrushError::Parse(
            ThrushErrorKind::SyntaxError,
            String::from("Undeterminated Code"),
            String::from("The code has ended abruptly and without any order, review the code and write the syntax correctly."),
            self.previous().line,
            String::new()
        ))
    }

    fn advance(&mut self) -> Result<&'instr Token, ThrushError> {
        if !self.end() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(ThrushError::Parse(
            ThrushErrorKind::SyntaxError,
            String::from("Undeterminated Code"),
            String::from("The code has ended abruptly and without any order, review the code and write the syntax correctly."),
            self.previous().line,
            String::new()
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
