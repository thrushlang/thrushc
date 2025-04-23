#![allow(clippy::upper_case_acronyms)]

use std::mem;

use super::{
    super::super::{
        common::error::ThrushCompilerError,
        frontend::{
            lexer::{TokenKind, Type},
            types::Constructor,
        },
    },
    memory::MemoryFlag,
    types::FunctionPrototype,
};

use super::{
    objects::CompilerObjects,
    types::{BinaryOp, Structure, StructureFields, ThrushAttributes, UnaryOp},
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
    LLVMValue(BasicValueEnum<'ctx>),

    // Types

    // T<?> array<[T, N]> Vec<T, 'heap>
    ComplexType(
        Type,                           // Type
        &'ctx str,                      // Structure Name
        Option<MemoryFlag>,             // Customization heap!, type
        Option<Box<Instruction<'ctx>>>, // Parent recusive type T<A<B<C>>>
    ),

    // Structures

    /*

        // EXAMPLE:

        struct Vector {
            data T;
            size u64;
            capacity u64;
        };

    */
    Struct {
        name: &'ctx str,
        fields_types: StructureFields<'ctx>,
    },

    // new Vec { ... };
    InitStruct {
        name: &'ctx str,
        arguments: Constructor<'ctx>,
    },

    // Enums

    /*

        // EXAMPLE:

        enum Colors {
            Red : s32 = x0FF0000;
            Green : s32 = x00FF00;
            Blue : s64 = x0000FF;
        };

    */
    EnumField {
        kind: Box<Instruction<'ctx>>,
        value: Box<Instruction<'ctx>>,
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
        attributes: ThrushAttributes<'ctx>,
    },

    Return(Box<Instruction<'ctx>>, Box<Instruction<'ctx>>),

    // Constants
    Const {
        name: &'ctx str,
        kind: Box<Instruction<'ctx>>,
        value: Box<Instruction<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
    },

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
        kind: Box<Instruction<'ctx>>,
        take: bool,
        line: usize,
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
        kind: Box<Instruction<'ctx>>,
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

    Comptime,

    #[default]
    Null,
}

impl<'ctx> Instruction<'ctx> {
    pub fn build_struct_type(
        &self,
        context: &'ctx Context,
        struct_fields: Option<&StructureFields>,
        compiler_objects: &CompilerObjects,
    ) -> StructType<'ctx> {
        if let Some(from_fields) = struct_fields {
            return utils::build_struct_type_from_fields(context, from_fields);
        }

        if let Instruction::InitStruct { name, .. } = self {
            let structure: &Structure = compiler_objects.get_struct(name);
            let fields: &StructureFields = &structure.1;

            return utils::build_struct_type_from_fields(context, fields);
        }

        if let Instruction::LocalRef { kind, .. } = self {
            let structure_type: &str = kind.get_structure_type();
            let structure: &Structure = compiler_objects.get_struct(structure_type);
            let fields: &StructureFields = &structure.1;

            return utils::build_struct_type_from_fields(context, fields);
        }

        if let Instruction::Call { kind, .. } = self {
            let structure_type: &str = kind.get_structure_type();
            let structure: &Structure = compiler_objects.get_struct(structure_type);
            let fields: &StructureFields = &structure.1;

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
    ) -> Result<(), ThrushCompilerError> {
        if let Instruction::ComplexType(_, _, _, _) = self {
            return Ok(());
        }

        Err(ThrushCompilerError::Error(
            String::from("Undeterminated type"),
            String::from("Expected type."),
            line,
            Some(span),
        ))
    }

    #[inline(always)]
    pub fn get_basic_type(&self) -> &Type {
        match self {
            Instruction::ComplexType(datatype, _, _, _) => datatype,

            Instruction::Integer(datatype, ..)
            | Instruction::Float(datatype, ..)
            | Instruction::LocalRef { kind: datatype, .. }
            | Instruction::LocalMut { kind: datatype, .. }
            | Instruction::Local { kind: datatype, .. }
            | Instruction::Call { kind: datatype, .. }
            | Instruction::BinaryOp { kind: datatype, .. }
            | Instruction::Group { kind: datatype, .. }
            | Instruction::UnaryOp { kind: datatype, .. }
            | Instruction::FunctionParameter { kind: datatype, .. }
            | Instruction::EnumField { kind: datatype, .. } => datatype.get_basic_type(),

            Instruction::Str(_) => &Type::Str,
            Instruction::Boolean(_) => &Type::Bool,
            Instruction::Char(_) => &Type::Char,
            Instruction::GEP { .. } => &Type::Ptr,
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
            | Instruction::FunctionParameter { kind: datatype, .. }
            | Instruction::GEP { kind: datatype, .. }
            | Instruction::EnumField { kind: datatype, .. } => (**datatype).clone(),

            Instruction::Str(_) => Instruction::ComplexType(Type::Str, "", None, None),
            Instruction::Boolean(_) => Instruction::ComplexType(Type::Bool, "", None, None),
            Instruction::Char(_) => Instruction::ComplexType(Type::Char, "", None, None),

            Instruction::InitStruct { name, .. } => {
                Instruction::ComplexType(Type::Struct, name, None, None)
            }

            Instruction::Struct { name, .. } => {
                Instruction::ComplexType(Type::Struct, name, None, None)
            }

            instruction if instruction.is_complex_type() => instruction.clone(),

            e => {
                println!("{:?}", e);
                unimplemented!()
            }
        }
    }

    pub fn as_function(&self) -> FunctionPrototype {
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

    pub fn get_enum_field_value(&mut self) -> Instruction<'ctx> {
        if let Instruction::EnumField { value, .. } = self {
            return mem::take(value);
        }

        unreachable!()
    }

    pub fn get_structure_type(&self) -> &'ctx str {
        if let Instruction::ComplexType(_, structure_type, _, _) = self {
            return structure_type;
        }

        unreachable!()
    }

    pub fn cast_signess(&mut self, operator: TokenKind) {
        if let Instruction::Integer(kind, _, is_signed) = self {
            if operator.is_minus_operator() {
                *kind = Box::new(kind.narrowing_cast());
                *is_signed = true;
            }
        }

        if let Instruction::LocalRef { kind, .. } = self {
            if kind.is_integer_type() && operator.is_minus_operator() {
                *kind = Box::new(kind.narrowing_cast());
            }
        }

        if let Instruction::Float(_, _, is_signed) = self {
            if operator.is_minus_operator() {
                *is_signed = true;
            }
        }
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

        Instruction::ComplexType(narrowed_type, instruction_structure_type, None, None)
    }
}

impl Instruction<'_> {
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
        if let Instruction::ComplexType(tp, _, _, _) = self {
            return tp.is_integer_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_float_type(&self) -> bool {
        if let Instruction::ComplexType(tp, _, _, _) = self {
            return tp.is_float_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_ptr_type(&self) -> bool {
        if let Instruction::ComplexType(tp, _, _, _) = self {
            return tp.is_ptr_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_void_type(&self) -> bool {
        if let Instruction::ComplexType(tp, _, _, _) = self {
            return tp.is_void_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_struct_type(&self) -> bool {
        if let Instruction::ComplexType(tp, _, _, _) = self {
            return tp.is_struct_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_bool_type(&self) -> bool {
        if let Instruction::ComplexType(tp, _, _, _) = self {
            return tp.is_bool_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_str_type(&self) -> bool {
        if let Instruction::ComplexType(tp, _, _, _) = self {
            return tp.is_str_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_raw_ptr_type(&self) -> bool {
        if let Instruction::ComplexType(tp, _, _, _) = self {
            return tp.is_raw_ptr_type();
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
        matches!(self, Instruction::ComplexType { .. })
    }

    #[inline(always)]
    pub const fn is_gep(&self) -> bool {
        matches!(self, Instruction::GEP { .. })
    }

    #[inline(always)]
    pub const fn is_local_ref(&self) -> bool {
        matches!(self, Instruction::LocalRef { .. })
    }

    #[inline(always)]
    pub const fn is_null(&self) -> bool {
        matches!(self, Instruction::ComplexType(Type::Void, _, _, _))
    }

    #[inline(always)]
    pub const fn is_pre_unaryop(&self) -> bool {
        matches!(self, Instruction::UnaryOp { is_pre: true, .. })
    }

    #[inline(always)]
    pub const fn is_function(&self) -> bool {
        matches!(self, Instruction::Function { .. })
    }

    #[inline(always)]
    pub const fn is_enum_field(&self) -> bool {
        matches!(self, Instruction::EnumField { .. })
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
    pub const fn is_integer(&self) -> bool {
        matches!(self, Instruction::Integer { .. })
    }

    #[inline(always)]
    pub const fn is_float(&self) -> bool {
        matches!(self, Instruction::Float { .. })
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
