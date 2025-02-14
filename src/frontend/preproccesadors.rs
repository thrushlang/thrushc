use {
    super::{
        super::{
            backend::{compiler::options::ThrushFile, instruction::Instruction},
            diagnostic::Diagnostic,
            error::{ThrushError, ThrushErrorKind},
            logging::LogType,
        },
        lexer::{DataTypes, Token, TokenKind},
        objects::ParserObjects,
    },
    ahash::AHashMap as HashMap,
    std::{mem, process},
};

pub struct Import<'instr> {
    tokens: Vec<Token>,
    stmts: Vec<Instruction<'instr>>,
    errors: Vec<ThrushError>,
    current: usize,
    diagnostic: Diagnostic,
    parser_objects: ParserObjects<'instr>,
}

impl<'instr> Import<'instr> {
    pub fn generate(
        tokens: Vec<Token>,
        file: &ThrushFile,
    ) -> (Vec<Instruction<'instr>>, ParserObjects<'instr>) {
        let mut imports: Import = Self {
            tokens,
            stmts: Vec::new(),
            errors: Vec::new(),
            current: 0,
            diagnostic: Diagnostic::new(file),
            parser_objects: ParserObjects::new(HashMap::new()),
        };

        imports._parse()
    }

    fn _parse(&mut self) -> (Vec<Instruction<'instr>>, ParserObjects<'instr>) {
        while !self.end() {
            if self.match_token(TokenKind::Public) {
                if let Err(e) = self.public() {
                    self.errors.push(e);
                }
            }

            self.only_advance();
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error: &ThrushError| {
                self.diagnostic.report(error, LogType::ERROR, false);
            });

            process::exit(1);
        }

        (
            mem::take(&mut self.stmts),
            mem::take(&mut self.parser_objects),
        )
    }

    fn public(&mut self) -> Result<(), ThrushError> {
        if self.peek().kind == TokenKind::Extern {
            while self.peek().kind != TokenKind::Fn || self.peek().kind != TokenKind::Struct {
                self.only_advance();
            }
        }

        if self.match_token(TokenKind::Fn) {
            self.build_function()?;
            return Ok(());
        }

        if self.match_token(TokenKind::Struct) {
            self.build_struct()?;
            return Ok(());
        }

        Ok(())
    }

    fn build_struct(&mut self) -> Result<(), ThrushError> {
        let name: String = self
            .consume(
                TokenKind::Identifier,
                ThrushErrorKind::SyntaxError,
                String::from("Expected struct name"),
                String::from("Write the struct name: \"struct --> name <-- { ... };\"."),
                self.previous().line,
                String::new(),
            )?
            .lexeme
            .clone()
            .unwrap();

        let line: usize = self.previous().line;

        self.consume(
            TokenKind::LBrace,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '{'."),
            line,
            String::new(),
        )?;

        let mut fields_types: HashMap<String, DataTypes> = HashMap::new();

        while self.peek().kind != TokenKind::RBrace {
            if self.match_token(TokenKind::Comma) {
                continue;
            }

            if self.match_token(TokenKind::Identifier) {
                let field_name: String = self.previous().lexeme.clone().unwrap();
                let line: usize = self.previous().line;

                let field_type: DataTypes = match self.peek().kind {
                    TokenKind::DataType(kind) => {
                        self.only_advance();
                        kind
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

            self.only_advance();
        }

        self.consume(
            TokenKind::RBrace,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '}'."),
            line,
            String::new(),
        )?;

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
            line,
            String::new(),
        )?;

        self.add_struct(name, fields_types);

        Ok(())
    }

    fn build_function(&mut self) -> Result<(), ThrushError> {
        let line: usize = self.previous().line;

        let name: String = self
            .consume(
                TokenKind::Identifier,
                ThrushErrorKind::SyntaxError,
                String::from("Expected function name"),
                String::from("Expected a name to the function."),
                self.previous().line,
                String::from("fn hello() {}"),
            )?
            .lexeme
            .clone()
            .unwrap();

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
            line,
            format!("{}(", name),
        )?;

        let mut params: Vec<Instruction> = Vec::new();
        let mut params_types: Vec<DataTypes> = Vec::new();

        let mut param_position: u32 = 0;

        while !self.match_token(TokenKind::RParen) {
            if self.match_token(TokenKind::Comma) {
                continue;
            }

            if self.match_token(TokenKind::Pass) {
                continue;
            }

            if !self.match_token(TokenKind::Identifier) {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected argument name."),
                    line,
                    String::from("hello :: type, "),
                ));
            }

            let ident: String = self.previous().lexeme.clone().unwrap();
            let line: usize = self.previous().line;

            if !self.match_token(TokenKind::ColonColon) {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected '::'."),
                    line,
                    format!("{} :: type, ", ident),
                ));
            }

            let kind: DataTypes = match self.peek().kind {
                TokenKind::DataType(kind) => {
                    self.only_advance();

                    kind
                }
                _ => {
                    self.errors.push(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Syntax Error"),
                        String::from("Expected argument type."),
                        line,
                        format!("{} :: type, ", ident),
                    ));

                    DataTypes::Void
                }
            };

            params.push(Instruction::Param {
                name: ident,
                kind,
                line,
                position: param_position,
            });

            params_types.push(kind);
            param_position += 1;
        }

        if self.peek().kind == TokenKind::Colon {
            self.consume(
                TokenKind::Colon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Missing return type. Expected ':' followed by return type."),
                line,
                format!("fn {}(...): type", name),
            )?;
        }

        let return_type: Option<(DataTypes, String)> = match self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance();
                Some((kind, String::new()))
            }

            TokenKind::Identifier => {
                if self
                    .parser_objects
                    .get_struct(self.peek().lexeme.as_ref().unwrap(), line)
                    .is_ok()
                {
                    self.only_advance();
                    Some((DataTypes::Struct, self.previous().lexeme.clone().unwrap()))
                } else {
                    None
                }
            }
            _ => None,
        };

        let return_type: (DataTypes, String) =
            return_type.unwrap_or((DataTypes::Void, String::new()));

        self.add_build_function(name.clone(), return_type.0, params_types);

        self.stmts.push(Instruction::Extern {
            name: name.clone(),
            instr: Box::new(Instruction::Function {
                name,
                params,
                body: None,
                return_type: return_type.0,
                is_public: true,
            }),
            kind: TokenKind::Fn,
        });

        Ok(())
    }

    fn add_struct(&mut self, name: String, fields: HashMap<String, DataTypes>) {
        self.parser_objects.insert_new_struct(name, fields);
    }

    fn add_build_function(&mut self, name: String, kind: DataTypes, datatypes: Vec<DataTypes>) {
        self.parser_objects
            .insert_new_global(name, (kind, datatypes, true, false));
    }

    fn consume(
        &mut self,
        kind: TokenKind,
        error_kind: ThrushErrorKind,
        error_title: String,
        help: String,
        line: usize,
        suggest_code: String,
    ) -> Result<&Token, ThrushError> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushError::Parse(
            error_kind,
            error_title,
            help,
            line,
            suggest_code,
        ))
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.end() {
            return false;
        } else if self.peek().kind == kind {
            self.only_advance();

            return true;
        }

        false
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn only_advance(&mut self) {
        if !self.end() {
            self.current += 1;
        }
    }

    fn advance(&mut self) -> Result<&Token, ThrushError> {
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

    fn end(&self) -> bool {
        self.tokens[self.current].kind == TokenKind::Eof
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
