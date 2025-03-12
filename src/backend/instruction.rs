use {
    super::{
        super::{
            error::ThrushError,
            frontend::{
                lexer::{DataTypes, TokenKind},
                types::StructFields,
            },
        },
        compiler::{
            objects::CompilerObjects,
            types::{BinaryOp, Function, Struct},
            utils,
        },
    },
    inkwell::{context::Context, types::StructType, values::BasicValueEnum},
};

#[derive(Debug, Clone, Default)]
pub enum Instruction<'ctx> {
    BasicValueEnum(BasicValueEnum<'ctx>),
    Str(String),
    Char(u8),
    Boolean(bool),
    DataTypes(DataTypes),
    Struct {
        name: &'ctx str,
        types: Struct<'ctx>,
    },
    InitStruct {
        name: &'ctx str,
        fields: StructFields<'ctx>,
        kind: DataTypes,
    },
    If {
        cond: Box<Instruction<'ctx>>,
        block: Box<Instruction<'ctx>>,
        elfs: Option<Vec<Instruction<'ctx>>>,
        otherwise: Option<Box<Instruction<'ctx>>>,
    },
    Elif {
        cond: Box<Instruction<'ctx>>,
        block: Box<Instruction<'ctx>>,
    },
    Else {
        block: Box<Instruction<'ctx>>,
    },
    ForLoop {
        variable: Box<Instruction<'ctx>>,
        cond: Box<Instruction<'ctx>>,
        actions: Box<Instruction<'ctx>>,
        block: Box<Instruction<'ctx>>,
    },
    Continue,
    Break,
    Integer(DataTypes, f64, bool),
    Float(DataTypes, f64, bool),
    Block {
        stmts: Vec<Instruction<'ctx>>,
    },
    EntryPoint {
        body: Box<Instruction<'ctx>>,
    },
    Param {
        name: &'ctx str,
        kind: DataTypes,
        position: u32,
        line: usize,
    },
    Function {
        name: &'ctx str,
        params: Vec<Instruction<'ctx>>,
        body: Option<Box<Instruction<'ctx>>>,
        return_type: DataTypes,
        is_public: bool,
    },
    Return(Box<Instruction<'ctx>>, DataTypes),
    Var {
        name: &'ctx str,
        kind: DataTypes,
        value: Box<Instruction<'ctx>>,
        line: usize,
        exist_only_comptime: bool,
    },
    RefVar {
        name: &'ctx str,
        line: usize,
        kind: DataTypes,
        struct_type: String,
    },
    MutVar {
        name: &'ctx str,
        kind: DataTypes,
        value: Box<Instruction<'ctx>>,
    },
    Call {
        name: &'ctx str,
        args: Vec<Instruction<'ctx>>,
        kind: DataTypes,
        struct_type: String,
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
        struct_type: String,
    },
    Extern {
        name: &'ctx str,
        instr: Box<Instruction<'ctx>>,
        kind: TokenKind,
    },
    Pass,
    NullPtr,
    #[default]
    Null,
}

impl PartialEq for Instruction<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Instruction::Integer(_, _, _), Instruction::Integer(_, _, _))
            | (Instruction::Float(_, _, _), Instruction::Float(_, _, _))
            | (Instruction::Str(_), Instruction::Str(_)) => true,
            (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
        }
    }
}

impl<'ctx> Instruction<'ctx> {
    pub fn build_struct_type(
        &self,
        context: &'ctx Context,
        struct_fields: Option<&Struct>,
        compiler_objects: &CompilerObjects,
    ) -> StructType<'ctx> {
        if let Some(from_fields) = struct_fields {
            return utils::build_struct_type_from_fields(context, from_fields);
        }

        if let Instruction::InitStruct { fields, .. } = self {
            let mut new_fields: Vec<(&'ctx str, DataTypes, u32)> = Vec::with_capacity(fields.len());

            fields.iter().for_each(|field| {
                new_fields.push((field.0, field.2, field.3));
            });

            return utils::build_struct_type_from_fields(context, &new_fields);
        }

        if let Instruction::RefVar { struct_type, .. } = self {
            let fields: &Struct = compiler_objects.get_struct(struct_type).unwrap();
            return utils::build_struct_type_from_fields(context, fields);
        }

        if let Instruction::Call { struct_type, .. } = self {
            let fields: &Struct = compiler_objects.get_struct(struct_type).unwrap();
            return utils::build_struct_type_from_fields(context, fields);
        }

        unreachable!()
    }

    pub fn is_chained(
        &self,
        other: &Instruction,
        location: (usize, (usize, usize)),
    ) -> Result<(), ThrushError> {
        if let (Instruction::BinaryOp { .. }, Instruction::BinaryOp { .. }) = (self, other) {
            return Ok(());
        }

        if let (Instruction::BinaryOp { .. }, Instruction::Group { .. }) = (self, other) {
            return Ok(());
        }

        if let (Instruction::Group { .. }, Instruction::BinaryOp { .. }) = (self, other) {
            return Ok(());
        }

        if let (Instruction::Group { .. }, Instruction::Group { .. }) = (self, other) {
            return Ok(());
        }

        if let (
            Instruction::RefVar { .. }
            | Instruction::Char { .. }
            | Instruction::Float { .. }
            | Instruction::Integer { .. }
            | Instruction::Boolean(_),
            Instruction::Char { .. }
            | Instruction::Float { .. }
            | Instruction::Integer { .. }
            | Instruction::RefVar { .. }
            | Instruction::Boolean(_),
        ) = (self, other)
        {
            return Ok(());
        }

        println!("{:?} {:?}", self, other);

        Err(ThrushError::Error(
            String::from("Type Checking"),
            String::from("Operators cannot be chained. Use logical gates as \"&&\" or \"||\"."),
            location.0,
            Some(location.1),
        ))
    }

    #[inline]
    pub fn has_return(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            stmts.iter().any(|stmt| stmt.is_return())
        } else {
            false
        }
    }

    #[inline]
    pub fn has_break(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            stmts.iter().any(|stmt| stmt.is_break())
        } else {
            false
        }
    }

    #[inline]
    pub fn has_continue(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            stmts.iter().any(|stmt| stmt.is_continue())
        } else {
            false
        }
    }

    #[inline]
    pub fn return_with_ptr(&self) -> Option<&'ctx str> {
        if let Instruction::Return(instr, _) = self {
            if let Instruction::RefVar { name, kind, .. } = instr.as_ref() {
                if kind.is_ptr_heaped() {
                    return Some(name);
                }
            }
        }

        None
    }

    #[inline]
    pub const fn is_extern(&self) -> bool {
        if let Instruction::Extern { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub const fn is_function(&self) -> bool {
        if let Instruction::Function { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub const fn is_binary(&self) -> bool {
        if let Instruction::BinaryOp { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub const fn is_group(&self) -> bool {
        if let Instruction::Group { .. } = self {
            return true;
        }

        false
    }

    #[inline]
    pub const fn is_return(&self) -> bool {
        if let Instruction::Return(_, _) = self {
            return true;
        }

        false
    }

    #[inline]
    pub const fn is_break(&self) -> bool {
        if let Instruction::Break = self {
            return true;
        }

        false
    }

    #[inline]
    pub const fn is_continue(&self) -> bool {
        if let Instruction::Continue = self {
            return true;
        }

        false
    }

    #[inline]
    pub const fn as_extern(&self) -> (&str, &Instruction, &TokenKind) {
        if let Instruction::Extern { name, instr, kind } = self {
            return (name, instr, kind);
        }

        unreachable!()
    }

    #[inline]
    pub fn as_function(&self) -> Function {
        if let Instruction::Function {
            name,
            params,
            body,
            return_type,
            is_public,
        } = self
        {
            return (name, params, body.as_ref(), return_type, is_public);
        }

        unreachable!()
    }

    #[inline]
    pub const fn as_binary(&self) -> BinaryOp {
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

    #[inline]
    pub const fn as_basic_value(&self) -> &BasicValueEnum<'ctx> {
        match self {
            Instruction::BasicValueEnum(value) => value,
            _ => unreachable!(),
        }
    }

    pub const fn get_data_type_recursive(&self) -> DataTypes {
        if let Instruction::BinaryOp { left, .. } = self {
            return left.get_data_type_recursive();
        }

        if let Instruction::UnaryOp { value, .. } = self {
            return value.get_data_type_recursive();
        }

        if let Instruction::Group { instr, .. } = self {
            return instr.get_data_type_recursive();
        }

        if let Instruction::RefVar {
            kind: refvar_type, ..
        } = self
        {
            return *refvar_type;
        }

        if let Instruction::Call { kind, .. } = self {
            return *kind;
        }

        if let Instruction::Integer(integer_type, _, _) = self {
            return *integer_type;
        }

        if let Instruction::Float(float_type, _, _) = self {
            return *float_type;
        }

        if let Instruction::Char(_) = self {
            return DataTypes::Char;
        }

        if let Instruction::Str(_) = self {
            return DataTypes::Str;
        }

        if let Instruction::Boolean(_) = self {
            return DataTypes::Bool;
        }

        if let Instruction::NullPtr = self {
            return DataTypes::Ptr;
        }

        unimplemented!()
    }

    pub fn get_data_type(&self) -> DataTypes {
        match self {
            Instruction::Integer(datatype, ..)
            | Instruction::Float(datatype, ..)
            | Instruction::RefVar { kind: datatype, .. }
            | Instruction::Group { kind: datatype, .. }
            | Instruction::BinaryOp { kind: datatype, .. }
            | Instruction::Param { kind: datatype, .. }
            | Instruction::Call { kind: datatype, .. }
            | Instruction::DataTypes(datatype) => *datatype,

            Instruction::Str(_) => DataTypes::Str,
            Instruction::Boolean(_) => DataTypes::Bool,
            Instruction::Char(_) => DataTypes::Char,
            Instruction::NullPtr => DataTypes::Ptr,
            Instruction::InitStruct { kind: datatype, .. } => *datatype,

            Instruction::UnaryOp { value, .. } => value.get_data_type(),

            e => {
                println!("{:?}", e);
                unimplemented!()
            }
        }
    }
}
