#![allow(clippy::upper_case_acronyms)]

use std::rc::Rc;

use inkwell::values::BasicValueEnum;

use crate::{common::error::ThrushCompilerError, frontend::lexer::Span};

use super::{
    statement::{BinaryOp, Constructor, FunctionPrototype, ThrushAttributes, UnaryOp},
    types::{TokenKind, Type},
};

#[derive(Debug, Clone, Default)]
pub enum Instruction<'ctx> {
    // Primitive types
    Str(Type, Vec<u8>, Span),
    Char(Type, u8, Span),
    Boolean(Type, bool, Span),
    Integer(Type, f64, bool, Span),
    Float(Type, f64, bool, Span),
    NullPtr {
        span: Span,
    },

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
        span: Span,
    },

    Property {
        name: &'ctx str,
        indexes: Vec<(Type, u32)>,
        kind: Type,
        is_mutable: bool,
        span: Span,
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
        cond: Rc<Instruction<'ctx>>,
        block: Rc<Instruction<'ctx>>,
        elfs: Vec<Instruction<'ctx>>,
        otherwise: Option<Rc<Instruction<'ctx>>>,
    },
    Elif {
        cond: Rc<Instruction<'ctx>>,
        block: Rc<Instruction<'ctx>>,
    },
    Else {
        block: Rc<Instruction<'ctx>>,
    },

    // Loops
    ForLoop {
        variable: Rc<Instruction<'ctx>>,
        cond: Rc<Instruction<'ctx>>,
        actions: Rc<Instruction<'ctx>>,
        block: Rc<Instruction<'ctx>>,
    },
    WhileLoop {
        cond: Rc<Instruction<'ctx>>,
        block: Rc<Instruction<'ctx>>,
    },
    Loop {
        block: Rc<Instruction<'ctx>>,
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
        body: Rc<Instruction<'ctx>>,
    },

    FunctionParameter {
        name: &'ctx str,
        kind: Type,
        position: u32,
        is_mutable: bool,
        span: Span,
    },
    Function {
        name: &'ctx str,
        params: Vec<Instruction<'ctx>>,
        param_types: Vec<Type>,
        body: Rc<Instruction<'ctx>>,
        return_type: Type,
        attributes: ThrushAttributes<'ctx>,
    },

    Return(Type, Rc<Instruction<'ctx>>),

    // Constants
    Const {
        name: &'ctx str,
        kind: Type,
        value: Rc<Instruction<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },
    ConstRef {
        name: &'ctx str,
        kind: Type,
        span: Span,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        kind: Type,
        value: Rc<Instruction<'ctx>>,
        is_mutable: bool,
        comptime: bool,
        span: Span,
    },
    LocalRef {
        name: &'ctx str,
        kind: Type,
        span: Span,
    },
    LocalMut {
        source: (&'ctx str, Option<Rc<Instruction<'ctx>>>),
        target: Rc<Instruction<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Pointer Manipulation
    Address {
        name: &'ctx str,
        indexes: Vec<Instruction<'ctx>>,
        kind: Type,
        span: Span,
    },

    Write {
        write_to: (&'ctx str, Option<Rc<Instruction<'ctx>>>),
        write_value: Rc<Instruction<'ctx>>,
        write_type: Type,
        span: Span,
    },

    Carry {
        name: &'ctx str,
        expression: Option<Rc<Instruction<'ctx>>>,
        carry_type: Type,
        span: Span,
    },

    // Expressions
    Call {
        name: &'ctx str,
        args: Vec<Instruction<'ctx>>,
        kind: Type,
        is_mutable: bool,
        span: Span,
    },
    BinaryOp {
        left: Rc<Instruction<'ctx>>,
        operator: TokenKind,
        right: Rc<Instruction<'ctx>>,
        kind: Type,
        span: Span,
    },
    UnaryOp {
        operator: TokenKind,
        kind: Type,
        expression: Rc<Instruction<'ctx>>,
        is_pre: bool,
        span: Span,
    },
    Group {
        expression: Rc<Instruction<'ctx>>,
        kind: Type,
        span: Span,
    },

    #[default]
    Null,
}

impl<'ctx> Instruction<'ctx> {
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

            Instruction::Str(kind, _, _) => kind,
            Instruction::Boolean(kind, _, _) => kind,
            Instruction::Char(kind, _, _) => kind,
            Instruction::Address { .. } => &Type::Address,
            Instruction::InitStruct { kind, .. } => kind,
            Instruction::Carry {
                carry_type: kind, ..
            } => kind,
            Instruction::Property { kind, .. } => kind,
            Instruction::NullPtr { .. } => &Type::Ptr(None),

            _ => &Type::Void,
        }
    }

    pub fn get_span(&self) -> Span {
        match self {
            Instruction::Integer(_, _, _, span) => *span,
            Instruction::Float(_, _, _, span) => *span,
            Instruction::Local { span, .. } => *span,
            Instruction::LocalMut { span, .. } => *span,
            Instruction::FunctionParameter { span, .. } => *span,
            Instruction::LocalRef { span, .. } => *span,
            Instruction::ConstRef { span, .. } => *span,
            Instruction::Call { span, .. } => *span,
            Instruction::BinaryOp { span, .. } => *span,
            Instruction::Group { span, .. } => *span,
            Instruction::UnaryOp { span, .. } => *span,

            Instruction::Str(_, _, span) => *span,
            Instruction::Boolean(_, _, span) => *span,
            Instruction::Char(_, _, span) => *span,
            Instruction::Address { span, .. } => *span,
            Instruction::InitStruct { span, .. } => *span,
            Instruction::Carry { span, .. } => *span,
            Instruction::Property { span, .. } => *span,
            Instruction::NullPtr { span } => *span,
            Instruction::Write { span, .. } => *span,
            Instruction::Const { span, .. } => *span,

            _ => unreachable!(),
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Instruction::Local { is_mutable, .. } => *is_mutable,
            Instruction::Property { is_mutable, .. } => *is_mutable,
            Instruction::Call { is_mutable, .. } => *is_mutable,
            _ => false,
        }
    }

    pub fn as_function(&self) -> FunctionPrototype {
        if let Instruction::Function {
            name,
            params,
            param_types,
            body,
            return_type,
            attributes,
        } = self
        {
            return (name, return_type, params, param_types, body, attributes);
        }

        unreachable!()
    }

    pub fn as_binary(&self) -> BinaryOp {
        if let Instruction::BinaryOp {
            left,
            operator,
            right,
            ..
        } = self
        {
            return (&**left, operator, &**right);
        }

        if let Instruction::Group { expression, .. } = self {
            return expression.as_binary();
        }

        unreachable!()
    }

    pub fn as_unaryop(&self) -> UnaryOp {
        if let Instruction::UnaryOp {
            operator,
            kind,
            expression,
            ..
        } = self
        {
            return (operator, kind, expression);
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
        if let Instruction::Integer(kind, _, is_signed, _) = self {
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

        if let Instruction::Float(_, _, is_signed, _) = self {
            if operator.is_minus_operator() {
                *is_signed = true;
            }
        }
    }

    pub fn throw_attemping_use_jit(&self, span: Span) -> Result<(), ThrushCompilerError> {
        if !self.is_integer() && !self.is_float() && !self.is_bool() {
            return Err(ThrushCompilerError::Error(
                String::from("Attemping use JIT"),
                String::from("This expression cannot be compiled correctly."),
                String::from(
                    "The compiler does not accept runtime-only expressions until the Just-in-Time (JIT) compiler development is complete.",
                ),
                span,
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
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self.get_type(),
            Type::U8 | Type::U16 | Type::U32 | Type::U64
        )
    }

    #[inline]
    pub const fn is_block(&self) -> bool {
        matches!(self, Instruction::Block { .. })
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
    pub const fn is_null(&self) -> bool {
        matches!(self, Instruction::Null)
    }

    #[inline]
    pub const fn is_integer(&self) -> bool {
        matches!(self, Instruction::Integer { .. })
    }

    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Instruction::Boolean(_, _, _))
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
