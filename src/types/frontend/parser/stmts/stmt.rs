#![allow(clippy::upper_case_acronyms)]

use std::rc::Rc;

use inkwell::values::BasicValueEnum;

use crate::{
    frontend::lexer::span::Span,
    standard::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    types::{
        backend::llvm::types::{LLVMBinaryOp, LLVMFunctionPrototype, LLVMUnaryOp},
        frontend::lexer::{tokenkind::TokenKind, types::ThrushType},
    },
};

use super::{
    ident::ReferenceIndentificator,
    sites::LLIAllocationSite,
    types::{CompilerAttributes, Constructor, EnumFields, StructFields},
};

#[derive(Debug, Clone)]
pub enum ThrushStatement<'ctx> {
    Str {
        kind: ThrushType,
        bytes: Vec<u8>,
        span: Span,
    },

    Char {
        kind: ThrushType,
        byte: u64,
        span: Span,
    },

    Boolean {
        kind: ThrushType,
        value: u64,
        span: Span,
    },

    Integer {
        kind: ThrushType,
        value: u64,
        signed: bool,
        span: Span,
    },

    Float {
        kind: ThrushType,
        value: f64,
        signed: bool,
        span: Span,
    },

    NullPtr {
        span: Span,
    },

    LLVMValue(BasicValueEnum<'ctx>, Span),

    // Structures
    Struct {
        name: &'ctx str,
        fields: StructFields<'ctx>,
        kind: ThrushType,
        span: Span,
    },

    Constructor {
        name: &'ctx str,
        arguments: Constructor<'ctx>,
        kind: ThrushType,
        span: Span,
    },

    Methods {
        name: String,
        binds: Vec<ThrushStatement<'ctx>>,
        span: Span,
    },

    Method {
        name: &'ctx str,
        parameters: Vec<ThrushStatement<'ctx>>,
        parameters_types: Vec<ThrushType>,
        body: Rc<ThrushStatement<'ctx>>,
        return_type: ThrushType,
        attributes: CompilerAttributes<'ctx>,
        span: Span,
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
        cond: Rc<ThrushStatement<'ctx>>,
        block: Rc<ThrushStatement<'ctx>>,
        elfs: Vec<ThrushStatement<'ctx>>,
        otherwise: Option<Rc<ThrushStatement<'ctx>>>,
        span: Span,
    },
    Elif {
        cond: Rc<ThrushStatement<'ctx>>,
        block: Rc<ThrushStatement<'ctx>>,
        span: Span,
    },
    Else {
        block: Rc<ThrushStatement<'ctx>>,
        span: Span,
    },

    // Loops
    For {
        local: Rc<ThrushStatement<'ctx>>,
        cond: Rc<ThrushStatement<'ctx>>,
        actions: Rc<ThrushStatement<'ctx>>,
        block: Rc<ThrushStatement<'ctx>>,
        span: Span,
    },
    While {
        cond: Rc<ThrushStatement<'ctx>>,
        block: Rc<ThrushStatement<'ctx>>,
        span: Span,
    },
    Loop {
        block: Rc<ThrushStatement<'ctx>>,
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
        stmts: Vec<ThrushStatement<'ctx>>,
        span: Span,
    },

    // Enums
    Enum {
        name: &'ctx str,
        fields: EnumFields<'ctx>,
        span: Span,
    },
    EnumValue {
        name: String,
        value: Rc<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    // Functions

    // Entrypoint -> fn main() {}
    EntryPoint {
        body: Rc<ThrushStatement<'ctx>>,
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
        parameters: Vec<ThrushStatement<'ctx>>,
        parameter_types: Vec<ThrushType>,
        body: Rc<ThrushStatement<'ctx>>,
        return_type: ThrushType,
        attributes: CompilerAttributes<'ctx>,
        span: Span,
    },
    Return {
        expression: Option<Rc<ThrushStatement<'ctx>>>,
        kind: ThrushType,
        span: Span,
    },

    // Constants
    Const {
        name: &'ctx str,
        kind: ThrushType,
        value: Rc<ThrushStatement<'ctx>>,
        attributes: CompilerAttributes<'ctx>,
        span: Span,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        kind: ThrushType,
        value: Rc<ThrushStatement<'ctx>>,
        is_mutable: bool,
        span: Span,
    },

    // Reference
    Reference {
        name: &'ctx str,
        kind: ThrushType,
        span: Span,
        identificator: ReferenceIndentificator,
    },

    // Mutation
    Mut {
        source: (Option<&'ctx str>, Option<Rc<ThrushStatement<'ctx>>>),
        value: Rc<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    // Low Level Instruction
    LLI {
        name: &'ctx str,
        kind: ThrushType,
        value: Rc<ThrushStatement<'ctx>>,
        span: Span,
    },

    // Pointer Manipulation
    Alloc {
        type_to_alloc: ThrushType,
        site_allocation: LLIAllocationSite,
        attributes: CompilerAttributes<'ctx>,
        span: Span,
    },

    Address {
        name: &'ctx str,
        indexes: Vec<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    Write {
        write_to: (Option<&'ctx str>, Option<Rc<ThrushStatement<'ctx>>>),
        write_value: Rc<ThrushStatement<'ctx>>,
        write_type: ThrushType,
        span: Span,
    },

    Load {
        load: (Option<&'ctx str>, Option<Rc<ThrushStatement<'ctx>>>),
        kind: ThrushType,
        span: Span,
    },

    // Casts
    CastPtr {
        from: Rc<ThrushStatement<'ctx>>,
        cast_type: ThrushType,
        span: Span,
    },

    // Expressions
    Call {
        name: &'ctx str,
        args: Vec<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },
    MethodCall {
        name: String,
        args: Vec<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },
    BinaryOp {
        left: Rc<ThrushStatement<'ctx>>,
        operator: TokenKind,
        right: Rc<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },
    UnaryOp {
        operator: TokenKind,
        kind: ThrushType,
        expression: Rc<ThrushStatement<'ctx>>,
        is_pre: bool,
        span: Span,
    },
    Group {
        expression: Rc<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    Pass {
        span: Span,
    },

    Null {
        span: Span,
    },
}

impl<'ctx> ThrushStatement<'ctx> {
    pub fn get_stmt_type(&self) -> Result<&ThrushType, ThrushCompilerIssue> {
        match self {
            ThrushStatement::Integer { kind, .. } => Ok(kind),
            ThrushStatement::Float { kind, .. } => Ok(kind),
            ThrushStatement::Local { kind, .. } => Ok(kind),
            ThrushStatement::Mut { kind, .. } => Ok(kind),
            ThrushStatement::FunctionParameter { kind, .. } => Ok(kind),
            ThrushStatement::Reference { kind, .. } => Ok(kind),
            ThrushStatement::Call { kind, .. } => Ok(kind),
            ThrushStatement::BinaryOp { kind, .. } => Ok(kind),
            ThrushStatement::Group { kind, .. } => Ok(kind),
            ThrushStatement::UnaryOp { kind, .. } => Ok(kind),

            ThrushStatement::Str { kind, .. } => Ok(kind),
            ThrushStatement::Boolean { kind, .. } => Ok(kind),
            ThrushStatement::Char { kind, .. } => Ok(kind),
            ThrushStatement::Address { .. } => Ok(&ThrushType::Address),
            ThrushStatement::Constructor { kind, .. } => Ok(kind),
            ThrushStatement::Load { kind, .. } => Ok(kind),
            ThrushStatement::Property { kind, .. } => Ok(kind),
            ThrushStatement::NullPtr { .. } => Ok(&ThrushType::Ptr(None)),
            ThrushStatement::This { kind, .. } => Ok(kind),
            ThrushStatement::Alloc {
                type_to_alloc: kind,
                ..
            } => Ok(kind),
            ThrushStatement::MethodCall { kind, .. } => Ok(kind),
            ThrushStatement::BindParameter { kind, .. } => Ok(kind),
            ThrushStatement::EnumValue { kind, .. } => Ok(kind),

            _ => Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected a valid statemant to get a type."),
                None,
                self.get_span(),
            )),
        }
    }

    pub fn get_value_type(&self) -> Result<&ThrushType, ThrushCompilerIssue> {
        match self {
            ThrushStatement::Integer { kind, .. } => Ok(kind),
            ThrushStatement::Float { kind, .. } => Ok(kind),
            ThrushStatement::Mut { kind, .. } => Ok(kind),
            ThrushStatement::Reference { kind, .. } => Ok(kind),
            ThrushStatement::Call { kind, .. } => Ok(kind),
            ThrushStatement::BinaryOp { kind, .. } => Ok(kind),
            ThrushStatement::Group { kind, .. } => Ok(kind),
            ThrushStatement::UnaryOp { kind, .. } => Ok(kind),

            ThrushStatement::Str { kind, .. } => Ok(kind),
            ThrushStatement::Boolean { kind, .. } => Ok(kind),
            ThrushStatement::Char { kind, .. } => Ok(kind),
            ThrushStatement::Address { .. } => Ok(&ThrushType::Address),
            ThrushStatement::Constructor { kind, .. } => Ok(kind),
            ThrushStatement::Load { kind, .. } => Ok(kind),
            ThrushStatement::Property { kind, .. } => Ok(kind),
            ThrushStatement::NullPtr { .. } => Ok(&ThrushType::Ptr(None)),
            ThrushStatement::This { kind, .. } => Ok(kind),
            ThrushStatement::Alloc {
                type_to_alloc: kind,
                ..
            } => Ok(kind),
            ThrushStatement::MethodCall { kind, .. } => Ok(kind),
            ThrushStatement::EnumValue { kind, .. } => Ok(kind),
            ThrushStatement::CastPtr {
                cast_type: kind, ..
            } => Ok(kind),

            _ => Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected a valid statemant to get a type."),
                None,
                self.get_span(),
            )),
        }
    }

    pub fn get_type_unwrapped(&self) -> &ThrushType {
        match self {
            ThrushStatement::Integer { kind, .. } => kind,
            ThrushStatement::Float { kind, .. } => kind,
            ThrushStatement::Local { kind, .. } => kind,
            ThrushStatement::Mut { kind, .. } => kind,
            ThrushStatement::FunctionParameter { kind, .. } => kind,
            ThrushStatement::Reference { kind, .. } => kind,
            ThrushStatement::Call { kind, .. } => kind,
            ThrushStatement::BinaryOp { kind, .. } => kind,
            ThrushStatement::Group { kind, .. } => kind,
            ThrushStatement::UnaryOp { kind, .. } => kind,

            ThrushStatement::Str { kind, .. } => kind,
            ThrushStatement::Boolean { kind, .. } => kind,
            ThrushStatement::Char { kind, .. } => kind,
            ThrushStatement::Address { .. } => &ThrushType::Address,
            ThrushStatement::Constructor { kind, .. } => kind,
            ThrushStatement::Load { kind, .. } => kind,
            ThrushStatement::Property { kind, .. } => kind,
            ThrushStatement::NullPtr { .. } => &ThrushType::Ptr(None),
            ThrushStatement::This { kind, .. } => kind,
            ThrushStatement::MethodCall { kind, .. } => kind,
            ThrushStatement::BindParameter { kind, .. } => kind,
            ThrushStatement::Return { kind, .. } => kind,
            ThrushStatement::EnumValue { kind, .. } => kind,
            ThrushStatement::CastPtr {
                cast_type: kind, ..
            } => kind,

            any => {
                panic!("Attempting to unwrap a null type: {:?}.", any)
            }
        }
    }

    pub fn get_span(&self) -> Span {
        match self {
            ThrushStatement::Integer { span, .. } => *span,
            ThrushStatement::Float { span, .. } => *span,
            ThrushStatement::Local { span, .. } => *span,
            ThrushStatement::Mut { span, .. } => *span,
            ThrushStatement::FunctionParameter { span, .. } => *span,
            ThrushStatement::Reference { span, .. } => *span,
            ThrushStatement::Call { span, .. } => *span,
            ThrushStatement::BinaryOp { span, .. } => *span,
            ThrushStatement::Group { span, .. } => *span,
            ThrushStatement::UnaryOp { span, .. } => *span,
            ThrushStatement::CastPtr { span, .. } => *span,

            ThrushStatement::Str { span, .. } => *span,
            ThrushStatement::Boolean { span, .. } => *span,
            ThrushStatement::Char { span, .. } => *span,
            ThrushStatement::Address { span, .. } => *span,
            ThrushStatement::Constructor { span, .. } => *span,
            ThrushStatement::Load { span, .. } => *span,
            ThrushStatement::Property { span, .. } => *span,
            ThrushStatement::NullPtr { span } => *span,
            ThrushStatement::Write { span, .. } => *span,
            ThrushStatement::Const { span, .. } => *span,
            ThrushStatement::This { span, .. } => *span,
            ThrushStatement::MethodCall { span, .. } => *span,
            ThrushStatement::BindParameter { span, .. } => *span,
            ThrushStatement::Return { span, .. } => *span,
            ThrushStatement::Enum { span, .. } => *span,
            ThrushStatement::EnumValue { span, .. } => *span,
            ThrushStatement::Struct { span, .. } => *span,

            ThrushStatement::Else { span, .. } => *span,
            ThrushStatement::Elif { span, .. } => *span,
            ThrushStatement::If { span, .. } => *span,

            ThrushStatement::Continue { span, .. } => *span,
            ThrushStatement::Break { span, .. } => *span,
            ThrushStatement::While { span, .. } => *span,
            ThrushStatement::For { span, .. } => *span,
            ThrushStatement::Pass { span } => *span,
            ThrushStatement::Method { span, .. } => *span,
            ThrushStatement::Methods { span, .. } => *span,
            ThrushStatement::LLI { span, .. } => *span,

            ThrushStatement::Null { span } => *span,
            ThrushStatement::LLVMValue(_, span) => *span,
            ThrushStatement::Block { span, .. } => *span,
            ThrushStatement::Loop { span, .. } => *span,
            ThrushStatement::EntryPoint { span, .. } => *span,
            ThrushStatement::Function { span, .. } => *span,
            ThrushStatement::Alloc { span, .. } => *span,
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            ThrushStatement::Local { is_mutable, .. } => *is_mutable,
            ThrushStatement::Property { is_mutable, .. } => *is_mutable,
            _ => false,
        }
    }

    pub fn as_llvm_function_proto(&self) -> LLVMFunctionPrototype {
        if let ThrushStatement::Function {
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
        if let ThrushStatement::BinaryOp {
            left,
            operator,
            right,
            ..
        } = self
        {
            return (&**left, operator, &**right);
        }

        if let ThrushStatement::Group { expression, .. } = self {
            return expression.as_binary();
        }

        unreachable!()
    }

    pub fn as_unaryop(&self) -> LLVMUnaryOp {
        if let ThrushStatement::UnaryOp {
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
        if let ThrushStatement::LLVMValue(llvm_value, _) = self {
            return llvm_value;
        }

        unreachable!()
    }

    pub fn get_reference_type(&self) -> Result<ThrushType, ThrushCompilerIssue> {
        if let ThrushStatement::Reference { kind, .. } = self {
            return Ok(kind.clone());
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Reference not caught"),
            String::from("Expected a local reference."),
            self.get_span(),
            CompilationPosition::Parser,
            line!(),
        ))
    }

    pub fn get_method_name(&self) -> Result<&'ctx str, ThrushCompilerIssue> {
        if let ThrushStatement::Method { name, .. } = self {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Method not caught"),
            String::from("Expected a method definition reference."),
            self.get_span(),
            CompilationPosition::Parser,
            line!(),
        ))
    }

    pub fn get_method_parameters_types(&self) -> Result<Vec<ThrushType>, ThrushCompilerIssue> {
        if let ThrushStatement::Method { parameters, .. } = self {
            let mut parameters_types: Vec<ThrushType> = Vec::with_capacity(10);

            for parameter in parameters {
                parameters_types.push(parameter.get_stmt_type()?.clone());
            }

            return Ok(parameters_types);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Method not caught"),
            String::from("Expected a method definition reference."),
            self.get_span(),
            CompilationPosition::Parser,
            line!(),
        ))
    }

    pub fn get_method_type(&self) -> Result<ThrushType, ThrushCompilerIssue> {
        if let ThrushStatement::Method { return_type, .. } = self {
            return Ok(return_type.clone());
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Method not caught"),
            String::from("Expected a method definition reference."),
            self.get_span(),
            CompilationPosition::Parser,
            line!(),
        ))
    }
}

impl<'ctx> ThrushStatement<'ctx> {
    pub fn new_float(
        kind: ThrushType,
        value: f64,
        signed: bool,
        span: Span,
    ) -> ThrushStatement<'ctx> {
        ThrushStatement::Float {
            kind,
            value,
            signed,
            span,
        }
    }

    pub fn new_integer(
        kind: ThrushType,
        value: u64,
        signed: bool,
        span: Span,
    ) -> ThrushStatement<'ctx> {
        ThrushStatement::Integer {
            kind,
            value,
            signed,
            span,
        }
    }

    pub fn new_boolean(kind: ThrushType, value: u64, span: Span) -> ThrushStatement<'ctx> {
        ThrushStatement::Boolean { kind, value, span }
    }

    pub fn new_char(kind: ThrushType, byte: u64, span: Span) -> ThrushStatement<'ctx> {
        ThrushStatement::Char { kind, byte, span }
    }

    pub fn new_str(kind: ThrushType, bytes: Vec<u8>, span: Span) -> ThrushStatement<'ctx> {
        ThrushStatement::Str { kind, bytes, span }
    }
}

impl ThrushStatement<'_> {
    pub fn cast_signess(&mut self, operator: TokenKind) {
        if let ThrushStatement::Integer { kind, signed, .. } = self {
            if operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
                *signed = true;
            }
        }

        if let ThrushStatement::Reference { kind, .. } = self {
            if kind.is_integer_type() && operator.is_minus_operator() {
                *kind = kind.narrowing_cast();
            }
        }

        if let ThrushStatement::Float { signed, .. } = self {
            if operator.is_minus_operator() {
                *signed = true;
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

impl ThrushStatement<'_> {
    pub fn has_block(&self) -> bool {
        if let ThrushStatement::Block { stmts, .. } = self {
            return !stmts.is_empty();
        }

        false
    }

    pub fn has_return(&self) -> bool {
        if let ThrushStatement::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_return());
        }

        false
    }

    pub fn has_break(&self) -> bool {
        if let ThrushStatement::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_break());
        }

        false
    }

    pub fn has_continue(&self) -> bool {
        if let ThrushStatement::Block { stmts, .. } = self {
            return stmts.iter().any(|stmt| stmt.is_continue());
        }

        false
    }

    #[inline]
    pub fn is_block(&self) -> bool {
        matches!(self, ThrushStatement::Block { .. })
    }

    #[inline]
    pub fn is_unsigned_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            ThrushType::U8 | ThrushType::U16 | ThrushType::U32 | ThrushType::U64
        ))
    }

    #[inline]
    pub fn is_anyu32bit_integer(&self) -> Result<bool, ThrushCompilerIssue> {
        Ok(matches!(
            self.get_value_type()?,
            ThrushType::U8 | ThrushType::U16 | ThrushType::U32
        ))
    }

    #[inline]
    pub const fn is_ref(&self) -> bool {
        matches!(self, ThrushStatement::Reference { .. })
    }

    #[inline]
    pub const fn is_ref_lli(&self) -> bool {
        matches!(
            self,
            ThrushStatement::Reference {
                identificator: ReferenceIndentificator::LowLevelInstruction,
                ..
            }
        )
    }

    #[inline]
    pub const fn is_ref_local(&self) -> bool {
        matches!(
            self,
            ThrushStatement::Reference {
                identificator: ReferenceIndentificator::Local,
                ..
            }
        )
    }

    #[inline]
    pub const fn is_pre_unaryop(&self) -> bool {
        matches!(self, ThrushStatement::UnaryOp { is_pre: true, .. })
    }

    #[inline]
    pub const fn is_function(&self) -> bool {
        matches!(self, ThrushStatement::Function { .. })
    }

    #[inline]
    pub const fn is_struct(&self) -> bool {
        matches!(self, ThrushStatement::Struct { .. })
    }

    #[inline]
    pub const fn is_enum(&self) -> bool {
        matches!(self, ThrushStatement::Enum { .. })
    }

    #[inline]
    pub const fn is_constant(&self) -> bool {
        matches!(self, ThrushStatement::Const { .. })
    }

    #[inline]
    pub const fn is_constructor(&self) -> bool {
        matches!(self, ThrushStatement::Constructor { .. })
    }

    #[inline]
    pub const fn is_binary(&self) -> bool {
        matches!(self, ThrushStatement::BinaryOp { .. })
    }

    #[inline]
    pub const fn is_lli(&self) -> bool {
        matches!(
            self,
            ThrushStatement::Write { .. }
                | ThrushStatement::Load { .. }
                | ThrushStatement::Address { .. }
                | ThrushStatement::Alloc { .. }
                | ThrushStatement::CastPtr { .. }
        )
    }

    #[inline]
    pub const fn is_write(&self) -> bool {
        matches!(self, ThrushStatement::Write { .. })
    }

    #[inline]
    pub const fn is_methods(&self) -> bool {
        matches!(self, ThrushStatement::Methods { .. })
    }

    #[inline]
    pub const fn is_group(&self) -> bool {
        matches!(self, ThrushStatement::Group { .. })
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, ThrushStatement::Null { .. })
    }

    #[inline]
    pub const fn is_integer(&self) -> bool {
        matches!(self, ThrushStatement::Integer { .. })
    }

    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(self, ThrushStatement::Boolean { .. })
    }

    #[inline]
    pub const fn is_float(&self) -> bool {
        matches!(self, ThrushStatement::Float { .. })
    }

    #[inline]
    pub const fn is_return(&self) -> bool {
        matches!(self, ThrushStatement::Return { .. })
    }

    #[inline]
    pub const fn is_break(&self) -> bool {
        matches!(self, ThrushStatement::Break { .. })
    }

    #[inline]
    pub const fn is_continue(&self) -> bool {
        matches!(self, ThrushStatement::Continue { .. })
    }
}
