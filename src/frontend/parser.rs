use {
    super::{
        super::{
            backend::compiler::{Instruction, Options, Scope},
            diagnostic::Diagnostic,
            error::{ThrushError, ThrushErrorKind},
            logging::Logging,
        },
        lexer::{DataTypes, Token, TokenKind},
    },
    ahash::AHashMap as HashMap,
    std::mem,
};

const C_FMTS: [&str; 2] = ["%s", "%d"];

pub struct Parser<'instr, 'a> {
    stmts: Vec<Instruction<'instr>>,
    errors: Vec<ThrushError>,
    pub tokens: Option<&'instr [Token]>,
    pub options: Option<&'a Options>,
    function: u16,
    ret: Option<DataTypes>,
    current: usize,
    globals: HashMap<&'instr str, DataTypes>,
    locals: Vec<HashMap<&'instr str, DataTypes>>,
    scope: usize,
    scoper: ThrushScoper<'instr>,
}

impl<'instr, 'a> Parser<'instr, 'a> {
    pub fn new() -> Self {
        Self {
            stmts: Vec::new(),
            errors: Vec::with_capacity(10),
            tokens: None,
            options: None,
            current: 0,
            ret: None,
            function: 0,
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            scope: 0,
            scoper: ThrushScoper::new(),
        }
    }

    pub fn start(&mut self) -> Result<&[Instruction<'instr>], String> {
        while !self.end() {
            match self.parse() {
                Ok(instr) => {
                    self.stmts.push(instr);
                }
                Err(e) => {
                    if self.errors.len() >= 10 {
                        break;
                    }

                    self.errors.push(e)
                }
            }
        }

        if !self.errors.is_empty() {
            for error in mem::take(&mut self.errors) {
                if let ThrushError::Compile(msg) = error {
                    Logging::new(msg).error();
                    continue;
                }

                Diagnostic::new(error).report();
            }

            return Err(String::from("Compilation proccess ended with errors."));
        }

        Ok(self.stmts.as_slice())
    }

    fn parse(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        match &self.peek().kind {
            TokenKind::Print => Ok(self.print()?),
            TokenKind::Def => Ok(self.def(false)?),
            TokenKind::LBrace => Ok(self.block(false, false)?),
            TokenKind::Return => Ok(self.ret()?),
            TokenKind::Pub => Ok(self.pub_def()?),
            TokenKind::Let => Ok(self.var()?),
            _ => Ok(self.expr()?),
        }
    }

    fn var(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.advance();

        let name: &'instr Token = self.consume(
            TokenKind::Identifier,
            ThrushErrorKind::SyntaxError,
            String::from("Expected variable name"),
            String::from("Expected let <name>."),
        )?;

        let kind: Option<DataTypes> = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                self.advance();

                Some(kind.dereference())
            }

            _ => None,
        };

        if self.peek().kind == TokenKind::SemiColon && kind.is_none() {
            self.advance();

            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                format!("let {}", name.lexeme.as_ref().unwrap()),
                String::from("Syntax Error"),
                String::from(
                    "Variable type is undefined. Did you forget to specify the variable type to undefined variable?",
                ),
                name.span,
                name.line,
            ));
        } else if self.peek().kind == TokenKind::SemiColon {
            self.consume(
                TokenKind::SemiColon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ';'."),
            )?;

            return Ok(Instruction::Var {
                name: name.lexeme.as_ref().unwrap(),
                kind: kind.unwrap(),
                value: None,
            });
        }

        self.consume(
            TokenKind::Eq,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '=' for the variable definition."),
        )?;

        let value: Instruction<'instr> = self.parse()?;

        if kind.is_some() {
            match &value {
                Instruction::Integer(data_type, _) => {
                    if data_type != kind.as_ref().unwrap() {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            format!("let {}", name.lexeme.as_ref().unwrap()),
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}' number.",
                                kind.unwrap(),
                                data_type
                            ),
                            name.span,
                            name.line,
                        ));
                    }
                }

                Instruction::String(_) => {
                    if kind.as_ref().unwrap() != &DataTypes::String {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            format!("let {}", name.lexeme.as_ref().unwrap()),
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}'.",
                                kind.as_ref().unwrap(),
                                DataTypes::String
                            ),
                            name.span,
                            name.line,
                        ));
                    }
                }

                _ => todo!(),
            }
        }

        let variable: Instruction<'_> = if kind.as_ref().is_none() {
            Instruction::Var {
                name: name.lexeme.as_ref().unwrap(),
                kind: value.get_data_type(),
                value: Some(Box::new(value)),
            }
        } else {
            Instruction::Var {
                name: name.lexeme.as_ref().unwrap(),
                kind: kind.unwrap(),
                value: Some(Box::new(value)),
            }
        };

        self.define_local(name.lexeme.as_ref().unwrap(), variable.get_kind().unwrap());

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
        )?;

        Ok(variable)
    }

    fn pub_def(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.advance();

        match &self.peek().kind {
            TokenKind::Def => Ok(self.def(true)?),
            _ => unimplemented!(),
        }
    }

    fn ret(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.advance();

        if self.function == 0 {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                self.peek().lexeme.as_ref().unwrap().to_string(),
                String::from("Syntax Error"),
                String::from("Return statement outside of function. Invoke this keyword in scope of function definition."),
                self.peek().span,
                self.peek().line,
            ));
        }

        if self.peek().kind == TokenKind::SemiColon {
            self.consume(
                TokenKind::SemiColon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ';'."),
            )?;

            return Ok(Instruction::Return(Box::new(Instruction::Null)));
        }

        let value: Instruction<'instr> = self.parse()?;

        match &value {
            Instruction::Integer(kind, _) => match kind {
                DataTypes::U8 => self.ret = Some(DataTypes::U8),
                DataTypes::U16 => self.ret = Some(DataTypes::U16),
                DataTypes::U32 => self.ret = Some(DataTypes::U32),
                DataTypes::U64 => self.ret = Some(DataTypes::U64),

                DataTypes::I8 => self.ret = Some(DataTypes::I8),
                DataTypes::I16 => self.ret = Some(DataTypes::I16),
                DataTypes::I32 => self.ret = Some(DataTypes::I32),
                DataTypes::I64 => self.ret = Some(DataTypes::I64),

                DataTypes::F32 => self.ret = Some(DataTypes::F32),
                DataTypes::F64 => self.ret = Some(DataTypes::F64),

                _ => unreachable!(),
            },

            Instruction::String(_) => self.ret = Some(DataTypes::String),
            Instruction::Boolean(_) => self.ret = Some(DataTypes::Bool),

            _ => unreachable!(),
        }

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Return(Box::new(value)))
    }

    fn block(
        &mut self,
        in_function: bool,
        in_loop: bool,
    ) -> Result<Instruction<'instr>, ThrushError> {
        self.advance();

        self.begin_scope();

        let mut stmts: Vec<Instruction> = Vec::new();

        while !self.match_token(TokenKind::RBrace) {
            stmts.push(self.parse()?)
        }

        self.end_scope();

        self.scoper
            .begin_scope(Instruction::Block(stmts.clone()), in_function, in_loop);

        self.scoper.analyze()?;

        Ok(Instruction::Block(stmts))
    }

    fn def(&mut self, is_public: bool) -> Result<Instruction<'instr>, ThrushError> {
        self.advance();

        self.begin_function();

        let name: &'instr Token = self.consume(
            TokenKind::Identifier,
            ThrushErrorKind::SyntaxError,
            String::from("Expected function name"),
            String::from("Expected def <name>."),
        )?;

        if name.lexeme.as_ref().unwrap() == "main"
            && self.scope == 0
            && self.options.unwrap().is_main
        {
            self.consume(
                TokenKind::LParen,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected '('."),
            )?;

            self.consume(
                TokenKind::RParen,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ')'."),
            )?;

            self.consume(
                TokenKind::Colon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ':' for the return."),
            )?;

            match self.peek().kind {
                TokenKind::DataType(DataTypes::Void) => {
                    self.advance();

                    if self.peek().kind != TokenKind::LBrace {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            self.peek().lexeme.as_ref().unwrap().to_string(),
                            String::from("Syntax Error"),
                            String::from("Expected '{'."),
                            self.peek().span,
                            self.peek().line,
                        ));
                    }

                    if self.peek().kind == TokenKind::LBrace {
                        return Ok(Instruction::EntryPoint {
                            body: Box::new(self.block(true, false)?),
                        });
                    } else {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            self.peek().lexeme.as_ref().unwrap().to_string(),
                            String::from("Syntax Error"),
                            String::from("Expected 'block' for the function body."),
                            self.peek().span,
                            self.peek().line,
                        ));
                    }
                }

                _ => {
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        self.peek().lexeme.as_ref().unwrap().to_string(),
                        String::from("Syntax Error"),
                        String::from("Expected 'void' type return."),
                        self.peek().span,
                        self.peek().line,
                    ));
                }
            }
        }

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
        )?;

        let mut params: Vec<Instruction> = Vec::with_capacity(8);

        while !self.match_token(TokenKind::RParen) {
            if self.match_token(TokenKind::Comma) {
                continue;
            }

            if params.len() >= 8 {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::TooManyArguments,
                    self.peek().lexeme.as_ref().unwrap().to_string(),
                    String::from("Syntax Error"),
                    String::from("Too many arguments for the function. The maximum number of arguments is 8."),
                    self.peek().span,
                    self.peek().line,
                ));
            }

            if !self.match_token(TokenKind::Identifier) {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    self.peek().lexeme.as_ref().unwrap().to_string(),
                    String::from("Syntax Error"),
                    String::from("Expected argument name."),
                    self.peek().span,
                    self.peek().line,
                ));
            }

            let ident: &str = self.previous().lexeme.as_ref().unwrap();

            if !self.match_token(TokenKind::ColonColon) {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    self.peek().lexeme.as_ref().unwrap().to_string(),
                    String::from("Syntax Error"),
                    String::from("Expected '::'."),
                    self.peek().span,
                    self.peek().line,
                ));
            }

            let kind: DataTypes = match &self.peek().kind {
                TokenKind::DataType(kind) => {
                    self.advance();

                    kind.dereference()
                }
                _ => {
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        self.peek().lexeme.as_ref().unwrap().to_string(),
                        String::from("Syntax Error"),
                        String::from("Expected argument type."),
                        self.peek().span,
                        self.peek().line,
                    ));
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
            )?;
        }

        let return_kind: Option<DataTypes> = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                self.advance();
                Some(kind.dereference())
            }
            _ => None,
        };

        let body: Box<Instruction> = Box::new(self.block(true, false)?);

        match &return_kind {
            Some(kind) => {
                if self.ret.is_none() {
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        name.lexeme.as_ref().unwrap().to_string(),
                        String::from("Syntax Error"),
                        format!("Missing return statement with type '{}', you should add a return statement with type '{}'.", kind, kind),
                        name.span,
                        name.line,
                    ));
                }

                match kind != self.ret.as_ref().unwrap() {
                    true => {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            name.lexeme.as_ref().unwrap().to_string(),
                            String::from("Syntax Error"),
                            format!(
                                "Expected return type of '{}', found '{}'. You should write a return statement with type '{}'.",
                                kind,
                                self.ret.as_ref().unwrap(),
                                kind
                            ),
                            name.span,
                            name.line,
                        ))
                    }

                    false => {}
                }
            }

            None => {}
        }

        self.end_function();

        match &return_kind {
            Some(kind) => {
                self.define_global(name.lexeme.as_ref().unwrap(), kind.dereference());
            }

            None => {
                self.define_global(name.lexeme.as_ref().unwrap(), DataTypes::Void);
            }
        }

        Ok(Instruction::Function {
            name: name.lexeme.as_ref().unwrap(),
            params,
            body,
            return_kind,
            is_public,
        })
    }

    fn print(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.advance();

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
        )?;

        let mut args: Vec<Instruction<'instr>> = Vec::with_capacity(24);

        while !self.match_token(TokenKind::RParen) {
            if self.match_token(TokenKind::Comma) {
                continue;
            }

            if args.len() >= 24 {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::TooManyArguments,
                    self.peek().lexeme.as_ref().unwrap().to_string(),
                    String::from("Syntax Error"),
                    String::from("Expected ')'. Too many arguments. Max is 24."),
                    self.peek().span,
                    self.peek().line,
                ));
            }

            args.push(self.expr()?);
        }

        if args.is_empty() && self.match_token(TokenKind::SemiColon) {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                self.peek().lexeme.as_ref().unwrap().to_string(),
                String::from("Syntax Error"),
                String::from(
                    "Expected at least 1 argument for 'println' call. Like 'println(`Hi!`);'",
                ),
                self.peek().span,
                self.peek().line,
            ));
        } else if let Instruction::String(str) = &args[0] {
            if args.len() <= 1 && C_FMTS.iter().any(|fmt| str.contains(*fmt)) {
                self.consume(
                    TokenKind::SemiColon,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected ';'."),
                )?;

                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    self.peek().lexeme.as_ref().unwrap().to_string(),
                    String::from("Syntax Error"),
                    String::from(
                        "Expected at least 2 arguments for 'println' call. Like 'println(`%d`, 2);'",
                    ),
                    self.peek().span,
                    self.peek().line,
                ));
            }
        }

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Print(args))
    }

    fn expr(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let expr: Instruction = self.primary()?;

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let primary: Instruction = match &self.peek().kind {
            TokenKind::String => {
                Instruction::String(self.advance().lexeme.as_ref().unwrap().to_string())
            }
            kind => match kind {
                TokenKind::Integer(kind, num) => {
                    self.advance();

                    match kind {
                        DataTypes::I8 => Instruction::Integer(DataTypes::I8, (*num as i8).into()),
                        DataTypes::I16 => {
                            Instruction::Integer(DataTypes::I16, (*num as i16).into())
                        }
                        DataTypes::I32 => {
                            Instruction::Integer(DataTypes::I32, (*num as i32).into())
                        }
                        DataTypes::I64 => {
                            Instruction::Integer(DataTypes::I64, (*num as i64) as f64)
                        }

                        DataTypes::U8 => Instruction::Integer(DataTypes::U8, (*num as u8).into()),
                        DataTypes::U16 => Instruction::Integer(DataTypes::U8, (*num as u16).into()),
                        DataTypes::U32 => {
                            Instruction::Integer(DataTypes::U32, (*num as u32).into())
                        }
                        DataTypes::U64 => {
                            Instruction::Integer(DataTypes::U64, (*num as u64) as f64)
                        }

                        DataTypes::F32 => {
                            Instruction::Integer(DataTypes::F32, (*num as f32).into())
                        }

                        DataTypes::F64 => Instruction::Integer(DataTypes::F64, *num),

                        _ => unreachable!(),
                    }
                }

                TokenKind::Identifier => {
                    self.advance();

                    let scope: Scope = self.find_scope(self.previous().lexeme.as_ref().unwrap());

                    if self.peek().kind == TokenKind::Eq {
                        let name: &str = self.previous().lexeme.as_ref().unwrap();
                        self.advance();

                        let expr: Instruction<'instr> = self.expr()?;

                        match scope {
                            Scope::Global => match self.globals.get(name) {
                                None => {}
                                Some(instr) => {
                                    todo!()
                                }
                            },

                            Scope::Local => match self.locals[self.scope].get(name) {
                                None => {}
                                Some(instr) => {
                                    todo!()
                                }
                            },

                            Scope::Unreachable => {}
                        }

                        return Ok(Instruction::MutVar {
                            name,
                            value: Box::new(expr),
                            scope,
                        });
                    }

                    Instruction::RefVar {
                        name: self.previous().lexeme.as_ref().unwrap(),
                        scope,
                    }
                }

                TokenKind::True => {
                    self.advance();

                    Instruction::Boolean(true)
                }

                TokenKind::False => {
                    self.advance();

                    Instruction::Boolean(false)
                }

                TokenKind::RParen | TokenKind::RBrace => {
                    self.advance();

                    return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            self.peek().lexeme.as_ref().unwrap().to_string(),
                            String::from("Syntax Error"),
                            format!("Expected expression, found '{}'. Is this a function call or an function definition?", kind),
                            self.peek().span,
                            self.peek().line,
                        ));
                }

                kind => {
                    self.advance();

                    println!("{:?}", self.previous());

                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        self.peek().lexeme.as_ref().unwrap().to_string(),
                        String::from("Syntax Error"),
                        format!("Unexpected code '{}', check the code and review the syntax rules in the documentation.", kind),
                        self.peek().span,
                        self.peek().line,
                    ));
                }
            },
        };

        Ok(primary)
    }

    fn consume(
        &mut self,
        kind: TokenKind,
        error_kind: ThrushErrorKind,
        error_title: String,
        help: String,
    ) -> Result<&'instr Token, ThrushError> {
        if self.peek().kind == kind {
            return Ok(self.advance());
        }

        println!("{:?}", self.peek());

        Err(ThrushError::Parse(
            error_kind,
            self.peek().lexeme.as_ref().unwrap().to_string(),
            error_title,
            help,
            self.peek().span,
            self.peek().line,
        ))
    }

    fn find_scope(&self, name: &str) -> Scope {
        if self.locals[self.scope - 1].contains_key(name) {
            return Scope::Local;
        } else if self.globals.contains_key(name) {
            return Scope::Global;
        }

        Scope::Unreachable
    }

    fn define_global(&mut self, name: &'instr str, kind: DataTypes) {
        self.globals.insert(name, kind);
    }

    fn define_local(&mut self, name: &'instr str, kind: DataTypes) {
        self.locals[self.scope - 1].insert(name, kind);
    }

    fn begin_scope(&mut self) {
        self.scope += 1;
    }

    fn end_scope(&mut self) {
        self.scope -= 1;
    }

    fn begin_function(&mut self) {
        self.function += 1;
    }

    fn end_function(&mut self) {
        self.function -= 1;
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        if self.end() {
            return false;
        }

        self.peek().kind == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.end() {
            return false;
        } else if self.peek().kind == kind {
            self.advance();
            return true;
        }

        false
    }

    fn advance(&mut self) -> &'instr Token {
        if !self.end() {
            self.current += 1;
        }

        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens.unwrap()[self.current].clone()
    }

    fn previous(&self) -> &'instr Token {
        &self.tokens.unwrap()[self.current - 1]
    }

    fn end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}

impl PartialEq for Instruction<'_> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Instruction::Integer(_, _) => {
                matches!(other, Instruction::Integer(_, _))
            }

            Instruction::String(_) => {
                matches!(other, Instruction::String(_))
            }

            _ => self == other,
        }
    }
}

impl<'instr> Instruction<'instr> {
    pub fn get_data_type(&self) -> DataTypes {
        match self {
            Instruction::Integer(data_type, _) => match data_type {
                DataTypes::U8 => DataTypes::U8,
                DataTypes::U16 => DataTypes::U16,
                DataTypes::U32 => DataTypes::U32,
                DataTypes::U64 => DataTypes::U64,

                DataTypes::I8 => DataTypes::I8,
                DataTypes::I16 => DataTypes::I16,
                DataTypes::I32 => DataTypes::I32,
                DataTypes::I64 => DataTypes::I64,

                DataTypes::F32 => DataTypes::F32,
                DataTypes::F64 => DataTypes::F64,

                _ => unreachable!(),
            },

            Instruction::String(_) => DataTypes::String,
            Instruction::Boolean(_) => DataTypes::Bool,

            _ => unreachable!(),
        }
    }

    pub fn get_kind(&self) -> Option<DataTypes> {
        match self {
            Instruction::Var { kind, .. } => Some(kind.dereference()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ThrushScoper<'ctx> {
    blocks: Vec<ThrushBlock<'ctx>>,
    count: usize,
}

#[derive(Debug)]
pub struct ThrushBlock<'ctx> {
    instructions: Vec<ThrushInstruction<'ctx>>,
    position: usize,
    in_function: bool,
    in_loop: bool,
}

#[derive(Debug)]
pub struct ThrushInstruction<'ctx> {
    pub instr: Instruction<'ctx>,
    pub depth: usize,
}

impl<'ctx> ThrushScoper<'ctx> {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            count: 0,
        }
    }

    pub fn begin_scope(&mut self, instr: Instruction<'ctx>, in_function: bool, in_loop: bool) {
        self.count += 1;

        if let Instruction::Block(body) = instr {
            let mut instructions: Vec<ThrushInstruction> = Vec::with_capacity(body.len());

            for (depth, instr) in body.iter().enumerate() {
                instructions.push(ThrushInstruction {
                    instr: instr.clone(),
                    depth,
                });
            }

            self.blocks.push(ThrushBlock::new(
                instructions,
                self.count,
                in_function,
                in_loop,
            ));
        }
    }

    pub fn analyze(&self) -> Result<(), ThrushError> {
        for instr in &self.blocks[self.count - 1].instructions {
            self.analyze_instruction(&instr.instr, instr.depth)?;
        }

        Ok(())
    }

    fn analyze_instruction(
        &self,
        instr: &Instruction<'ctx>,
        depth: usize,
    ) -> Result<(), ThrushError> {
        if let Instruction::Block(body) = instr {
            for instr in body {
                self.analyze_instruction(instr, depth + 1)?;
            }
        }

        if let Instruction::Function { body, .. } = instr {
            self.analyze_instruction(body, depth)?;
        }

        if let Instruction::EntryPoint { body } = instr {
            self.analyze_instruction(body, depth)?;
        }

        if let Instruction::Print(params) = instr {
            for instr in params {
                self.analyze_instruction(instr, depth)?;
            }
        }

        match instr {
            Instruction::String(_) => Ok(()),
            Instruction::Var { .. } => Ok(()),
            Instruction::RefVar { name, .. } => {
                if !self.is_at_current_scope(name) && !self.is_at_top_scope(name) {
                    return Err(ThrushError::Compile(format!(
                        "Variable: `{}` is not defined.",
                        name
                    )));
                }

                if self.is_at_current_scope(name)
                    && !self.is_reacheable_at_curret_scope(name, depth)
                {
                    return Err(ThrushError::Compile(format!(
                        "Variable: `{}` is unreacheable in this scope.",
                        name
                    )));
                }

                if self.is_at_top_scope(name) && !self.is_reacheable_at_top_scope(name, depth) {
                    return Err(ThrushError::Compile(format!(
                        "Variable: `{}` is unreacheable in the top scope.",
                        name
                    )));
                }

                Ok(())
            }

            e => {
                println!("{:?}", e);

                todo!()
            }
        }
    }

    fn is_reacheable_at_curret_scope(&self, name: &str, depth: usize) -> bool {
        self.blocks[self.count - 1]
            .instructions
            .iter()
            .any(|instr| match instr.instr {
                Instruction::Var { name: n, .. } => *n == *name && depth >= instr.depth,
                _ => false,
            })
    }

    fn is_at_current_scope(&self, name: &str) -> bool {
        self.blocks[self.count - 1]
            .instructions
            .iter()
            .any(|instr| match instr.instr {
                Instruction::Var { name: n, .. } => *n == *name,
                _ => false,
            })
    }

    fn is_reacheable_at_top_scope(&self, name: &str, depth: usize) -> bool {
        for block in &self.blocks {
            if block.in_function || block.in_loop {
                continue;
            }

            if block.position == self.blocks[self.count - 1].position {
                return false;
            }

            if block.instructions.iter().any(|instr| match instr.instr {
                Instruction::Var { name: n, .. } => *n == *name && depth >= instr.depth,
                _ => false,
            }) {
                return true;
            }
        }

        false
    }

    fn is_at_top_scope(&self, name: &str) -> bool {
        for block in &self.blocks {
            if block.in_function || block.in_loop {
                continue;
            }

            if block.position == self.blocks[self.count - 1].position {
                return false;
            }

            if block.instructions.iter().any(|instr| match instr.instr {
                Instruction::Var { name: n, .. } => *n == *name,
                _ => false,
            }) {
                return true;
            }
        }

        false
    }

    pub fn end_scope(&mut self) {
        self.blocks.pop();
        self.count -= 1;
    }
}

impl<'ctx> ThrushBlock<'ctx> {
    pub fn new(
        instructions: Vec<ThrushInstruction<'ctx>>,
        position: usize,
        in_function: bool,
        in_loop: bool,
    ) -> Self {
        Self {
            instructions,
            position,
            in_function,
            in_loop,
        }
    }
}
