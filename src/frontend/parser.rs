use {
    super::{
        super::{
            backend::{compiler::options::CompilerOptions, instruction::Instruction},
            diagnostic::Diagnostic,
            error::{ThrushError, ThrushErrorKind},
            logging,
        },
        lexer::{DataTypes, Token, TokenKind},
        scoper::ThrushScoper,
        type_checking,
    },
    ahash::AHashMap as HashMap,
    std::{mem, process::exit},
};

const VALID_INTEGER_TYPES: [DataTypes; 8] = [
    DataTypes::U8,
    DataTypes::U16,
    DataTypes::U32,
    DataTypes::U64,
    DataTypes::I8,
    DataTypes::I16,
    DataTypes::I32,
    DataTypes::I64,
];

const VALID_FLOAT_TYPES: [DataTypes; 2] = [DataTypes::F32, DataTypes::F64];
const STANDARD_FORMATS: [&str; 5] = ["%s", "%d", "%c", "%ld", "%f"];

type ParserLocals<'instr> = Vec<HashMap<&'instr str, (DataTypes, bool, bool, usize)>>;

pub struct Parser<'instr, 'a> {
    stmts: Vec<Instruction<'instr>>,
    errors: Vec<ThrushError>,
    tokens: &'instr [Token],
    options: &'a CompilerOptions,
    function: u16,
    ret: Option<DataTypes>,
    current: usize,
    globals: HashMap<&'instr str, DataTypes>,
    locals: ParserLocals<'instr>,
    scope: usize,
    scoper: ThrushScoper<'instr>,
    diagnostic: Diagnostic,
    has_entry_point: bool,
}

impl<'instr, 'a> Parser<'instr, 'a> {
    pub fn new(options: &'a CompilerOptions, tokens: &'instr [Token]) -> Self {
        Self {
            stmts: Vec::new(),
            errors: Vec::new(),
            tokens,
            options,
            current: 0,
            ret: None,
            function: 0,
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            scope: 0,
            scoper: ThrushScoper::new(&options.file_path),
            diagnostic: Diagnostic::new(&options.file_path),
            has_entry_point: false,
        }
    }

    pub fn start(&mut self) -> &[Instruction<'instr>] {
        while !self.end() {
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
            self.errors.iter().for_each(|error| {
                self.diagnostic.report(error);
            });

            exit(1);
        } else if self.options.is_main && !self.has_entry_point {
            logging::log(
                logging::LogType::ERROR,
                "Missing entrypoint \"fn main() {}\" in main.th file.",
            );

            exit(1);
        }

        self.scoper.analyze();

        self.stmts.as_slice()
    }

    fn parse(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        match &self.peek().kind {
            TokenKind::Println => Ok(self.println()?),
            TokenKind::Print => Ok(self.print()?),
            TokenKind::Fn => Ok(self.function(false)?),
            TokenKind::LBrace => Ok(self.block(&mut [])?),
            TokenKind::Return => Ok(self.ret()?),
            TokenKind::Public => Ok(self.public()?),
            TokenKind::Var => Ok(self.variable(false)?),
            TokenKind::For => Ok(self.for_loop()?),
            _ => Ok(self.expr()?),
        }
    }

    fn for_loop(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let variable: Instruction<'instr> = self.variable(false)?;

        let cond: Instruction<'instr> = self.expr()?;

        let actions: Instruction<'instr> = self.expr()?;

        let mut variable_clone: Instruction<'instr> = variable.clone();

        if let Instruction::Var { comptime, .. } = &mut variable_clone {
            *comptime = true;
        }

        let body: Instruction<'instr> = self.block(&mut [variable_clone])?;

        Ok(Instruction::ForLoop {
            variable: Some(Box::new(variable)),
            cond: Some(Box::new(cond)),
            actions: Some(Box::new(actions)),
            block: Box::new(body),
        })
    }

    fn variable(&mut self, comptime: bool) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        let name: &'instr Token = self.consume(
            TokenKind::Identifier,
            ThrushErrorKind::SyntaxError,
            String::from("Expected variable name"),
            String::from("Expected var (name)."),
        )?;

        if self.peek().kind == TokenKind::SemiColon {
            self.only_advance()?;

            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected an type for the variable. You forget the `:`."),
                name.line,
            ));
        } else if self.peek().kind == TokenKind::Colon {
            self.consume(
                TokenKind::Colon,
                ThrushErrorKind::SyntaxError,
                String::from("Expected variable type indicator"),
                String::from("Expected `var name --> : <-- type = value;`."),
            )?;
        }

        let mut kind: Option<DataTypes> = match &self.peek().kind {
            TokenKind::DataType(kind) => {
                if self.previous().kind != TokenKind::Colon {
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Expected variable type indicator"),
                        String::from("Expected `var name --> : <-- type = value;`."),
                        name.line,
                    ));
                }

                self.only_advance()?;

                Some(*kind)
            }

            TokenKind::Eq => None,

            _ => {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected an type for the variable."),
                    name.line,
                ));
            }
        };

        if self.peek().kind == TokenKind::SemiColon && kind.is_none() {
            self.only_advance()?;

            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "Variable type is undefined. Did you forget to specify the variable type to undefined variable?",
                ),
                name.line,
            ));
        } else if self.peek().kind == TokenKind::SemiColon {
            match kind.as_ref().unwrap() {
                DataTypes::Integer => kind = Some(DataTypes::I32),
                DataTypes::Float => kind = Some(DataTypes::F32),
                _ => {}
            }

            self.consume(
                TokenKind::SemiColon,
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected ';'."),
            )?;

            self.define_local(
                name.lexeme.as_ref().unwrap(),
                (*kind.as_ref().unwrap(), true, false, 0),
            );

            return Ok(Instruction::Var {
                name: name.lexeme.as_ref().unwrap(),
                kind: kind.unwrap(),
                value: Box::new(Instruction::Null),
                line: name.line,
                comptime,
            });
        }

        self.consume(
            TokenKind::Eq,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '=' for the variable definition."),
        )?;

        let mut value: Instruction<'instr> = self.parse()?;

        if kind.is_some() {
            match &value {
                Instruction::Integer(data_type, _) => {
                    if kind.as_ref().unwrap() == &DataTypes::Integer {
                        if !VALID_INTEGER_TYPES.contains(data_type) {
                            return Err(ThrushError::Parse(
                                ThrushErrorKind::SyntaxError,
                                String::from("Syntax Error"),
                                format!(
                                    "Variable type mismatch. Expected '{}' but found '{}'.",
                                    kind.unwrap(),
                                    data_type
                                ),
                                name.line,
                            ));
                        }

                        kind = Some(*data_type);
                    }

                    if !kind.as_ref().unwrap().check(data_type) {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}'",
                                kind.unwrap(),
                                data_type
                            ),
                            name.line,
                        ));
                    }
                }

                Instruction::Float(data_type, _) => {
                    if kind.as_ref().unwrap() == &DataTypes::Float {
                        if !VALID_FLOAT_TYPES.contains(data_type) {
                            return Err(ThrushError::Parse(
                                ThrushErrorKind::SyntaxError,
                                String::from("Syntax Error"),
                                format!(
                                    "Variable type mismatch. Expected '{}' but found '{}'.",
                                    kind.unwrap(),
                                    data_type
                                ),
                                name.line,
                            ));
                        }

                        kind = Some(*data_type);
                    }

                    if !kind.as_ref().unwrap().check(data_type) {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}'",
                                kind.unwrap(),
                                data_type
                            ),
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
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}'.",
                                kind.as_ref().unwrap(),
                                DataTypes::String
                            ),
                            name.line,
                        ));
                    }
                }

                Instruction::Boolean(_) => {
                    if kind.as_ref().unwrap() != &DataTypes::Bool {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}'.",
                                kind.as_ref().unwrap(),
                                DataTypes::String
                            ),
                            name.line,
                        ));
                    }
                }

                Instruction::Char(_) => {
                    if kind.as_ref().unwrap() != &DataTypes::Char {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}'.",
                                kind.as_ref().unwrap(),
                                DataTypes::String
                            ),
                            name.line,
                        ));
                    }
                }

                Instruction::RefVar {
                    kind: refvar_kind, ..
                } => {
                    if !kind.as_ref().unwrap().check(refvar_kind) {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}'.",
                                kind.as_ref().unwrap(),
                                refvar_kind
                            ),
                            name.line,
                        ));
                    } else if kind.as_ref().unwrap().need_cast(refvar_kind)
                        && kind.as_ref().unwrap().is_unreachable_cast(refvar_kind)
                    {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            format!(
                                "Variable type cannot be cast into correct type. Use original type `{}` instead.",
                                refvar_kind,
                            ),
                            name.line,
                        ));
                    }
                }

                Instruction::Indexe {
                    origin,
                    kind: indexe_kind,
                    index,
                    ..
                } => {
                    if kind.as_ref().unwrap() != indexe_kind {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            format!(
                                "Variable type mismatch. Expected '{}' but found '{}'.",
                                kind.as_ref().unwrap(),
                                indexe_kind
                            ),
                            name.line,
                        ));
                    }

                    value = Instruction::Indexe {
                        origin,
                        name: Some(name.lexeme.as_ref().unwrap()),
                        index: *index,
                        kind: *indexe_kind,
                    }
                }

                Instruction::Unary { .. } | Instruction::Group { .. } => {}

                Instruction::Binary {
                    left,
                    op,
                    right,
                    kind: type_binary,
                    ..
                } => {
                    if let DataTypes::Integer = type_binary {
                        type_checking::check_binary_int_with_parent_instr(
                            kind.as_ref().unwrap(),
                            &left.get_data_type(),
                            op,
                            &right.get_data_type(),
                            name.line,
                        )?;
                    }
                }

                e => {
                    println!("{:?}", e);

                    todo!()
                }
            }
        }

        let variable: Instruction<'_> = if kind.as_ref().is_none() {
            Instruction::Var {
                name: name.lexeme.as_ref().unwrap(),
                kind: value.get_data_type(),
                value: Box::new(value),
                line: name.line,
                comptime,
            }
        } else {
            Instruction::Var {
                name: name.lexeme.as_ref().unwrap(),
                kind: kind.unwrap(),
                value: Box::new(value),
                line: name.line,
                comptime,
            }
        };

        self.define_local(
            name.lexeme.as_ref().unwrap(),
            (variable.get_kind().unwrap(), false, false, 0),
        );

        if self.match_token(TokenKind::SemiColon)? {
            self.only_advance()?;
        }

        Ok(variable)
    }

    fn public(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        match &self.peek().kind {
            TokenKind::Fn => Ok(self.function(true)?),
            _ => unimplemented!(),
        }
    }

    fn ret(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        if self.function == 0 {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Return statement outside of function. Invoke this keyword in scope of function definition."),
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
        with_instrs: &mut [Instruction<'instr>],
    ) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        self.begin_scope();

        let mut stmts: Vec<Instruction> = Vec::new();

        for instr in with_instrs.iter_mut() {
            stmts.push(mem::take(instr));
        }

        while !self.match_token(TokenKind::RBrace)? {
            stmts.push(self.parse()?)
        }

        let mut frees: Vec<Instruction> = Vec::new();

        for stmt in self.locals[self.scope].iter_mut() {
            if let (_, (DataTypes::String, false, false, 0)) = stmt {
                frees.push(Instruction::Free {
                    name: stmt.0,
                    is_string: true,
                });

                stmt.1 .2 = true;
            }
        }

        stmts.extend(frees);

        self.end_scope();

        self.scoper.add_scope(stmts.clone());

        Ok(Instruction::Block { stmts })
    }

    fn function(&mut self, is_public: bool) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        if self.scope != 0 {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "The functions must go in the global scope. Rewrite it in the global scope.",
                ),
                self.previous().line,
            ));
        }

        self.begin_function();

        let name: &'instr Token = self.consume(
            TokenKind::Identifier,
            ThrushErrorKind::SyntaxError,
            String::from("Expected function name"),
            String::from("Expected fn < name >."),
        )?;

        if name.lexeme.as_ref().unwrap() == "main" && self.options.is_main {
            if self.has_entry_point {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Duplicated EntryPoint"),
                    String::from("The language not support two entrypoints, remove one."),
                    name.line,
                ));
            }

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

            if self.peek().kind != TokenKind::LBrace {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected '{'."),
                    self.peek().line,
                ));
            }

            if self.peek().kind == TokenKind::LBrace {
                self.has_entry_point = true;

                return Ok(Instruction::EntryPoint {
                    body: Box::new(self.block(&mut [])?),
                });
            } else {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected 'block ({ ... })' for the function body."),
                    self.peek().line,
                ));
            }
        }

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
        )?;

        let mut params: Vec<Instruction> = Vec::with_capacity(8);

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if params.len() >= 8 {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::TooManyArguments,
                    String::from("Syntax Error"),
                    String::from("Too many arguments for the function. The maximum number of arguments is 8."),
                    self.peek().line,
                ));
            }

            if !self.match_token(TokenKind::Identifier)? {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected argument name."),
                    self.peek().line,
                ));
            }

            let ident: &str = self.previous().lexeme.as_ref().unwrap();

            if !self.match_token(TokenKind::ColonColon)? {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected '::'."),
                    self.peek().line,
                ));
            }

            let kind: DataTypes = match &self.peek().kind {
                TokenKind::DataType(kind) => {
                    self.only_advance()?;

                    *kind
                }
                _ => {
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Syntax Error"),
                        String::from("Expected argument type."),
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
                self.only_advance()?;
                Some(*kind)
            }
            _ => None,
        };

        let body: Box<Instruction> = Box::new(self.block(&mut [])?);

        if let Some(return_type) = &return_kind {
            if self.ret.is_none() {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    format!("Missing return statement with type '{}', you should add a return statement with type '{}'.", return_type, return_type),
                    name.line,
                ));
            } else if return_type != self.ret.as_ref().unwrap() {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    format!(
                        "Expected return type of '{}', found '{}'. You should write a return statement with type '{}'.",
                        return_type,
                        self.ret.as_ref().unwrap(),
                        return_type
                    ),
                    name.line,
                ));
            }
        }

        self.end_function();

        match &return_kind {
            Some(kind) => {
                self.define_global(name.lexeme.as_ref().unwrap(), *kind);
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
        self.only_advance()?;

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
        )?;

        let mut types: Vec<DataTypes> = Vec::with_capacity(24);
        let mut args: Vec<Instruction<'instr>> = Vec::with_capacity(24);

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if args.len() >= 24 || types.len() >= 24 {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::TooManyArguments,
                    String::from("Syntax Error"),
                    String::from("Expected ')'. Too many arguments. Max is 24."),
                    self.peek().line,
                ));
            }

            let expr: Instruction<'instr> = self.expr()?;

            if !args.is_empty() {
                types.push(expr.get_data_type());
            }

            args.push(expr);
        }

        if args.is_empty() && self.match_token(TokenKind::SemiColon)? {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from("Expected at least 1 argument for 'print' call. Like 'print(`Hi!`);'"),
                self.peek().line,
            ));
        } else if let Instruction::String(str) = &args[0] {
            if args.len() == 1 && STANDARD_FORMATS.iter().any(|fmt| str.trim().contains(*fmt)) {
                self.consume(
                    TokenKind::SemiColon,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected ';'."),
                )?;

                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from(
                        "Expected at least 2 arguments for 'println' call. Like 'print(`%d`, 2);'",
                    ),
                    self.previous().line,
                ));
            } else if types.len() != args.iter().skip(1).collect::<Vec<_>>().len() {
                self.consume(
                    TokenKind::SemiColon,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected ';'."),
                )?;

                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("The formating and arguments should be the same count."),
                    self.previous().line,
                ));
            }

            let formats: Vec<&str> = str
                .trim()
                .split("%")
                .skip(1)
                .filter_map(|fmt| {
                    STANDARD_FORMATS
                        .iter()
                        .find(|std_fmt| format!("%{}", fmt.trim()).contains(**std_fmt))
                        .copied()
                })
                .collect();

            if formats.len() != types.len() {
                self.consume(
                    TokenKind::SemiColon,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected ';'."),
                )?;

                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Argument without an specific formatter `%x`."),
                    self.previous().line,
                ));
            }

            for (index, kind) in types.iter().enumerate() {
                match kind {
                    DataTypes::String if formats[index] != "%s" => {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("The formating for string type is '%s'."),
                            self.previous().line,
                        ));
                    }

                    DataTypes::Char if formats[index] != "%c" => {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("The formating for char type is '%c'."),
                            self.previous().line,
                        ));
                    }

                    DataTypes::U8 | DataTypes::U16 | DataTypes::I8 | DataTypes::I16
                        if formats[index] != "%d" =>
                    {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from(
                                "The formating for integer type (8 bits - 16 bits) is '%d'.",
                            ),
                            self.previous().line,
                        ));
                    }

                    DataTypes::U32 | DataTypes::U64 | DataTypes::I32 | DataTypes::I64
                        if formats[index] != "%ld" =>
                    {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from(
                                "The formating for integer type (32 bits - 64 bits) is '%ld'.",
                            ),
                            self.previous().line,
                        ));
                    }

                    DataTypes::F32 | DataTypes::F64 if formats[index] != "%f" => {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from(
                                "The formating for float type (32 bits - 64 bits) is '%f'.",
                            ),
                            self.previous().line,
                        ));
                    }

                    _ => {}
                }
            }
        }

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
        )?;

        args.iter().try_for_each(|arg| match arg {
            Instruction::String(str) => {
                if str.contains("\n") {
                    return Err(ThrushError::Parse(
                        ThrushErrorKind::SyntaxError,
                        String::from("Syntax Error"),
                        String::from(
                            "You can't print strings that contain newlines. Use 'println' instead.",
                        ),
                        self.peek().line,
                    ));
                }

                Ok(())
            }
            _ => Ok(()),
        })?;

        Ok(Instruction::Print(args))
    }

    fn println(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.only_advance()?;

        self.consume(
            TokenKind::LParen,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected '('."),
        )?;

        let mut types: Vec<DataTypes> = Vec::with_capacity(24);
        let mut args: Vec<Instruction<'instr>> = Vec::with_capacity(24);

        while !self.match_token(TokenKind::RParen)? {
            if self.match_token(TokenKind::Comma)? {
                continue;
            }

            if args.len() >= 24 || types.len() >= 24 {
                return Err(ThrushError::Parse(
                    ThrushErrorKind::TooManyArguments,
                    String::from("Syntax Error"),
                    String::from("Expected ')'. Too many arguments. Max is 24."),
                    self.peek().line,
                ));
            }

            let expr: Instruction<'_> = match self.expr()? {
                Instruction::String(str) => {
                    if args.len() > 1 {
                        types.push(DataTypes::String);
                    }

                    Instruction::String(str)
                }
                expr => {
                    types.push(expr.get_data_type());
                    expr
                }
            };

            args.push(expr);
        }

        if args.is_empty() && self.match_token(TokenKind::SemiColon)? {
            return Err(ThrushError::Parse(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "Expected at least 1 argument for 'println' call. Like 'println(`Hi!`);'",
                ),
                self.peek().line,
            ));
        } else if let Instruction::String(str) = &args[0] {
            if args.len() == 1 && STANDARD_FORMATS.iter().any(|fmt| str.trim().contains(*fmt)) {
                self.consume(
                    TokenKind::SemiColon,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected ';'."),
                )?;

                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from(
                        "Expected at least 2 arguments for 'println' call. Like 'println(`%d`, 2);'",
                    ),

                    self.previous().line,
                ));
            } else if types.len() != args.iter().skip(1).collect::<Vec<_>>().len() {
                self.consume(
                    TokenKind::SemiColon,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected ';'."),
                )?;

                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("The formating and arguments should be the same count."),
                    self.previous().line,
                ));
            }

            let formats: Vec<&str> = str
                .trim()
                .split("%")
                .skip(1)
                .filter_map(|fmt| {
                    STANDARD_FORMATS
                        .iter()
                        .find(|std_fmt| format!("%{}", fmt.trim()).contains(**std_fmt))
                        .copied()
                })
                .collect();

            if formats.len() != types.len() {
                self.consume(
                    TokenKind::SemiColon,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected ';'."),
                )?;

                return Err(ThrushError::Parse(
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Argument without an specific formatter `%x`."),
                    self.previous().line,
                ));
            }

            for (index, kind) in types.iter().enumerate() {
                match kind {
                    DataTypes::String if formats[index] != "%s" => {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("The formating for string type is '%s'."),
                            self.previous().line,
                        ));
                    }

                    DataTypes::Char if formats[index] != "%c" => {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("The formating for char type is '%c'."),
                            self.previous().line,
                        ));
                    }

                    DataTypes::U8 | DataTypes::U16 | DataTypes::I8 | DataTypes::I16
                        if formats[index] != "%d" =>
                    {
                        self.consume(
                            TokenKind::SemiColon,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ';'."),
                        )?;

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from(
                                "The formating for integer type (8 bits - 16 bits) is '%d'.",
                            ),
                            self.previous().line,
                        ));
                    }

                    DataTypes::U32 | DataTypes::U64 | DataTypes::I32 | DataTypes::I64
                        if formats[index] != "%ld" =>
                    {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from(
                                "The formating for integer type (32 bits - 64 bits) is '%ld'.",
                            ),
                            self.previous().line,
                        ));
                    }

                    DataTypes::F32 | DataTypes::F64 if formats[index] != "%f" => {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from(
                                "The formating for float type (32 bits - 64 bits) is '%f'.",
                            ),
                            self.previous().line,
                        ));
                    }

                    _ => {}
                }
            }
        }

        self.consume(
            TokenKind::SemiColon,
            ThrushErrorKind::SyntaxError,
            String::from("Syntax Error"),
            String::from("Expected ';'."),
        )?;

        Ok(Instruction::Println(args))
    }

    fn expr(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let instr: Instruction = self.or()?;

        self.locals.iter_mut().for_each(|scope| {
            scope.values_mut().for_each(|variable| {
                if variable.3 > 0 {
                    variable.3 -= 1;
                }
            });
        });

        if self.peek().kind == TokenKind::SemiColon {
            self.only_advance()?;
        }

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

            instr = Instruction::Binary {
                left: Box::new(instr),
                op,
                right: Box::new(right),
                kind: DataTypes::Bool,
                line: self.previous().line,
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

            instr = Instruction::Binary {
                left: Box::new(instr),
                op,
                right: Box::new(right),
                kind: DataTypes::Bool,
                line: self.previous().line,
            }
        }

        Ok(instr)
    }

    fn equality(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction<'_> = self.comparison()?;

        while self.match_token(TokenKind::BangEq)? || self.match_token(TokenKind::EqEq)? {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction<'_> = self.comparison()?;

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
            )?;

            instr = Instruction::Binary {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind: DataTypes::Bool,
                line: self.previous().line,
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

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
            )?;

            instr = Instruction::Binary {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind: DataTypes::Bool,
                line: self.previous().line,
            };
        }

        Ok(instr)
    }

    fn term(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        let mut instr: Instruction<'_> = self.unary()?;

        while self.match_token(TokenKind::Plus)?
            || self.match_token(TokenKind::Minus)?
            || self.match_token(TokenKind::Slash)?
            || self.match_token(TokenKind::Star)?
        {
            let op: &TokenKind = &self.previous().kind;
            let right: Instruction<'_> = self.unary()?;

            type_checking::check_binary_instr(
                op,
                &instr.get_data_type(),
                &right.get_data_type(),
                self.previous().line,
            )?;

            instr = Instruction::Binary {
                left: Box::from(instr),
                op,
                right: Box::from(right),
                kind: DataTypes::Integer,
                line: self.previous().line,
            };
        }

        Ok(instr)
    }

    fn unary(&mut self) -> Result<Instruction<'instr>, ThrushError> {
        if self.match_token(TokenKind::Bang)? {
            let op: &TokenKind = &self.previous().kind;
            let value: Instruction<'instr> = self.primary()?;

            type_checking::check_unary_instr(op, &value.get_data_type(), self.previous().line)?;

            return Ok(Instruction::Unary {
                op,
                value: Box::from(value),
                kind: DataTypes::Bool,
            });
        } else if self.match_token(TokenKind::PlusPlus)?
            | self.match_token(TokenKind::MinusMinus)?
            | self.match_token(TokenKind::Minus)?
        {
            let op: &TokenKind = &self.previous().kind;
            let value: Instruction<'instr> = self.primary()?;
            let value_type: &DataTypes = &value.get_data_type();

            type_checking::check_unary_instr(op, value_type, self.previous().line)?;

            return Ok(Instruction::Unary {
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
                self.only_advance()?;

                let instr: Instruction<'instr> = self.expression()?;

                self.consume(
                    TokenKind::RParen,
                    ThrushErrorKind::SyntaxError,
                    String::from("Syntax Error"),
                    String::from("Expected ')'."),
                )?;

                return Ok(Instruction::Group {
                    instr: Box::new(instr),
                });
            }

            TokenKind::String => {
                Instruction::String(self.advance()?.lexeme.as_ref().unwrap().to_string())
            }
            TokenKind::Char => {
                Instruction::Char(self.advance()?.lexeme.as_ref().unwrap().as_bytes()[0])
            }

            kind => match kind {
                TokenKind::Integer(kind, num) => {
                    self.only_advance()?;

                    let instr: Instruction<'instr> = match kind {
                        DataTypes::I8 => Instruction::Integer(DataTypes::I8, *num),
                        DataTypes::I16 => Instruction::Integer(DataTypes::I16, *num),
                        DataTypes::I32 => Instruction::Integer(DataTypes::I32, *num),
                        DataTypes::I64 => Instruction::Integer(DataTypes::I64, *num),
                        DataTypes::U8 => Instruction::Integer(DataTypes::U8, *num),
                        DataTypes::U16 => Instruction::Integer(DataTypes::U16, *num),
                        DataTypes::U32 => Instruction::Integer(DataTypes::U32, *num),
                        DataTypes::U64 => Instruction::Integer(DataTypes::U64, *num),

                        _ => unreachable!(),
                    };

                    if self.match_token(TokenKind::PlusPlus)?
                        | self.match_token(TokenKind::MinusMinus)?
                    {
                        type_checking::check_unary_instr(
                            &self.previous().kind,
                            kind,
                            self.previous().line,
                        )?;

                        return Ok(Instruction::Unary {
                            op: &self.previous().kind,
                            value: Box::from(instr),
                            kind: *kind,
                        });
                    }

                    instr
                }

                TokenKind::Float(kind, num) => {
                    self.only_advance()?;

                    let instr: Instruction<'instr> = match kind {
                        DataTypes::F32 => Instruction::Float(DataTypes::F32, *num),
                        DataTypes::F64 => Instruction::Float(DataTypes::F64, *num),
                        _ => unreachable!(),
                    };

                    if self.match_token(TokenKind::PlusPlus)?
                        | self.match_token(TokenKind::MinusMinus)?
                    {
                        type_checking::check_unary_instr(
                            &self.previous().kind,
                            kind,
                            self.previous().line,
                        )?;

                        return Ok(Instruction::Unary {
                            op: &self.previous().kind,
                            value: Box::from(instr),
                            kind: *kind,
                        });
                    }

                    instr
                }

                TokenKind::Identifier => {
                    self.only_advance()?;

                    let var: (DataTypes, bool) =
                        self.find_and_get_variable(self.previous().lexeme.as_ref().unwrap())?;

                    if self.peek().kind == TokenKind::LeftBracket {
                        let name: &str = self.previous().lexeme.as_ref().unwrap();

                        self.consume(
                            TokenKind::LeftBracket,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected '['."),
                        )?;

                        let expr: Instruction<'instr> = self.primary()?;

                        self.consume(
                            TokenKind::RightBracket,
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected ']'."),
                        )?;

                        if var.1 {
                            return Err(ThrushError::Parse(
                                ThrushErrorKind::VariableNotDeclared,
                                String::from("Variable Not Declared"),
                                format!(
                                    "Variable `{}` is not declared for are use it. Declare the variable before of the use.",
                                    self.previous().lexeme.as_ref().unwrap(),
                                ),
                                self.previous().line,
                            ));
                        }

                        let kind: DataTypes = if var.0 == DataTypes::String {
                            DataTypes::Char
                        } else {
                            todo!()
                        };

                        if let Instruction::Integer(_, num) = expr {
                            return Ok(Instruction::Indexe {
                                origin: name,
                                name: None,
                                index: num as u64,
                                kind,
                            });
                        }

                        return Err(ThrushError::Parse(
                            ThrushErrorKind::SyntaxError,
                            String::from("Syntax Error"),
                            String::from("Expected unsigned number for the build an indexe."),
                            self.previous().line,
                        ));
                    } else if self.peek().kind == TokenKind::Eq {
                        let name: &str = self.previous().lexeme.as_ref().unwrap();
                        self.only_advance()?;

                        let expr: Instruction<'instr> = self.expr()?;

                        if expr.get_data_type() != var.0
                            && VALID_INTEGER_TYPES.contains(&var.0)
                            && expr.get_data_type() != DataTypes::Integer
                            && expr.get_data_type() != DataTypes::Float
                        {
                            return Err(ThrushError::Parse(
                                ThrushErrorKind::SyntaxError,
                                String::from("Syntax Error"),
                                format!(
                                    "Variable type mismatch. Expected '{}' but found '{}'.",
                                    var.0,
                                    expr.get_data_type()
                                ),
                                self.previous().line,
                            ));
                        }

                        self.locals[self.scope].insert(name, (var.0, false, false, 0));

                        return Ok(Instruction::MutVar {
                            name,
                            value: Box::new(expr),
                            kind: var.0,
                        });
                    }

                    if var.1 {
                        return Err(ThrushError::Parse(
                            ThrushErrorKind::VariableNotDeclared,
                            String::from("Variable Not Declared"),
                            format!(
                                "Variable `{}` is not declared for are use it. Declare the variable before of the use.",
                                self.previous().lexeme.as_ref().unwrap(),
                            ),
                            self.previous().line,
                        ));
                    }

                    let refvar: Instruction<'_> = Instruction::RefVar {
                        name: self.previous().lexeme.as_ref().unwrap(),
                        line: self.previous().line,
                        kind: var.0,
                    };

                    if self.match_token(TokenKind::PlusPlus)?
                        | self.match_token(TokenKind::MinusMinus)?
                    {
                        type_checking::check_unary_instr(
                            &self.previous().kind,
                            &refvar.get_data_type(),
                            self.previous().line,
                        )?;

                        let expr: Instruction<'_> = Instruction::Unary {
                            op: &self.previous().kind,
                            value: Box::from(refvar),
                            kind: DataTypes::Integer,
                        };

                        if self.peek().kind == TokenKind::SemiColon {
                            self.only_advance()?;
                        }

                        return Ok(expr);
                    }

                    refvar
                }

                TokenKind::True => {
                    self.only_advance()?;

                    Instruction::Boolean(true)
                }

                TokenKind::False => {
                    self.only_advance()?;

                    Instruction::Boolean(false)
                }

                err => {
                    self.only_advance()?;

                    println!("{:?}", err);

                    Instruction::Null
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
            return self.advance();
        }

        Err(ThrushError::Parse(
            error_kind,
            error_title,
            help,
            self.peek().line,
        ))
    }

    #[inline]
    fn find_and_get_variable(
        &mut self,
        name: &'instr str,
    ) -> Result<(DataTypes, bool), ThrushError> {
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                // DataTypes, bool <- (is_null), bool <- (is_freeded), usize <- (number of references)
                let mut var: (DataTypes, bool, bool, usize) = *scope.get(name).unwrap();

                var.3 += 1;

                scope.insert(name, var);

                return Ok((var.0, var.1));
            }
        }

        if self.globals.contains_key(name) {
            return Ok((*self.globals.get(name).unwrap(), false));
        }

        Err(ThrushError::Parse(
            ThrushErrorKind::VariableNotDefined,
            String::from("Variable Not Defined"),
            format!("The variable `{}` is not defined in this scope.", name),
            self.previous().line,
        ))
    }

    #[inline]
    fn define_global(&mut self, name: &'instr str, kind: DataTypes) {
        self.globals.insert(name, kind);
    }

    #[inline]
    fn define_local(&mut self, name: &'instr str, value: (DataTypes, bool, bool, usize)) {
        self.locals[self.scope].insert(name, value);
    }

    #[inline]
    fn begin_scope(&mut self) {
        self.scope += 1;
        self.locals.push(HashMap::new());
    }

    #[inline]
    fn end_scope(&mut self) {
        self.scope -= 1;
        self.locals.pop();
    }

    #[inline]
    fn begin_function(&mut self) {
        self.function += 1;
    }

    #[inline]
    fn end_function(&mut self) {
        self.function -= 1;
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
        ))
    }

    fn sync(&mut self) {
        if !self.end() {
            self.current += 1;
        }

        while !self.end() {
            if self.previous().kind == TokenKind::SemiColon {
                return;
            }

            match self.peek().kind {
                TokenKind::Var | TokenKind::Fn => return,
                _ => (),
            }

            self.current += 1;
        }
    }

    #[inline]
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
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
