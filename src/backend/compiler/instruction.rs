#![allow(clippy::upper_case_acronyms)]

use super::{
    super::super::{
        common::error::ThrushCompilerError,
        frontend::{
            lexer::{TokenKind, Type},
            types::{CodeLocation, Constructor},
        },
    },
    types::FunctionPrototype,
};

use super::types::{BinaryOp, ThrushAttributes, UnaryOp};

use inkwell::values::BasicValueEnum;

#[derive(Debug, Clone, Default)]
pub enum Instruction<'ctx> {
    // Primitive types
    Str(Type, Vec<u8>),
    Char(Type, u8),
    Boolean(Type, bool),
    Integer(Type, f64, bool),
    Float(Type, f64, bool),

    // LLVMValue
    LLVMValue(BasicValueEnum<'ctx>),

    // Structures

    /*

        // EXAMPLE:

        struct Vector {
            data T;
            size u64;
            capacity u64;
        };

    */
    // new Vec { ... };
    InitStruct {
        arguments: Constructor<'ctx>,
        kind: Type,
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
        position: u32,
        line: usize,
        span: (usize, usize),
    },
    Function {
        name: &'ctx str,
        params: Vec<Instruction<'ctx>>,
        body: Option<Box<Instruction<'ctx>>>,
        return_type: Type,
        attributes: ThrushAttributes<'ctx>,
    },

    Return(Type, Box<Instruction<'ctx>>),

    // Constants
    Const {
        name: &'ctx str,
        kind: Type,
        value: Box<Instruction<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
    },
    ConstRef {
        name: &'ctx str,
        kind: Type,
        take: bool,
        line: usize,
    },

    // LOW-LEVEL instructions
    Instr {
        name: &'ctx str,
        kind: Type,
        value: Box<Instruction<'ctx>>,
        line: usize,
    },

    InstrRef {
        name: &'ctx str,
        kind: Type,
        line: usize,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        kind: Type,
        value: Box<Instruction<'ctx>>,
        comptime: bool,
        line: usize,
    },
    LocalRef {
        name: &'ctx str,
        kind: Type,
        take: bool,
        line: usize,
    },
    LocalMut {
        name: &'ctx str,
        kind: Type,
        value: Box<Instruction<'ctx>>,
    },

    // Pointer Manipulation
    Address {
        name: &'ctx str,
        indexes: Vec<Instruction<'ctx>>,
        kind: Type,
    },

    Carry {
        name: &'ctx str,
        expression: Option<Box<Instruction<'ctx>>>,
        kind: Type,
    },

    // Expressions
    Call {
        name: &'ctx str,
        args: Vec<Instruction<'ctx>>,
        kind: Type,
    },
    BinaryOp {
        left: Box<Instruction<'ctx>>,
        op: &'ctx TokenKind,
        right: Box<Instruction<'ctx>>,
        kind: Type,
    },
    UnaryOp {
        op: &'ctx TokenKind,
        kind: Type,
        expression: Box<Instruction<'ctx>>,
        is_pre: bool,
    },
    Group {
        expression: Box<Instruction<'ctx>>,
        kind: Type,
    },

    #[default]
    Null,
}

impl<'ctx> Instruction<'ctx> {
    #[inline]
    pub fn get_type(&self) -> &Type {
        match self {
            Instruction::Integer(kind, ..) => kind,
            Instruction::Float(kind, ..) => kind,
            Instruction::Local { kind, .. } => kind,
            Instruction::LocalMut { kind, .. } => kind,
            Instruction::FunctionParameter { kind, .. } => kind,
            Instruction::LocalRef { kind, .. } => kind,
            Instruction::ConstRef { kind, .. } => kind,
            Instruction::Call { kind, .. } => kind,
            Instruction::BinaryOp { kind, .. } => kind,
            Instruction::Group { kind, .. } => kind,
            Instruction::UnaryOp { kind, .. } => kind,

            Instruction::Str(kind, _) => kind,
            Instruction::Boolean(kind, _) => kind,
            Instruction::Char(kind, _) => kind,
            Instruction::Address { .. } => &Type::Address,
            Instruction::InitStruct { kind, .. } => kind,
            Instruction::Carry { kind, .. } => kind,

            _ => &Type::Void,
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
            kind,
            expression,
            ..
        } = self
        {
            return (op, kind, expression);
        }

        unreachable!()
    }

    pub fn as_llvm_value(&self) -> &BasicValueEnum<'ctx> {
        if let Instruction::LLVMValue(llvm_value) = self {
            return llvm_value;
        }

        unreachable!()
    }
}

impl Instruction<'_> {
    pub fn cast_signess(&mut self, operator: TokenKind) {
        if let Instruction::Integer(kind, _, is_signed) = self {
            if operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
                *is_signed = true;
            }
        }

        if let Instruction::LocalRef { kind, .. } | Instruction::ConstRef { kind, .. } = self {
            if kind.is_integer_type() && operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
            }
        }

        if let Instruction::Float(_, _, is_signed) = self {
            if operator.is_minus_operator() {
                *is_signed = true;
            }
        }
    }

    pub fn throw_attemping_use_jit(
        &self,
        location: CodeLocation,
    ) -> Result<(), ThrushCompilerError> {
        if !self.is_integer() && !self.is_float() && !self.is_bool() {
            return Err(ThrushCompilerError::Error(
                String::from("Attemping use JIT"),
                String::from(
                    "The compiler does not accept runtime-only expressions until the Just-in-Time (JIT) compiler development is complete.",
                ),
                location.0,
                Some(location.1),
            ));
        }

        Ok(())
    }
}

impl Instruction<'_> {
    #[inline]
    pub fn has_instruction(&self) -> bool {
        if let Instruction::Block { stmts } = self {
            return !stmts.is_empty();
        }

        false
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

    #[inline]
    pub fn is_low_level_instructions(&self) -> bool {
        matches!(
            self,
            Instruction::Carry { .. } | Instruction::Address { .. }
        )
    }

    #[inline]
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self.get_type(),
            Type::U8 | Type::U16 | Type::U32 | Type::U64
        )
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, Instruction::Null { .. })
    }

    #[inline]
    pub const fn is_gep(&self) -> bool {
        matches!(self, Instruction::Address { .. })
    }

    #[inline]
    pub const fn is_carry(&self) -> bool {
        matches!(self, Instruction::Carry { .. })
    }

    #[inline]
    pub const fn is_local_ref(&self) -> bool {
        matches!(self, Instruction::LocalRef { .. })
    }

    #[inline]
    pub const fn is_pre_unaryop(&self) -> bool {
        matches!(self, Instruction::UnaryOp { is_pre: true, .. })
    }

    #[inline]
    pub const fn is_function(&self) -> bool {
        matches!(self, Instruction::Function { .. })
    }

    #[inline]
    pub const fn is_binary(&self) -> bool {
        matches!(self, Instruction::BinaryOp { .. })
    }

    #[inline]
    pub const fn is_group(&self) -> bool {
        matches!(self, Instruction::Group { .. })
    }

    #[inline]
    pub const fn is_str(&self) -> bool {
        matches!(self, Instruction::Str { .. })
    }

    #[inline]
    pub const fn is_integer(&self) -> bool {
        matches!(self, Instruction::Integer { .. })
    }

    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Instruction::Boolean(_, _))
    }

    #[inline]
    pub const fn is_float(&self) -> bool {
        matches!(self, Instruction::Float { .. })
    }

    #[inline]
    pub const fn is_return(&self) -> bool {
        matches!(self, Instruction::Return(_, _))
    }

    #[inline]
    pub const fn is_break(&self) -> bool {
        matches!(self, Instruction::Break)
    }

    #[inline]
    pub const fn is_continue(&self) -> bool {
        matches!(self, Instruction::Continue)
    }
}
