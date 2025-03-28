#![allow(clippy::upper_case_acronyms)]

use {
    super::{
        super::{
            super::error::ThrushCompilerError,
            super::frontend::{
                lexer::{TokenKind, Type},
                types::StructFields,
            },
        },
        objects::CompilerObjects,
        types::{BinaryOp, CompilerAttributes, Function, Struct},
        utils,
    },
    inkwell::{context::Context, types::StructType, values::BasicValueEnum},
};

#[derive(Debug, Clone, Default)]
pub enum Instruction<'ctx> {
    // Primitive types
    Str(Vec<u8>),
    Char(u8),
    Boolean(bool),
    Integer(Type, f64, bool),
    Float(Type, f64, bool),
    Struct {
        name: &'ctx str,
        fields_types: Struct<'ctx>,
    },
    NullT,

    LLVMValue(BasicValueEnum<'ctx>),

    Type(Type),

    InitStruct {
        name: &'ctx str,
        fields: StructFields<'ctx>,
        kind: Type,
    },

    // Conditionals
    If {
        cond: Box<Instruction<'ctx>>,
        block: Box<Instruction<'ctx>>,
        elfs: Vec<Instruction<'ctx>>,
        otherwise: Option<Box<Instruction<'ctx>>>,
    },
    Elif {
        cond: Box<Instruction<'ctx>>,
        block: Box<Instruction<'ctx>>,
    },
    Else {
        block: Box<Instruction<'ctx>>,
    },

    // Loops
    ForLoop {
        variable: Box<Instruction<'ctx>>,
        cond: Box<Instruction<'ctx>>,
        actions: Box<Instruction<'ctx>>,
        block: Box<Instruction<'ctx>>,
    },
    WhileLoop {
        cond: Box<Instruction<'ctx>>,
        block: Box<Instruction<'ctx>>,
    },
    Loop {
        block: Box<Instruction<'ctx>>,
    },

    // Loop control flow
    Continue,
    Break,

    // Code block
    Block {
        stmts: Vec<Instruction<'ctx>>,
    },

    // Functions

    // Entrypoint -> fn main() {}
    EntryPoint {
        body: Box<Instruction<'ctx>>,
    },

    FunctionParameter {
        name: &'ctx str,
        kind: Type,
        struct_type: String,
        position: u32,
        line: usize,
        span: (usize, usize),
    },
    Function {
        name: &'ctx str,
        params: Vec<Instruction<'ctx>>,
        body: Option<Box<Instruction<'ctx>>>,
        return_type: Type,
        attributes: CompilerAttributes<'ctx>,
    },
    Return(Box<Instruction<'ctx>>, Type),

    // Locals variables
    Local {
        name: &'ctx str,
        kind: Type,
        value: Box<Instruction<'ctx>>,
        line: usize,
        exist_only_comptime: bool,
    },
    LocalRef {
        name: &'ctx str,
        line: usize,
        kind: Type,
        struct_type: String,
    },
    LocalMut {
        name: &'ctx str,
        kind: Type,
        value: Box<Instruction<'ctx>>,
    },

    // Pointer
    GEP {
        name: &'ctx str,
        index: Box<Instruction<'ctx>>,
    },

    // Expressions
    Call {
        name: &'ctx str,
        args: Vec<Instruction<'ctx>>,
        kind: Type,
        struct_type: String,
    },
    BinaryOp {
        left: Box<Instruction<'ctx>>,
        op: &'ctx TokenKind,
        right: Box<Instruction<'ctx>>,
        kind: Type,
    },
    UnaryOp {
        op: &'ctx TokenKind,
        value: Box<Instruction<'ctx>>,
        kind: Type,
        is_pre: bool,
    },
    Group {
        instr: Box<Instruction<'ctx>>,
        kind: Type,
    },

    // Heap deallocator
    Free {
        name: &'ctx str,
        struct_type: String,
    },

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
            let mut new_fields: Vec<(&'ctx str, Type, u32)> = Vec::with_capacity(fields.len());

            fields.iter().for_each(|field| {
                new_fields.push((field.0, field.2, field.3));
            });

            return utils::build_struct_type_from_fields(context, &new_fields);
        }

        if let Instruction::LocalRef { struct_type, .. } = self {
            let fields: &Struct = compiler_objects.get_struct(struct_type).unwrap();
            return utils::build_struct_type_from_fields(context, fields);
        }

        if let Instruction::Call { struct_type, .. } = self {
            let fields: &Struct = compiler_objects.get_struct(struct_type).unwrap();
            return utils::build_struct_type_from_fields(context, fields);
        }

        unreachable!()
    }

    #[inline]
    pub fn has_more_than_a_statement(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            return !stmts.is_empty();
        }

        false
    }

    #[inline]
    pub fn is_chained(
        &self,
        other: &Instruction,
        location: (usize, (usize, usize)),
    ) -> Result<(), ThrushCompilerError> {
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
            Instruction::LocalRef { .. }
            | Instruction::Char { .. }
            | Instruction::Float { .. }
            | Instruction::Integer { .. }
            | Instruction::Boolean(_)
            | Instruction::Call { .. },
            Instruction::Char { .. }
            | Instruction::Float { .. }
            | Instruction::Integer { .. }
            | Instruction::LocalRef { .. }
            | Instruction::Boolean(_)
            | Instruction::Call { .. },
        ) = (self, other)
        {
            return Ok(());
        }

        Err(ThrushCompilerError::Error(
            String::from("Type Checking"),
            String::from("Operators cannot be chained. Use logical gates as '&&' or '||'."),
            location.0,
            Some(location.1),
        ))
    }

    #[inline]
    pub fn return_with_heaped_ptr(&self) -> Option<&'ctx str> {
        if let Instruction::Return(instr, _) = self {
            if let Instruction::LocalRef { name, kind, .. } = instr.as_ref() {
                if kind.is_heaped_ptr() {
                    return Some(name);
                }
            }
        }

        None
    }

    #[inline(always)]
    pub fn as_function(&self) -> Function {
        if let Instruction::Function {
            name,
            params,
            body,
            return_type,
            attributes,
        } = self
        {
            return (name, return_type, params, body.as_ref(), attributes);
        }

        unreachable!()
    }

    #[inline(always)]
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

    #[inline(always)]
    pub const fn as_llvm_value(&self) -> &BasicValueEnum<'ctx> {
        match self {
            Instruction::LLVMValue(value) => value,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub const fn get_data_type_recursive(&self) -> Type {
        if let Instruction::BinaryOp { left, .. } = self {
            return left.get_data_type_recursive();
        }

        if let Instruction::UnaryOp { value, kind, .. } = self {
            return if !kind.is_void_type() {
                value.get_data_type_recursive()
            } else {
                *kind
            };
        }

        if let Instruction::Group { instr, .. } = self {
            return instr.get_data_type_recursive();
        }

        if let Instruction::LocalRef {
            kind: localref_type,
            ..
        } = self
        {
            return *localref_type;
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
            return Type::Char;
        }

        if let Instruction::Str(_) = self {
            return Type::Str;
        }

        if let Instruction::Boolean(_) = self {
            return Type::Bool;
        }

        if let Instruction::NullT | Instruction::GEP { .. } = self {
            return Type::T;
        }

        if let Instruction::LocalMut { .. } = self {
            return Type::Void;
        }

        unimplemented!()
    }

    pub fn get_data_type(&self) -> Type {
        match self {
            Instruction::Integer(datatype, ..)
            | Instruction::Float(datatype, ..)
            | Instruction::LocalRef { kind: datatype, .. }
            | Instruction::Group { kind: datatype, .. }
            | Instruction::BinaryOp { kind: datatype, .. }
            | Instruction::FunctionParameter { kind: datatype, .. }
            | Instruction::Call { kind: datatype, .. }
            | Instruction::LocalMut { kind: datatype, .. }
            | Instruction::Type(datatype) => *datatype,

            Instruction::Str(_) => Type::Str,
            Instruction::Boolean(_) => Type::Bool,
            Instruction::Char(_) => Type::Char,
            Instruction::NullT => Type::T,
            Instruction::GEP { .. } => Type::T,
            Instruction::InitStruct { kind: datatype, .. } => *datatype,

            Instruction::UnaryOp { kind: datatype, .. } => *datatype,

            e => {
                println!("{:?}", e);
                unimplemented!()
            }
        }
    }

    #[inline(always)]
    pub fn has_return(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            stmts.iter().any(|stmt| stmt.is_return())
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn has_break(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            stmts.iter().any(|stmt| stmt.is_break())
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn has_continue(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            stmts.iter().any(|stmt| stmt.is_continue())
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self.get_data_type(),
            Type::U8 | Type::U16 | Type::U32 | Type::U64
        )
    }

    #[inline(always)]
    pub const fn is_gep(&self) -> bool {
        matches!(self, Instruction::GEP { .. })
    }

    #[inline(always)]
    pub const fn is_nullt(&self) -> bool {
        matches!(self, Instruction::NullT { .. })
    }

    #[inline(always)]
    pub const fn is_pre_unaryop(&self) -> bool {
        matches!(self, Instruction::UnaryOp { is_pre: true, .. })
    }

    #[inline(always)]
    pub const fn is_local_reference(&self) -> bool {
        matches!(self, Instruction::LocalRef { .. })
    }

    #[inline(always)]
    pub const fn is_function(&self) -> bool {
        matches!(self, Instruction::Function { .. })
    }

    #[inline(always)]
    pub const fn is_binary(&self) -> bool {
        matches!(self, Instruction::BinaryOp { .. })
    }

    #[inline(always)]
    pub const fn is_group(&self) -> bool {
        matches!(self, Instruction::Group { .. })
    }

    #[inline(always)]
    pub const fn is_return(&self) -> bool {
        matches!(self, Instruction::Return(_, _))
    }

    #[inline(always)]
    pub const fn is_break(&self) -> bool {
        matches!(self, Instruction::Break)
    }

    #[inline(always)]
    pub const fn is_continue(&self) -> bool {
        matches!(self, Instruction::Continue)
    }
}
