use {
    super::{
        super::{
            backend::{compiler::options::ThrushFile, instruction::Instruction},
            diagnostic::Diagnostic,
            error::{ThrushError, ThrushErrorKind},
            logging::LogType,
        },
        lexer::{DataTypes, Token, TokenKind},
    },
    std::{mem, process},
};

pub struct Imports<'instr> {
    tokens: Vec<Token>,
    stmts: Vec<Instruction<'instr>>,
    errors: Vec<ThrushError>,
    current: usize,
    diagnostic: Diagnostic,
}

impl<'instr> Imports<'instr> {
    pub fn parse(tokens: Vec<Token>, file: &ThrushFile) -> Vec<Instruction<'instr>> {
        let mut imports: Imports = Self {
            tokens,
            stmts: Vec::new(),
            errors: Vec::new(),
            current: 0,
            diagnostic: Diagnostic::new(file),
        };

        imports._parse()
    }

    fn _parse(&mut self) -> Vec<Instruction<'instr>> {
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
                self.diagnostic.report(error, LogType::ERROR);
            });

            process::exit(1);
        }

        mem::take(&mut self.stmts)
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
        )?;

        let mut params: Vec<Instruction> = Vec::new();

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
                ));
            }

            let ident: String = self.previous().lexeme.clone().unwrap();

            if !self.match_token(TokenKind::ColonColon) {
                self.errors.push(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected '::'."),
                    line,
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
                    ));

                    DataTypes::Void
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
                line,
            )?;
        }

        let return_kind: Option<DataTypes> = match self.peek().kind {
            TokenKind::DataType(kind) => {
                self.only_advance();
                Some(kind)
            }
            _ => None,
        };

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

    fn consume(
        &mut self,
        kind: TokenKind,
        error_kind: ThrushErrorKind,
        error_title: String,
        help: String,
        line: usize,
    ) -> Result<&Token, ThrushError> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushError::Parse(error_kind, error_title, help, line))
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
        ))
    }

    fn end(&self) -> bool {
        self.tokens[self.current].kind == TokenKind::Eof
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
