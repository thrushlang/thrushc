use {
    super::{
        super::{
            error::ThrushError,
            frontend::{
                lexer::{DataTypes, TokenKind},
                types::StructFieldsParser,
            },
        },
        compiler::{
            objects::CompilerObjects,
            types::{BinaryOp, Function, StructFields},
            utils,
        },
    },
    inkwell::{
        context::Context,
        types::{BasicTypeEnum, StructType},
        values::BasicValueEnum,
        AddressSpace,
    },
};

#[derive(Debug, Clone, Default)]
pub enum Instruction<'ctx> {
    BasicValueEnum(BasicValueEnum<'ctx>),
    DataTypes(DataTypes),
    Struct {
        name: String,
        types: StructFields<'ctx>,
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
    InitStruct {
        name: String,
        fields: StructFieldsParser<'ctx>,
        kind: DataTypes,
    },
    Str(String),
    Char(u8),
    ForLoop {
        variable: Box<Instruction<'ctx>>,
        cond: Box<Instruction<'ctx>>,
        actions: Box<Instruction<'ctx>>,
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
        position: u32,
        line: usize,
    },
    Function {
        name: String,
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
        name: String,
        instr: Box<Instruction<'ctx>>,
        kind: TokenKind,
    },
    Boolean(bool),
    Pass,
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
        from_fields: Option<&StructFields>,
        compiler_objects: &mut CompilerObjects<'ctx>,
    ) -> StructType<'ctx> {
        if let Some(from_fields) = from_fields {
            return self.build_struct_from_fields(context, from_fields);
        }

        if let Instruction::InitStruct { fields, .. } = self {
            let mut new_fields: Vec<(DataTypes, u32)> = Vec::with_capacity(fields.len());

            fields.iter().for_each(|field| {
                new_fields.push((field.2, field.3));
            });

            return self.build_struct_from_fields(context, &new_fields);
        }

        if let Instruction::RefVar { struct_type, .. } = self {
            let fields: &StructFields = compiler_objects.get_struct_fields(struct_type);

            return self.build_struct_from_fields(context, fields);
        }

        if let Instruction::Call { struct_type, .. } = self {
            let fields: &StructFields = compiler_objects.get_struct_fields(struct_type);

            return self.build_struct_from_fields(context, fields);
        }

        unreachable!()
    }

    pub fn is_chained(&self, other: &Instruction, line: usize) -> Result<(), ThrushError> {
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
            line,
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
    pub fn return_with_ptr(&self) -> Option<&'ctx str> {
        if let Instruction::Return(instr, _) = self {
            if let Instruction::RefVar { name, kind, .. } = instr.as_ref() {
                if kind.is_ptr_type() {
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

    pub fn as_extern(&self) -> (&str, &Instruction, &TokenKind) {
        if let Instruction::Extern { name, instr, kind } = self {
            return (name, instr, kind);
        }

        unreachable!()
    }

    pub fn as_function(&self) -> Function {
        if let Instruction::Function {
            name,
            params,
            body,
            return_type,
            is_public,
        } = self
        {
            return (name, params, body.as_ref(), return_type, *is_public);
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
            Instruction::InitStruct { kind: datatype, .. } => *datatype,

            Instruction::UnaryOp { value, .. } => value.get_data_type(),

            e => {
                debug_assert!(false, "Unexpected instruction: {:?}", e);
                unimplemented!()
            }
        }
    }

    pub fn get_data_type_recursive(&self) -> DataTypes {
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

        unimplemented!()
    }

    pub fn as_basic_value(&self) -> &BasicValueEnum<'ctx> {
        match self {
            Instruction::BasicValueEnum(value) => value,
            _ => unreachable!(),
        }
    }

    fn build_struct_from_fields(
        &self,
        context: &'ctx Context,
        fields: &StructFields,
    ) -> StructType<'ctx> {
        let mut compiled_field_types: Vec<BasicTypeEnum> = Vec::new();

        fields.iter().for_each(|field| {
            if field.0.is_integer_type() {
                compiled_field_types
                    .push(utils::datatype_integer_to_llvm_type(context, &field.0).into());
            }

            if field.0.is_float_type() {
                compiled_field_types
                    .push(utils::datatype_float_to_llvm_type(context, &field.0).into());
            }

            if field.0 == DataTypes::Bool {
                compiled_field_types.push(context.bool_type().into());
            }

            if field.0.is_ptr_type() {
                compiled_field_types.push(context.ptr_type(AddressSpace::default()).into());
            }
        });

        context.struct_type(&compiled_field_types, false)
    }
}
