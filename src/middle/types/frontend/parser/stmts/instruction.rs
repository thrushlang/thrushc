#![allow(clippy::upper_case_acronyms)]

use std::rc::Rc;

use inkwell::values::BasicValueEnum;

use crate::{
    frontend::lexer::Span,
    middle::types::{
        backend::llvm::types::{LLVMBinaryOp, LLVMFunctionPrototype, LLVMUnaryOp},
        frontend::lexer::{tokenkind::TokenKind, types::ThrushType},
    },
    standard::error::ThrushCompilerIssue,
};

use super::types::{CompilerAttributes, Constructor};

#[derive(Debug, Clone, Default)]
pub enum Instruction<'ctx> {
    // Primitive types
    Str(ThrushType, Vec<u8>, Span),
    Char(ThrushType, u8, Span),
    Boolean(ThrushType, bool, Span),
    Integer(ThrushType, f64, bool, Span),
    Float(ThrushType, f64, bool, Span),
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
    Constructor {
        arguments: Constructor<'ctx>,
        kind: ThrushType,
        span: Span,
    },

    Bindings {
        name: String,
        binds: Vec<Instruction<'ctx>>,
    },

    Bind {
        name: &'ctx str,
        parameters: Vec<Instruction<'ctx>>,
        body: Rc<Instruction<'ctx>>,
        return_type: ThrushType,
        attributes: CompilerAttributes<'ctx>,
    },

    This {
        kind: ThrushType,
        is_mutable: bool,
        span: Span,
    },

    BindParameter {
        name: &'ctx str,
        kind: ThrushType,
        position: u32,
        is_mutable: bool,
        span: Span,
    },

    Property {
        name: &'ctx str,
        indexes: Vec<(ThrushType, u32)>,
        kind: ThrushType,
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
        span: Span,
    },
    Elif {
        cond: Rc<Instruction<'ctx>>,
        block: Rc<Instruction<'ctx>>,
        span: Span,
    },
    Else {
        block: Rc<Instruction<'ctx>>,
        span: Span,
    },

    // Loops
    For {
        local: Rc<Instruction<'ctx>>,
        cond: Rc<Instruction<'ctx>>,
        actions: Rc<Instruction<'ctx>>,
        block: Rc<Instruction<'ctx>>,
        span: Span,
    },
    While {
        cond: Rc<Instruction<'ctx>>,
        block: Rc<Instruction<'ctx>>,
        span: Span,
    },
    Loop {
        block: Rc<Instruction<'ctx>>,
        span: Span,
    },

    // Loop control flow
    Continue {
        span: Span,
    },
    Break {
        span: Span,
    },

    // Code block
    Block {
        stmts: Vec<Instruction<'ctx>>,
        span: Span,
    },

    // Functions

    // Entrypoint -> fn main() {}
    EntryPoint {
        body: Rc<Instruction<'ctx>>,
        span: Span,
    },

    FunctionParameter {
        name: &'ctx str,
        kind: ThrushType,
        position: u32,
        is_mutable: bool,
        span: Span,
    },
    Function {
        name: &'ctx str,
        parameters: Vec<Instruction<'ctx>>,
        parameter_types: Vec<ThrushType>,
        body: Rc<Instruction<'ctx>>,
        return_type: ThrushType,
        attributes: CompilerAttributes<'ctx>,
        span: Span,
    },

    Return {
        expression: Option<Rc<Instruction<'ctx>>>,
        kind: ThrushType,
        span: Span,
    },

    // Constants
    Const {
        name: &'ctx str,
        kind: ThrushType,
        value: Rc<Instruction<'ctx>>,
        attributes: CompilerAttributes<'ctx>,
        span: Span,
    },
    ConstRef {
        name: &'ctx str,
        kind: ThrushType,
        span: Span,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        kind: ThrushType,
        value: Rc<Instruction<'ctx>>,
        is_mutable: bool,
        comptime: bool,
        span: Span,
    },
    LocalRef {
        name: &'ctx str,
        kind: ThrushType,
        span: Span,
    },

    // Mutation
    Mut {
        source: (Option<&'ctx str>, Option<Rc<Instruction<'ctx>>>),
        value: Rc<Instruction<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    // Pointer Manipulation
    Address {
        name: &'ctx str,
        indexes: Vec<Instruction<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    Write {
        write_to: (&'ctx str, Option<Rc<Instruction<'ctx>>>),
        write_value: Rc<Instruction<'ctx>>,
        write_type: ThrushType,
        span: Span,
    },

    Carry {
        name: &'ctx str,
        expression: Option<Rc<Instruction<'ctx>>>,
        carry_type: ThrushType,
        span: Span,
    },

    // Expressions
    Call {
        name: &'ctx str,
        args: Vec<Instruction<'ctx>>,
        kind: ThrushType,
        span: Span,
    },
    BindCall {
        name: String,
        args: Vec<Instruction<'ctx>>,
        kind: ThrushType,
        span: Span,
    },
    BinaryOp {
        left: Rc<Instruction<'ctx>>,
        operator: TokenKind,
        right: Rc<Instruction<'ctx>>,
        kind: ThrushType,
        span: Span,
    },
    UnaryOp {
        operator: TokenKind,
        kind: ThrushType,
        expression: Rc<Instruction<'ctx>>,
        is_pre: bool,
        span: Span,
    },
    Group {
        expression: Rc<Instruction<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    Pass {
        span: Span,
    },

    #[default]
    Null,
}

impl<'ctx> Instruction<'ctx> {
    pub fn get_type(&self) -> Result<&ThrushType, ThrushCompilerIssue> {
        match self {
            Instruction::Integer(kind, ..) => Ok(kind),
            Instruction::Float(kind, ..) => Ok(kind),
            Instruction::Local { kind, .. } => Ok(kind),
            Instruction::Mut { kind, .. } => Ok(kind),
            Instruction::FunctionParameter { kind, .. } => Ok(kind),
            Instruction::LocalRef { kind, .. } => Ok(kind),
            Instruction::ConstRef { kind, .. } => Ok(kind),
            Instruction::Call { kind, .. } => Ok(kind),
            Instruction::BinaryOp { kind, .. } => Ok(kind),
            Instruction::Group { kind, .. } => Ok(kind),
            Instruction::UnaryOp { kind, .. } => Ok(kind),

            Instruction::Str(kind, _, _) => Ok(kind),
            Instruction::Boolean(kind, _, _) => Ok(kind),
            Instruction::Char(kind, _, _) => Ok(kind),
            Instruction::Address { .. } => Ok(&ThrushType::Address),
            Instruction::Constructor { kind, .. } => Ok(kind),
            Instruction::Carry {
                carry_type: kind, ..
            } => Ok(kind),
            Instruction::Property { kind, .. } => Ok(kind),
            Instruction::NullPtr { .. } => Ok(&ThrushType::Ptr(None)),
            Instruction::This { kind, .. } => Ok(kind),
            Instruction::BindCall { kind, .. } => Ok(kind),
            Instruction::BindParameter { kind, .. } => Ok(kind),
            Instruction::Return { kind, .. } => Ok(kind),

            _ => Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected a value to get a type."),
                None,
                self.get_span()?,
            )),
        }
    }

    pub fn get_type_unwrapped(&self) -> &ThrushType {
        match self {
            Instruction::Integer(kind, ..) => kind,
            Instruction::Float(kind, ..) => kind,
            Instruction::Local { kind, .. } => kind,
            Instruction::Mut { kind, .. } => kind,
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
            Instruction::Address { .. } => &ThrushType::Address,
            Instruction::Constructor { kind, .. } => kind,
            Instruction::Carry {
                carry_type: kind, ..
            } => kind,
            Instruction::Property { kind, .. } => kind,
            Instruction::NullPtr { .. } => &ThrushType::Ptr(None),
            Instruction::This { kind, .. } => kind,
            Instruction::BindCall { kind, .. } => kind,
            Instruction::BindParameter { kind, .. } => kind,
            Instruction::Return { kind, .. } => kind,

            any => {
                panic!("Attempting to unwrap a null type: {:?}.", any)
            }
        }
    }

    pub fn get_span(&self) -> Result<Span, ThrushCompilerIssue> {
        match self {
            Instruction::Integer(_, _, _, span) => Ok(*span),
            Instruction::Float(_, _, _, span) => Ok(*span),
            Instruction::Local { span, .. } => Ok(*span),
            Instruction::Mut { span, .. } => Ok(*span),
            Instruction::FunctionParameter { span, .. } => Ok(*span),
            Instruction::LocalRef { span, .. } => Ok(*span),
            Instruction::ConstRef { span, .. } => Ok(*span),
            Instruction::Call { span, .. } => Ok(*span),
            Instruction::BinaryOp { span, .. } => Ok(*span),
            Instruction::Group { span, .. } => Ok(*span),
            Instruction::UnaryOp { span, .. } => Ok(*span),

            Instruction::Str(_, _, span) => Ok(*span),
            Instruction::Boolean(_, _, span) => Ok(*span),
            Instruction::Char(_, _, span) => Ok(*span),
            Instruction::Address { span, .. } => Ok(*span),
            Instruction::Constructor { span, .. } => Ok(*span),
            Instruction::Carry { span, .. } => Ok(*span),
            Instruction::Property { span, .. } => Ok(*span),
            Instruction::NullPtr { span } => Ok(*span),
            Instruction::Write { span, .. } => Ok(*span),
            Instruction::Const { span, .. } => Ok(*span),
            Instruction::This { span, .. } => Ok(*span),
            Instruction::BindCall { span, .. } => Ok(*span),
            Instruction::BindParameter { span, .. } => Ok(*span),
            Instruction::Return { span, .. } => Ok(*span),

            Instruction::Else { span, .. } => Ok(*span),
            Instruction::Elif { span, .. } => Ok(*span),
            Instruction::If { span, .. } => Ok(*span),

            Instruction::Continue { span, .. } => Ok(*span),
            Instruction::Break { span, .. } => Ok(*span),
            Instruction::While { span, .. } => Ok(*span),
            Instruction::For { span, .. } => Ok(*span),
            Instruction::Pass { span } => Ok(*span),

            _ => Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected a valid statament to get a valid code point."),
                None,
                Span::default(),
            )),
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Instruction::Local { is_mutable, .. } => *is_mutable,
            Instruction::Property { is_mutable, .. } => *is_mutable,
            _ => false,
        }
    }

    pub fn as_llvm_function_proto(&self) -> LLVMFunctionPrototype {
        if let Instruction::Function {
            name,
            parameters,
            parameter_types,
            body,
            return_type,
            attributes,
            ..
        } = self
        {
            return (
                name,
                return_type,
                parameters,
                parameter_types,
                body,
                attributes,
            );
        }

        unreachable!()
    }

    pub fn as_binary(&self) -> LLVMBinaryOp {
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

    pub fn as_unaryop(&self) -> LLVMUnaryOp {
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

    pub fn get_binding_name(&self) -> &'ctx str {
        if let Instruction::Bind { name, .. } = self {
            return name;
        }

        unreachable!()
    }

    pub fn get_binding_parameters(&self) -> Vec<ThrushType> {
        if let Instruction::Bind { parameters, .. } = self {
            let parameters_type: Vec<ThrushType> = parameters
                .iter()
                .map(|parameter| parameter.get_type_unwrapped().clone())
                .collect();

            return parameters_type;
        }

        unreachable!()
    }

    pub fn get_binding_type(&self) -> ThrushType {
        if let Instruction::Bind { return_type, .. } = self {
            return return_type.clone();
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

    pub fn throw_attemping_use_jit(&self, span: Span) -> Result<(), ThrushCompilerIssue> {
        if !self.is_integer() && !self.is_float() && !self.is_bool() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Attemping use JIT"),
                String::from("This expression cannot be compiled correctly."),
                Some(String::from(
                    "The compiler does not accept runtime-only expressions until the Just-in-Time (JIT) compiler development is complete.",
                )),
                span,
            ));
        }

        Ok(())
    }
}

impl Instruction<'_> {
    pub fn has_instruction(&self) -> bool {
        if let Instruction::Block { stmts, .. } = self {
            return !stmts.is_empty();
        }

        false
    }

    pub fn has_return(&self) -> bool {
        if let Instruction::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_return());
        }

        false
    }

    pub fn has_break(&self) -> bool {
        if let Instruction::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_break());
        }

        false
    }

    pub fn has_continue(&self) -> bool {
        if let Instruction::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_continue());
        }

        false
    }

    #[inline]
    pub fn is_block(&self) -> bool {
        matches!(self, Instruction::Block { .. })
    }

    #[inline]
    pub fn is_unsigned_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_type()?,
            ThrushType::U8 | ThrushType::U16 | ThrushType::U32 | ThrushType::U64
        ))
    }

    #[inline]
    pub fn is_anyu32bit_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_type()?,
            ThrushType::U8 | ThrushType::U16 | ThrushType::U32
        ))
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
    pub const fn is_constant(&self) -> bool {
        matches!(self, Instruction::Const { .. })
    }

    #[inline]
    pub const fn is_constructor(&self) -> bool {
        matches!(self, Instruction::Constructor { .. })
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
        matches!(self, Instruction::Return { .. })
    }

    #[inline]
    pub const fn is_break(&self) -> bool {
        matches!(self, Instruction::Break { .. })
    }

    #[inline]
    pub const fn is_continue(&self) -> bool {
        matches!(self, Instruction::Continue { .. })
    }
}
