use {
    super::{
        super::frontend::lexer::{DataTypes, TokenKind},
        compiler::types::BinaryOp,
    },
    inkwell::values::BasicValueEnum,
};

#[derive(Debug, Clone, Default)]
pub enum Instruction<'ctx> {
    BasicValueEnum(BasicValueEnum<'ctx>),
    Println(Vec<Instruction<'ctx>>),
    Print(Vec<Instruction<'ctx>>),
    String(String, bool),
    Char(u8),
    ForLoop {
        variable: Option<Box<Instruction<'ctx>>>,
        cond: Option<Box<Instruction<'ctx>>>,
        actions: Option<Box<Instruction<'ctx>>>,
        block: Box<Instruction<'ctx>>,
    },
    Integer(DataTypes, f64, bool),
    Float(DataTypes, f64, bool),
    Block {
        stmts: Vec<Instruction<'ctx>>,
    },
    EntryPoint {
        body: Box<Instruction<'ctx>>,
    },
    Param {
        name: String,
        kind: DataTypes,
    },
    Function {
        name: String,
        params: Vec<Instruction<'ctx>>,
        body: Option<Box<Instruction<'ctx>>>,
        return_kind: Option<DataTypes>,
        is_public: bool,
    },
    Return(Box<Instruction<'ctx>>, DataTypes),
    Var {
        name: &'ctx str,
        kind: DataTypes,
        value: Box<Instruction<'ctx>>,
        line: usize,
        only_comptime: bool,
    },
    RefVar {
        name: &'ctx str,
        line: usize,
        kind: DataTypes,
    },
    MutVar {
        name: &'ctx str,
        kind: DataTypes,
        value: Box<Instruction<'ctx>>,
    },
    Indexe {
        origin: &'ctx str,
        index: u64,
        kind: DataTypes,
    },
    Call {
        name: &'ctx str,
        args: Vec<Instruction<'ctx>>,
        kind: DataTypes,
    },
    BinaryOp {
        left: Box<Instruction<'ctx>>,
        op: &'ctx TokenKind,
        right: Box<Instruction<'ctx>>,
        kind: DataTypes,
    },
    UnaryOp {
        op: &'ctx TokenKind,
        value: Box<Instruction<'ctx>>,
        kind: DataTypes,
    },
    Group {
        instr: Box<Instruction<'ctx>>,
        kind: DataTypes,
    },
    Free {
        name: &'ctx str,
        free_only: bool,
        is_string: bool,
    },
    Extern {
        name: String,
        data: Box<Instruction<'ctx>>,
        kind: TokenKind,
    },
    Boolean(bool),
    Pass,
    #[default]
    Null,
}

impl PartialEq for Instruction<'_> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Instruction::Integer(_, _, _) => {
                matches!(other, Instruction::Integer(_, _, _))
            }

            Instruction::Float(_, _, _) => {
                matches!(other, Instruction::Float(_, _, _))
            }

            Instruction::String(_, _) => {
                matches!(other, Instruction::String(_, _))
            }

            _ => self == other,
        }
    }
}

impl<'ctx> Instruction<'ctx> {
    #[inline]
    pub fn is_extern(&self) -> bool {
        if let Instruction::Extern { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        if let Instruction::Function { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub fn is_binary(&self) -> bool {
        if let Instruction::BinaryOp { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub fn is_group(&self) -> bool {
        if let Instruction::Group { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub fn is_var(&self) -> bool {
        if let Instruction::Var { .. } | Instruction::RefVar { .. } = self {
            return true;
        }
        false
    }

    #[inline]
    pub fn is_indexe_return_of_string(&self) -> bool {
        if self.is_return() {
            if let Instruction::Return(indexe, DataTypes::Char) = self {
                return indexe.is_indexe();
            }

            return false;
        }

        false
    }

    #[inline]
    pub fn is_indexe(&self) -> bool {
        if let Instruction::Indexe { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub fn is_return(&self) -> bool {
        if let Instruction::Return(_, _) = self {
            return true;
        }

        false
    }

    pub fn as_extern(&self) -> (&str, &Instruction, TokenKind) {
        if let Instruction::Extern { name, data, kind } = self {
            return (name, data, *kind);
        }

        unreachable!()
    }

    pub fn as_function(
        &self,
    ) -> (
        &str,
        &[Instruction],
        &Option<Box<Instruction>>,
        &Option<DataTypes>,
        bool,
    ) {
        if let Instruction::Function {
            name,
            params,
            body,
            return_kind,
            is_public,
        } = self
        {
            return (name, params, body, return_kind, *is_public);
        }

        unreachable!()
    }

    pub fn as_binary(&self) -> BinaryOp {
        if let Instruction::BinaryOp {
            left, op, right, ..
        } = self
        {
            return (&**left, op, &**right);
        }

        if let Instruction::Group { instr, .. } = self {
            return instr.as_binary();
        }

        unreachable!()
    }

    pub fn get_data_type(&self) -> DataTypes {
        match self {
            Instruction::Integer(datatype, _, _) => *datatype,
            Instruction::Float(datatype, _, _) => *datatype,
            Instruction::String(_, _) => DataTypes::String,
            Instruction::Boolean(_) => DataTypes::Bool,
            Instruction::Char(_) => DataTypes::Char,
            Instruction::RefVar { kind, .. } => *kind,
            Instruction::Group { kind, .. } => *kind,
            Instruction::BinaryOp { kind, .. } => *kind,
            Instruction::UnaryOp { value, .. } => value.get_data_type(),
            Instruction::Param { kind, .. } => *kind,
            Instruction::Call { kind, .. } => *kind,
            Instruction::Indexe { kind, .. } => *kind,
            e => {
                println!("{:?}", e);

                unimplemented!()
            }
        }
    }

    pub fn as_basic_value(&self) -> &BasicValueEnum<'ctx> {
        match self {
            Instruction::BasicValueEnum(value) => value,
            _ => unreachable!(),
        }
    }
}
