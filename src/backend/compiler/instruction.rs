#![allow(clippy::upper_case_acronyms)]

use super::{
    super::super::{
        common::error::ThrushCompilerError,
        frontend::{
            lexer::{TokenKind, Type},
            types::StructFields,
        },
    },
    types::CompilerType,
};

use super::{
    objects::CompilerObjects,
    types::{
        BinaryOp, CompilerAttributes, CompilerStructure, CompilerStructureFields, Function, UnaryOp,
    },
    utils,
};

use inkwell::{context::Context, types::StructType, values::BasicValueEnum};

#[derive(Debug, Clone, Default)]
pub enum Instruction<'ctx> {
    // Primitive types
    Str(Vec<u8>),
    Char(u8),
    Boolean(bool),
    Integer(Box<Instruction<'ctx>>, f64, bool),
    Float(Box<Instruction<'ctx>>, f64, bool),
    Struct {
        name: &'ctx str,
        fields_types: CompilerStructureFields<'ctx>,
    },
    NullT,

    LLVMValue(BasicValueEnum<'ctx>),

    Type(Type, &'ctx str),

    InitStruct {
        name: &'ctx str,
        fields: StructFields<'ctx>,
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
        kind: Box<Instruction<'ctx>>,
        position: u32,
        line: usize,
        span: (usize, usize),
    },
    Function {
        name: &'ctx str,
        params: Vec<Instruction<'ctx>>,
        body: Option<Box<Instruction<'ctx>>>,
        return_type: Box<Instruction<'ctx>>,
        attributes: CompilerAttributes<'ctx>,
    },

    Return(Box<Instruction<'ctx>>, Box<Instruction<'ctx>>),

    // Locals variables
    Local {
        name: &'ctx str,
        kind: Box<Instruction<'ctx>>,
        value: Box<Instruction<'ctx>>,
        comptime: bool,
        line: usize,
    },
    LocalRef {
        name: &'ctx str,
        line: usize,
        kind: Box<Instruction<'ctx>>,
    },
    LocalMut {
        name: &'ctx str,
        kind: Box<Instruction<'ctx>>,
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
        kind: Box<Instruction<'ctx>>,
    },
    BinaryOp {
        left: Box<Instruction<'ctx>>,
        op: &'ctx TokenKind,
        right: Box<Instruction<'ctx>>,
        kind: Box<Instruction<'ctx>>,
    },
    UnaryOp {
        op: &'ctx TokenKind,
        expression: Box<Instruction<'ctx>>,
        kind: Box<Instruction<'ctx>>,
        is_pre: bool,
    },
    Group {
        expression: Box<Instruction<'ctx>>,
        kind: Box<Instruction<'ctx>>,
    },

    #[default]
    Null,
}

impl<'ctx> Instruction<'ctx> {
    pub fn build_struct_type(
        &self,
        context: &'ctx Context,
        struct_fields: Option<&CompilerStructureFields>,
        compiler_objects: &CompilerObjects,
    ) -> StructType<'ctx> {
        if let Some(from_fields) = struct_fields {
            return utils::build_struct_type_from_fields(context, from_fields);
        }

        if let Instruction::InitStruct { fields, .. } = self {
            let mut new_fields: Vec<(&'ctx str, &'ctx str, Type, u32)> =
                Vec::with_capacity(fields.len());

            fields.iter().for_each(|field| {
                new_fields.push((field.0, "", field.2, field.3));
            });

            return utils::build_struct_type_from_fields(context, &new_fields);
        }

        if let Instruction::LocalRef { kind, .. } = self {
            let structure_type: &str = kind.get_structure_type();
            let structure: &CompilerStructure = compiler_objects.get_struct(structure_type);
            let fields: &CompilerStructureFields = &structure.1;

            return utils::build_struct_type_from_fields(context, fields);
        }

        if let Instruction::Call { kind, .. } = self {
            let structure_type: &str = kind.get_structure_type();
            let structure: &CompilerStructure = compiler_objects.get_struct(structure_type);
            let fields: &CompilerStructureFields = &structure.1;

            return utils::build_struct_type_from_fields(context, fields);
        }

        unreachable!()
    }

    #[inline]
    pub fn has_instruction(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            return !stmts.is_empty();
        }

        false
    }

    #[inline]
    pub fn expected_type(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<CompilerType, ThrushCompilerError> {
        if let Instruction::Type(tp, structure_type) = self {
            return Ok((tp, structure_type));
        }

        Err(ThrushCompilerError::Error(
            String::from("Undeterminated type"),
            String::from("Expected type."),
            line,
            Some(span),
        ))
    }

    #[inline]
    pub fn is_chained(
        &self,
        other: &Instruction,
        location: (usize, (usize, usize)),
    ) -> Result<(), ThrushCompilerError> {
        if matches!(
            (self, other),
            (Instruction::BinaryOp { .. }, Instruction::Group { .. })
                | (Instruction::BinaryOp { .. }, Instruction::BinaryOp { .. })
                | (Instruction::Group { .. }, Instruction::BinaryOp { .. })
                | (Instruction::Group { .. }, Instruction::Group { .. })
                | (
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
                )
        ) {
            return Ok(());
        }

        Err(ThrushCompilerError::Error(
            String::from("Type Checking"),
            String::from("Operators cannot be chained. Use logical gates as '&&' or '||'."),
            location.0,
            Some(location.1),
        ))
    }

    #[inline(always)]
    pub fn get_basic_type(&self) -> &Type {
        match self {
            Instruction::Type(datatype, _) => datatype,

            Instruction::Integer(datatype, ..)
            | Instruction::Float(datatype, ..)
            | Instruction::LocalRef { kind: datatype, .. }
            | Instruction::LocalMut { kind: datatype, .. }
            | Instruction::Local { kind: datatype, .. }
            | Instruction::Call { kind: datatype, .. }
            | Instruction::BinaryOp { kind: datatype, .. }
            | Instruction::Group { kind: datatype, .. }
            | Instruction::UnaryOp { kind: datatype, .. }
            | Instruction::FunctionParameter { kind: datatype, .. } => datatype.get_basic_type(),

            Instruction::Str(_) => &Type::Str,
            Instruction::Boolean(_) => &Type::Bool,
            Instruction::Char(_) => &Type::Char,
            Instruction::NullT => &Type::T,
            Instruction::GEP { .. } => &Type::T,
            Instruction::InitStruct { .. } => &Type::Struct,
            Instruction::Struct { .. } => &Type::Struct,

            e => {
                println!("{:?}", e);
                unimplemented!()
            }
        }
    }

    #[must_use]
    #[inline]
    pub fn get_type(&self) -> Instruction<'ctx> {
        match self {
            Instruction::Integer(datatype, ..)
            | Instruction::Float(datatype, ..)
            | Instruction::LocalRef { kind: datatype, .. }
            | Instruction::LocalMut { kind: datatype, .. }
            | Instruction::Local { kind: datatype, .. }
            | Instruction::Call { kind: datatype, .. }
            | Instruction::BinaryOp { kind: datatype, .. }
            | Instruction::Group { kind: datatype, .. }
            | Instruction::UnaryOp { kind: datatype, .. }
            | Instruction::FunctionParameter { kind: datatype, .. } => (**datatype).clone(),

            Instruction::Str(_) => Instruction::Type(Type::Str, ""),
            Instruction::Boolean(_) => Instruction::Type(Type::Bool, ""),
            Instruction::Char(_) => Instruction::Type(Type::Char, ""),
            Instruction::NullT => Instruction::Type(Type::T, ""),
            Instruction::GEP { .. } => Instruction::Type(Type::T, ""),
            Instruction::InitStruct { name, .. } => Instruction::Type(Type::Struct, name),
            Instruction::Struct { name, .. } => Instruction::Type(Type::Struct, name),

            instruction if instruction.is_complex_type() => instruction.clone(),

            e => {
                println!("{:?}", e);
                unimplemented!()
            }
        }
    }

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

    pub fn as_binary(&self) -> BinaryOp {
        if let Instruction::BinaryOp {
            left, op, right, ..
        } = self
        {
            return (&**left, op, &**right);
        }

        if let Instruction::Group { expression, .. } = self {
            return expression.as_binary();
        }

        unreachable!()
    }

    pub fn as_unaryop(&self) -> UnaryOp {
        if let Instruction::UnaryOp {
            op,
            expression,
            kind,
            ..
        } = self
        {
            return (op, expression, kind);
        }

        unreachable!()
    }

    pub fn as_llvm_value(&self) -> &BasicValueEnum<'ctx> {
        if let Instruction::LLVMValue(llvm_value) = self {
            return llvm_value;
        }

        unreachable!()
    }

    pub fn get_structure_type(&self) -> &'ctx str {
        if let Instruction::Type(_, structure_type) = self {
            return structure_type;
        }

        unreachable!()
    }

    pub fn narrowing_cast(&self) -> Instruction<'ctx> {
        let instruction_type: &Type = self.get_basic_type();
        let instruction_structure_type: &str = self.get_structure_type();

        let narrowed_type: Type = match instruction_type {
            Type::U8 => Type::S8,
            Type::U16 => Type::S16,
            Type::U32 => Type::S32,
            Type::U64 => Type::S64,
            _ => *instruction_type,
        };

        Instruction::Type(narrowed_type, instruction_structure_type)
    }

    pub fn has_return(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            return stmts.iter().any(|stmt| stmt.is_return());
        }

        false
    }

    pub fn has_break(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            return stmts.iter().any(|stmt| stmt.is_break());
        }

        false
    }

    pub fn has_continue(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            return stmts.iter().any(|stmt| stmt.is_continue());
        }

        false
    }

    #[inline(always)]
    pub fn is_integer_type(&self) -> bool {
        if let Instruction::Type(tp, _) = self {
            return tp.is_integer_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self.get_basic_type(),
            Type::U8 | Type::U16 | Type::U32 | Type::U64
        )
    }

    #[inline(always)]
    pub const fn is_complex_type(&self) -> bool {
        matches!(self, Instruction::Type { .. })
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
