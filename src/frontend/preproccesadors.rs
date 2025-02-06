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
            parser_objects: ParserObjects::new(),
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
            while self.peek().kind != TokenKind::Fn {
                self.only_advance();
            }
        }

        if self.match_token(TokenKind::Fn) {
            self.function()?;
            return Ok(());
        }

        Ok(())
    }

    fn function(&mut self) -> Result<(), ThrushError> {
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

            params.push(Instruction::Param { name: ident, kind });
            params_types.push(kind);
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

        let return_kind: Option<DataTypes> = match self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance();
                Some(kind)
            }
            _ => None,
        };

        self.add_function(
            name.clone(),
            return_kind.unwrap_or(DataTypes::Void),
            params_types,
        );

        self.stmts.push(Instruction::Extern {
            name: name.clone(),
            data: Box::new(Instruction::Function {
                name,
                params,
                body: None,
                return_kind,
                is_public: true,
            }),
            kind: TokenKind::Fn,
        });

        Ok(())
    }

    fn add_function(&mut self, name: String, kind: DataTypes, datatypes: Vec<DataTypes>) {
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
