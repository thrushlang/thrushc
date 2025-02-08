use {
    super::{
        super::{
            backend::{compiler::options::ThrushFile, instruction::Instruction},
            diagnostic::Diagnostic,
            error::{ThrushError, ThrushErrorKind},
            logging::LogType,
            CORE_LIBRARY_PATH,
        }, preproccesadors::Import, lexer::{DataTypes, Lexer, Token, TokenKind}, objects::ParserObjects, scoper::ThrushScoper, type_checking
    }, std::{fs, mem, path::{Path, PathBuf}, process}
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
    in_type_function: DataTypes,
    in_var_type: DataTypes,
    current: usize,
    scope: usize,
    has_entry_point: bool,
    is_main: bool,
    scoper: ThrushScoper<'instr>,
    diagnostic: Diagnostic,
    parser_objects: ParserObjects<'instr>,
}

impl<'instr> Parser<'instr> {
    pub fn new(tokens: &'instr [Token], file: &'instr ThrushFile) -> Self {
        Self {
            stmts: Vec::new(),
            errors: Vec::new(),
            tokens,
            current: 0,
            in_function: false,
            in_type_function: DataTypes::Void,
            in_var_type: DataTypes::Void,
            scope: 0,
            has_entry_point: false,
            is_main: file.is_main,
            scoper: ThrushScoper::new(file),
            diagnostic: Diagnostic::new(file),
            parser_objects: ParserObjects::new(),
        }
    }

    pub fn start(&mut self) -> &[Instruction<'instr>]{
        self.declare_functions();

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
                self.diagnostic.report(error, LogType::ERROR, false);
            });

            process::exit(1);
        } else if self.is_main && !self.has_entry_point {
            self.diagnostic.report(&ThrushError::Parse(ThrushErrorKind::MissingEntryPoint, String::from("Missing EntryPoint"), String::from("Write the entrypoint for this thrush main file."), 0, String::from("fn main() { ... }")), LogType::ERROR, true);
            process::exit(1);
        }

        self.scoper.analyze();

        self.stmts.as_slice()
    }

    fn parse(&mut self) -> Result<Instruction<'instr>, ThrushError> {

        self.parser_objects.decrease_local_references(self.scope);

        match &self.peek().kind {
            TokenKind::Println => Ok(self.println()?),
            TokenKind::Print => Ok(self.print()?),
            TokenKind::Fn => Ok(self.function(false)?),
            TokenKind::LBrace => Ok(self.block(&mut [])?),
            TokenKind::Return => Ok(self.ret()?),
            TokenKind::Public => Ok(self.public()?),
            TokenKind::Var => Ok(self.variable(false)?),
            TokenKind::For => Ok(self.for_loop()?),
            TokenKind::Extern => Ok(self.external()?),
            _ => Ok(self.expression()?),
        }
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
                String::new()
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

        let path: &str = self.consume(
            TokenKind::String,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected a String literal for @import(\"PATH\")."),
            line,
        )?.lexeme.as_ref().unwrap();

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

        let path_converted: &Path =  Path::new(path);

        if path.starts_with("core") && path.split("").filter(|c| *c == ".").count() >= 1 && !path_converted.exists() {
            if !CORE_LIBRARY_PATH.contains_key(path) {
                self.errors.push(ThrushError::Parse(ThrushErrorKind::SyntaxError, String::from("Import Error"), String::from("This module not exist in Thrush Core Library."), line, String::new()));
                return Ok(());
            }

            let library: &(String, String) = CORE_LIBRARY_PATH.get(path).unwrap();

            let mut name: String = String::new();
            let mut path: String = String::new();

            name.clone_from(&library.0);
            path.clone_from(&library.1);

            let file: ThrushFile = ThrushFile { name, path: PathBuf::from(path), is_main: false};

            let content: String = fs::read_to_string(&file.path).unwrap();

            let tokens: Vec<Token> = Lexer::lex(content.as_bytes(), &file);
            let imports: (Vec<Instruction<'_>>, ParserObjects<'_>) = Import::generate(tokens, &file);

            self.stmts.extend_from_slice(&imports.0);
            self.parser_objects.merge_globals(imports.1);

            return Ok(());
        }

        if path_converted.exists() {
            if path_converted.is_dir() {
                self.errors.push(ThrushError::Parse(ThrushErrorKind::SyntaxError, String::from("Import Error"), String::from("A path to directory is don't able to import."), line, String::from("@import(\"dir/to/file.th\");")));
                return Ok(());
            }

            if path_converted.extension().is_none() {
                self.errors.push(ThrushError::Parse(ThrushErrorKind::SyntaxError, String::from("Import Error"), String::from("The file should contain a extension (*.th)."), line, String::from("@import(\"only/th/extension/file.th\");")));
                return Ok(()); 
            }

            if path_converted.extension().unwrap() != "th" {
                self.errors.push(ThrushError::Parse(ThrushErrorKind::SyntaxError, String::from("Import Error"), String::from("Only thrush files (*.th) are allowed to been imported."), line, String::from("@import(\"only/thrush/files/file.th\");")));
                return Ok(());
            }

            if path_converted.file_name().is_none() {
                self.errors.push(ThrushError::Parse(ThrushErrorKind::SyntaxError, String::from("Import Error"), String::from("The file should contain a name (<mythfile>.th)."), line, String::from("@import(\"only/valid/files/file.th\");")));
                return Ok(());
            }

            let file: ThrushFile = ThrushFile {
                name: path_converted.file_name().unwrap().to_string_lossy().to_string(), 
                path: path_converted.to_path_buf(), 
                is_main: false
            };

            let content: String = fs::read_to_string(&file.path).unwrap();

            let tokens: Vec<Token> = Lexer::lex(content.as_bytes(), &file);
            let imports: (Vec<Instruction<'_>>, ParserObjects<'_>) = Import::generate(tokens, &file);

            self.stmts.extend_from_slice(&imports.0);
            self.parser_objects.merge_globals(imports.1);

            return Ok(());
        }

        Err(ThrushError::Parse(
            ThrushErrorKind::SyntaxError,
            String::from("Import not found"),
            String::from(
                "The import not found in the system or std or core library.",
            ),
            line,
            String::new()
        ))
    }

    fn external(&mut self) -> Result<Instruction<'instr>, ThrushError> {
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
            TokenKind::String,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected a String literal for @extern(\"NAME\")."),
            line,
        )?;

        self.consume(
            TokenKind::RParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ')'."),
            line,
        )?;

        let data = match self.peek().kind {
            TokenKind::Fn => self.function(true)?,
            _ => unreachable!(),
        };

        Ok(Instruction::Extern { name: name.lexeme.clone().unwrap(), data: Box::new(data), kind: TokenKind::Fn })
    }

    fn for_loop(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let start_line: usize = self.previous().line;

        let variable: Instruction<'instr> = self.variable(false)?;

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

        if let Instruction::Var { only_comptime, .. } = &mut variable_clone {
            *only_comptime = true;
        }

        let body: Instruction<'instr> = self.block(&mut [variable_clone])?;

        Ok(Instruction::ForLoop {
            variable: Some(Box::new(variable)),
            cond: Some(Box::new(cond)),
            actions: Some(Box::new(actions)),
            block: Box::new(body),
        })
    }

    fn variable(&mut self, only_comptime: bool) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let name: &Token = self.consume(
            TokenKind::Identifier,
            ThrushErrorKind::SyntaxError,
            String::from("Expected Variable Name"),
            String::from("Write the name: \"var --> name <-- : type = value;\"."),
            self.previous().line,
        )?;

        let line: usize = name.line;

        self.consume(
            TokenKind::Colon, 
            ThrushErrorKind::SyntaxError,
            String::from("Expected Variable Type Indicator"),
            String::from("Write the type indicator: \"var name --> : <-- type = value;\"."),    
            line
        )?;

        let kind: DataTypes = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance()?;

                *kind
            }

            _ => {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Expected Variable Type"),
                    String::from("Write the type: \"var name: --> type <-- = value;\"."),
                    line,
                    format!("var {}: String = \"\";", name.lexeme.as_ref().unwrap())
                ));
            }
        };

        if self.peek().kind == TokenKind::SemiColon && kind == DataTypes::Void {
            self.only_advance()?;

            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Variable Don't Declared Without Type"),
                String::from(
                    "Variable type is undefined. Did you forget to specify the variable type to undefined variable? Like \"var thrush: --> string <--;\".",
                ),
                line,
                format!("var {}: String;", name.lexeme.as_ref().unwrap())
            ));
        } else if self.peek().kind == TokenKind::SemiColon {
            self.consume(
                TokenKind::SemiColon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ';'."),
                line,
            )?;

            self.parser_objects.insert_new_local(self.scope, name.lexeme.as_ref().unwrap(), (kind, true, false, false,  0));

            return Ok(Instruction::Var {
                name: name.lexeme.as_ref().unwrap(),
                kind,
                value: Box::new(Instruction::Null),
                line,
                only_comptime,
            });
        }

        self.consume(
            TokenKind::Eq,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '='."),
            name.line,
        )?;

        self.in_var_type = kind;

        let value: Instruction<'instr> = self.expression()?;

        self.check_types(kind, value.get_data_type(), &value, name.line);

        self.parser_objects.insert_new_local(
            self.scope,
            name.lexeme.as_ref().unwrap(),
            (kind, false, false, false, 0),
        );

        if let Instruction::RefVar { kind, .. } = &value {
            if kind == &DataTypes::String {
                self.parser_objects.modify_object_deallocation(name.lexeme.as_ref().unwrap(), (false, true));
            }
        }

        let var: Instruction<'_> = Instruction::Var {
            name: name.lexeme.as_ref().unwrap(),
            kind,
            value: Box::new(value),
            line,
            only_comptime,
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

    fn public(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        match &self.peek().kind {
            TokenKind::Fn => Ok(self.function(true)?),
            TokenKind::Extern => Ok(self.external()?),
            _ => unimplemented!(),
        }
    }

    fn ret(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let line: usize = self.previous().line;

        if !self.in_function {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Return statement outside of function. Invoke this keyword in scope of function."),
                line,
                String::from("fn myfunction(): i32 {\n return 0;\n}")
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

            if self.in_type_function != DataTypes::Void {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    format!("Missing return statement with correctly type '{}', you should rewrite for return with type '{}'.", self.in_type_function, self.in_type_function),
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

        if let Instruction::RefVar { name, kind, .. } = value {
            if kind == DataTypes::String {
                self.parser_objects.modify_object_deallocation(name, (true, false));
            }
        }

        if self.in_type_function == DataTypes::Void && value.get_data_type() != DataTypes::Void {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                format!("Missing function type indicator with type '{}', you should add a correct function type indicator with type '{}'.", value.get_data_type(), value.get_data_type()),
                line,
                String::new()
            ));
        }

        self.check_types(self.in_type_function, value.get_data_type(), &value, line);

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            line,
        )?;

        Ok(Instruction::Return(Box::new(value), self.in_type_function))
    }

    fn block(
        &mut self,
        with_instrs: &mut [Instruction<'instr>],
    ) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        self.parser_objects.begin_local_scope();

        let mut stmts: Vec<Instruction> = Vec::new();
        let mut was_emited_deallocators: bool = false;

        for instr in with_instrs.iter_mut() {
            stmts.push(mem::take(instr));
        }

        while !self.match_token(TokenKind::RBrace)? {
            let instr: Instruction<'instr> = self.parse()?;
            let line: usize = self.previous().line;

            if instr.is_return() {
                if instr.is_indexe_return_of_string() {
                    self.errors.push(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Unreacheable Deallocation"),
                        String::from("The char should be stored in a variable and pass it variable to the return."),
                        line,
                        String::from("var a: String = \"hello\";\nvar b: char = a[0];\nreturn b;"),
                    ));
                }

                let deallocators: Vec<Instruction<'_>> = self.parser_objects.create_deallocators(self.scope);

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

    fn function(
        &mut self,
        is_public: bool,
    ) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        if self.scope != 0 {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "The functions must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
                String::new()
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

        if name.lexeme.as_ref().unwrap() == "main" && self.is_main {
            if self.has_entry_point {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Duplicated EntryPoint"),
                    String::from("The language not support two entrypoints, remove one."),
                    name.line,
                    String::from("fn main() { ... }"),
                ));
            }

            self.consume(
                TokenKind::LParen,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected '('."),
                name.line,
            )?;

            self.consume(
                TokenKind::RParen,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ')'."),
                name.line,
            )?;

            if self.peek().kind != TokenKind::LBrace {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected '{'."),
                    self.peek().line,
                    String::new()
                ));
            }

            if self.peek().kind == TokenKind::LBrace {
                self.has_entry_point = true;

                return Ok(Instruction::EntryPoint {
                    body: Box::new(self.block(&mut [])?),
                });
            } else {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected 'block ({ ... })' for the function body."),
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
                    name.line,
                    String::from("hello :: type, "),
                ));
            }

            let ident: String = self.previous().lexeme.clone().unwrap();

            if !self.match_token(TokenKind::ColonColon)? {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected '::'."),
                    name.line,
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
                        name.line,
                        format!("{} :: String, ", ident),
                    ));

                    self.only_advance()?;

                    continue;
                }
            };

            params.push(Instruction::Param { name: ident, kind })
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

        let return_kind: Option<DataTypes> = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance()?;
                Some(*kind)
            }
            _ => None,
        };

        self.in_type_function = return_kind.unwrap_or(DataTypes::Void);

        let mut function: Instruction<'_> =  Instruction::Function {
            name: name.lexeme.clone().unwrap(),
            params,
            body: None,
            return_kind,
            is_public,
        };


        if self.match_token(TokenKind::SemiColon)? {
            self.in_function = false;
            return Ok(function);
        }

        let body: Box<Instruction> = Box::new(self.block(&mut [])?);

        self.in_function = false;
        
        if let Instruction::Function { body: body_fn, ..} = &mut function {
            *body_fn = Some(body);
        }

        Ok(function)
    }

    fn print(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let start: &Token = self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
            self.previous().line,
        )?;

        let mut args: Vec<Instruction<'instr>> = Vec::with_capacity(24);

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            args.push(self.expression()?);
        }

        self.parse_string_formatted(&args, start.line, true);

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            start.line,
        )?;

        Ok(Instruction::Print(args))
    }

    fn println(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let start: &Token = self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
            self.previous().line,
        )?;

        let mut args: Vec<Instruction<'instr>> = Vec::new();

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            args.push(self.expression()?);
        }

        self.parse_string_formatted(&args, start.line, false);

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            start.line,
        )?;

        Ok(Instruction::Println(args))
    }

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
            let right: Instruction<'_> = self.comparison()?;

            let left_type: DataTypes = instr.get_data_type();
            let right_type: DataTypes = right.get_data_type();

            let kind: DataTypes = if left_type.is_integer() && right_type.is_integer() {
                left_type.calculate_integer_datatype(right_type)
            } else {
                self.in_var_type
            };

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
            )?;

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind,
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
            let right: Instruction<'_> = self.term()?;

            let left_type: DataTypes = instr.get_data_type();
            let right_type: DataTypes = right.get_data_type();

            let kind: DataTypes = if left_type.is_integer() && right_type.is_integer() {
                left_type.calculate_integer_datatype(right_type)
            } else {
                self.in_var_type
            };

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
            )?;

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind,
            };
        }

        Ok(instr)
    }

    fn term(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction<'_> = self.factor()?;

        while self.match_token(TokenKind::Plus)?
            || self.match_token(TokenKind::Minus)?
        {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction<'_> = self.factor()?;

            let left_type: DataTypes = instr.get_data_type();
            let right_type: DataTypes = right.get_data_type();

            let kind: DataTypes = if left_type.is_integer() && right_type.is_integer() {
                left_type.calculate_integer_datatype(right_type)
            } else {
                self.in_var_type
            };

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
            )?;

            instr = Instruction::BinaryOp {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind,
            };
        }

        Ok(instr)
    }

    fn factor(&mut self) -> Result<Instruction<'instr>, ThrushError>{
        let mut instr: Instruction<'_> = self.unary()?;

        while self.match_token(TokenKind::Slash)?
            || self.match_token(TokenKind::Star)?
        {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction<'_> = self.unary()?;

            let left_type: DataTypes = instr.get_data_type();
            let right_type: DataTypes = right.get_data_type();

            let kind: DataTypes = if left_type.is_integer() && right_type.is_integer() {
                left_type.calculate_integer_datatype(right_type)
            } else {
                self.in_var_type
            };

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
            )?;

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
            let value: Instruction<'instr> = self.primary()?;

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
            let mut value: Instruction<'instr> = self.primary()?;

            if let Instruction::Integer(_, _, is_signed) = &mut value {
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

        let instr: Instruction<'_> = self.primary()?;

        Ok(instr)
    }

    fn primary(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let primary: Instruction = match &self.peek().kind {
            TokenKind::LParen => {
                let line: usize = self.peek().line;

                self.only_advance()?;

                let instr: Instruction<'instr> = self.expression()?;
                let kind: DataTypes = instr.get_data_type();

                /* if !instr.is_binary() {
                    self.errors.push(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Syntax Error"),
                        String::from(
                            "Group the expressions \"(...)\" is only allowed if contain binary expressions.",
                        ),
                        line,
                        String::from("(T + T)")
                    ));
                } */

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

            TokenKind::String => {
                let current: &Token = self.advance()?;

                Instruction::String(
                    current.lexeme.as_ref().unwrap().to_string(),
                    current.lexeme.as_ref().unwrap().contains("{}"),
                )
            }
            TokenKind::Char => {
                Instruction::Char(self.advance()?.lexeme.as_ref().unwrap().as_bytes()[0])
            }

            kind => match kind {
                TokenKind::Integer(kind, num, is_signed) => {
                    self.only_advance()?;

                    let instr: Instruction<'_> = Instruction::Integer(*kind, *num, *is_signed);

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

                    let instr: Instruction<'instr> = Instruction::Float(*kind, *num, *is_signed);

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

                    let object: ParserObject =
                        self.parser_objects.get_object(current.lexeme.as_ref().unwrap(), line)?;

                    let name: &str = current.lexeme.as_ref().unwrap();

                    self.only_advance()?;

                    if self.peek().kind == TokenKind::LeftBracket {
                        self.consume(
                            TokenKind::LeftBracket,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected '['."),
                            line,
                        )?;

                        let expr: Instruction<'instr> = self.primary()?;

                        self.consume(
                            TokenKind::RightBracket,
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
                            String::new()
                        ));
                    } else if self.match_token(TokenKind::Eq)? {
                        let expr: Instruction<'instr> = self.expression()?;

                        self.check_types(object.0, expr.get_data_type(), &expr, line);

                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                            line,
                        )?;

                        self.parser_objects.insert_new_local(self.scope, name, (object.0, false, false, false, 0));

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
                            String::from("Variable Not Declared"),
                            format!(
                                "Variable `{}` is not declared for are use it. Declare the variable before of the use.",
                                name,
                            ),
                            line,
                            String::new()
                        ));
                    }

                    let refvar: Instruction<'_> = Instruction::RefVar {
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

                        let expr: Instruction<'_> = Instruction::UnaryOp {
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
                    ))
                }
            },
        };

        Ok(primary)
    }

    fn check_types(&mut self, target: DataTypes, value_type: DataTypes, value: &Instruction, line: usize) {
        if value.is_binary() || value.is_group() {
            if let Err(err) = type_checking::check_types(target, None, Some(value), None, line,
                String::from("Type Mismatch"),
                format!(
                    "Type mismatch. Expected '{}' but found '{}'.",
                    target, value_type
                )) {

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

        Err(ThrushError::Parse(error_kind, error_title, help, line, String::new()))
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
                String::from("fn myfunction() { ... }\n fn main() { myfunction(); }"),
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

        let mut index: usize = 0;

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
            args.iter().for_each(|arg| {
                let arg_kind: DataTypes = arg.get_data_type();
    
                if object.5.len() > index && object.5[index] != arg_kind  {
                    self.errors.push(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Syntax Error"),
                        format!(
                            "Function called, expected '{}' argument type in position {} don't '{}' type.",
                            object.5[index], index, arg_kind
                        ),
                        line,
                        String::new()
                    ));
                }
    
                index += 1;
            });
        }
        
        Ok(Instruction::Call {
            name,
            args,
            kind: object.0,
        })
    }

    fn parse_string_formatted(&mut self, args: &[Instruction], line: usize, scan_spaces: bool) {
        if args.is_empty() {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "Expected at least 1 argument for 'println' call. Like 'println(\"Hi!\");'",
                ),
                line,
                String::new()
            ));
        } else if let Instruction::String(str, _) = &args[0] {
            let mut formats: usize = 0;

            str.split_inclusive("{}").for_each(|substr| {
                if substr.contains("{}") {
                    formats += 1;
                }
            });

            if formats != args.iter().skip(1).collect::<Vec<_>>().len() {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Expected format"),
                    String::from("Missing format for argument or an argument. Should be like this println(\"{}\", arguments.size() == formatters.size());"),
                    line,
                    String::new()
                ));
            }
        }

        if scan_spaces {
            args.iter().for_each(|arg| {
                if let Instruction::String(str, _) = arg {
                    if str.contains("\n") {
                        self.errors.push(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from(
                                "You can't print strings that contain newlines. Use 'println' instead.",
                            ),
                            self.peek().line,
                            String::new()
                        ));
                    }
                }
            });
        }
    }


    fn declare_functions(&mut self) {
        let mut functions_positions: Vec<usize> = Vec::new();
        let mut pos: usize = 0;

        self.tokens.iter().for_each(|tok| {
            if let TokenKind::Fn = tok.kind {
                functions_positions.push(pos);
                pos += 1;
            }
            else {
                pos += 1;
            }
        });

        functions_positions.iter().for_each(|index| {
            let _ = self.declare_function(*index);
        });
    }

    fn declare_function(&mut self, index: usize) -> Result<(), ThrushError> {
        self.current = index;

        let is_external: bool = if self.current < 4 {
            false
        } else {
            self.tokens[self.current - 4].kind == TokenKind::Extern
        };

        if is_external {
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

        if ignore_more_params && !is_external {
            self.errors.push(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "Ignore statement \"...\" in functions is only allowed for external functions.",
                ),
                name.line,
                String::new()
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

        let return_kind: DataTypes = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance()?;
                *kind
            }
            _ => DataTypes::Void,
        };

        self.current = 0;

        self.parser_objects.insert_new_global(name.lexeme.clone().unwrap(), (return_kind, params, true, ignore_more_params));

        Ok(())
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
