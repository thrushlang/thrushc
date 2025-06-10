#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]

use std::rc::Rc;

use crate::{
    core::{
        console::logging::{self, LoggingType},
        errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    },
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        types::{
            lexer::ThrushType,
            representations::{AssemblerFunctionRepresentation, FunctionRepresentation},
        },
    },
};

use super::{
    ident::ReferenceIndentificator,
    sites::LLIAllocationSite,
    types::{Constructor, EnumFields, StructFields, ThrushAttributes},
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

    // Arrays
    Array {
        items: Vec<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    Index {
        name: &'ctx str,
        reference: Rc<ThrushStatement<'ctx>>,
        indexes: Vec<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    NullPtr {
        span: Span,
    },

    // Structures
    Struct {
        name: &'ctx str,
        fields: StructFields<'ctx>,
        kind: ThrushType,
        span: Span,
        attributes: ThrushAttributes<'ctx>,
    },

    Constructor {
        name: &'ctx str,
        arguments: Constructor<'ctx>,
        kind: ThrushType,
        span: Span,
    },

    Methods {
        name: String,
        methods: Vec<ThrushStatement<'ctx>>,
        span: Span,
    },

    Method {
        name: &'ctx str,
        parameters: Vec<ThrushStatement<'ctx>>,
        parameters_types: Vec<ThrushType>,
        body: Rc<ThrushStatement<'ctx>>,
        return_type: ThrushType,
        attributes: ThrushAttributes<'ctx>,
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
        reference: Rc<ThrushStatement<'ctx>>,
        indexes: Vec<(ThrushType, u32)>,
        kind: ThrushType,
        span: Span,
    },

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
    AssemblerFunction {
        name: &'ctx str,
        ascii_name: &'ctx str,
        parameters: Vec<ThrushStatement<'ctx>>,
        parameters_types: Vec<ThrushType>,
        assembler: String,
        constraints: String,
        return_type: ThrushType,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },
    AssemblerFunctionParameter {
        name: &'ctx str,
        kind: ThrushType,
        position: u32,
        span: Span,
    },
    Function {
        name: &'ctx str,
        ascii_name: &'ctx str,
        parameters: Vec<ThrushStatement<'ctx>>,
        parameter_types: Vec<ThrushType>,
        body: Rc<ThrushStatement<'ctx>>,
        return_type: ThrushType,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },
    FunctionParameter {
        name: &'ctx str,
        kind: ThrushType,
        position: u32,
        is_mutable: bool,
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
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        kind: ThrushType,
        value: Rc<ThrushStatement<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
        is_mutable: bool,
        span: Span,
    },

    // Reference
    Reference {
        name: &'ctx str,
        kind: ThrushType,
        span: Span,
        identificator: ReferenceIndentificator,
        is_mutable: bool,
        is_allocated: bool,
    },

    // Mutation
    Mut {
        source: (
            Option<(&'ctx str, Rc<ThrushStatement<'ctx>>)>,
            Option<Rc<ThrushStatement<'ctx>>>,
        ),
        value: Rc<ThrushStatement<'ctx>>,
        kind: ThrushType,
        cast_type: ThrushType,
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
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },

    Address {
        address_to: (
            Option<(&'ctx str, Rc<ThrushStatement<'ctx>>)>,
            Option<Rc<ThrushStatement<'ctx>>>,
        ),
        indexes: Vec<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    Write {
        write_to: (
            Option<(&'ctx str, Rc<ThrushStatement<'ctx>>)>,
            Option<Rc<ThrushStatement<'ctx>>>,
        ),
        write_value: Rc<ThrushStatement<'ctx>>,
        write_type: ThrushType,
        span: Span,
    },

    Load {
        value: (
            Option<(&'ctx str, Rc<ThrushStatement<'ctx>>)>,
            Option<Rc<ThrushStatement<'ctx>>>,
        ),
        kind: ThrushType,
        span: Span,
    },

    Deref {
        value: Rc<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    // Casts
    CastRaw {
        from: Rc<ThrushStatement<'ctx>>,
        cast: ThrushType,
        span: Span,
    },

    Cast {
        from: Rc<ThrushStatement<'ctx>>,
        cast: ThrushType,
        span: Span,
    },

    CastPtr {
        from: Rc<ThrushStatement<'ctx>>,
        cast: ThrushType,
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

    AsmValue {
        assembler: String,
        constraints: String,
        args: Vec<ThrushStatement<'ctx>>,
        kind: ThrushType,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },

    BinaryOp {
        left: Rc<ThrushStatement<'ctx>>,
        operator: TokenType,
        right: Rc<ThrushStatement<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    UnaryOp {
        operator: TokenType,
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
            ThrushStatement::AssemblerFunctionParameter { kind, .. } => Ok(kind),
            ThrushStatement::FunctionParameter { kind, .. } => Ok(kind),
            ThrushStatement::Reference { kind, .. } => Ok(kind),
            ThrushStatement::Call { kind, .. } => Ok(kind),
            ThrushStatement::BinaryOp { kind, .. } => Ok(kind),
            ThrushStatement::Group { kind, .. } => Ok(kind),
            ThrushStatement::UnaryOp { kind, .. } => Ok(kind),

            ThrushStatement::Str { kind, .. } => Ok(kind),
            ThrushStatement::Boolean { kind, .. } => Ok(kind),
            ThrushStatement::Char { kind, .. } => Ok(kind),
            ThrushStatement::Address { kind, .. } => Ok(kind),
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
            ThrushStatement::Array { kind, .. } => Ok(kind),
            ThrushStatement::Index { kind, .. } => Ok(kind),

            _ => Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected a valid statement to get a type."),
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
            ThrushStatement::Array { kind, .. } => Ok(kind),
            ThrushStatement::Index { kind, .. } => Ok(kind),
            ThrushStatement::Address { kind, .. } => Ok(kind),
            ThrushStatement::Constructor { kind, .. } => Ok(kind),
            ThrushStatement::Load { kind, .. } => Ok(kind),
            ThrushStatement::Property { kind, .. } => Ok(kind),
            ThrushStatement::NullPtr { .. } => Ok(&ThrushType::Ptr(None)),
            ThrushStatement::This { kind, .. } => Ok(kind),
            ThrushStatement::Alloc {
                type_to_alloc: kind,
                ..
            } => Ok(kind),
            ThrushStatement::FunctionParameter { kind, .. } => Ok(kind),
            ThrushStatement::AssemblerFunctionParameter { kind, .. } => Ok(kind),
            ThrushStatement::MethodCall { kind, .. } => Ok(kind),
            ThrushStatement::EnumValue { kind, .. } => Ok(kind),
            ThrushStatement::Deref { kind, .. } => Ok(kind),
            ThrushStatement::CastRaw { cast: kind, .. } => Ok(kind),
            ThrushStatement::Cast { cast: kind, .. } => Ok(kind),
            ThrushStatement::CastPtr { cast: kind, .. } => Ok(kind),
            ThrushStatement::AsmValue { kind, .. } => Ok(kind),

            _ => Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected a value to get a type."),
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
            ThrushStatement::AssemblerFunctionParameter { kind, .. } => kind,
            ThrushStatement::Reference { kind, .. } => kind,
            ThrushStatement::Call { kind, .. } => kind,
            ThrushStatement::BinaryOp { kind, .. } => kind,
            ThrushStatement::Group { kind, .. } => kind,
            ThrushStatement::UnaryOp { kind, .. } => kind,

            ThrushStatement::Str { kind, .. } => kind,
            ThrushStatement::Boolean { kind, .. } => kind,
            ThrushStatement::Array { kind, .. } => kind,
            ThrushStatement::Char { kind, .. } => kind,
            ThrushStatement::Address { kind, .. } => kind,
            ThrushStatement::Constructor { kind, .. } => kind,
            ThrushStatement::Load { kind, .. } => kind,
            ThrushStatement::Property { kind, .. } => kind,
            ThrushStatement::NullPtr { .. } => &ThrushType::Ptr(None),
            ThrushStatement::This { kind, .. } => kind,
            ThrushStatement::MethodCall { kind, .. } => kind,
            ThrushStatement::BindParameter { kind, .. } => kind,
            ThrushStatement::Return { kind, .. } => kind,
            ThrushStatement::EnumValue { kind, .. } => kind,
            ThrushStatement::Deref { kind, .. } => kind,
            ThrushStatement::AsmValue { kind, .. } => kind,

            ThrushStatement::CastPtr { cast: kind, .. } => kind,
            ThrushStatement::CastRaw { cast: kind, .. } => kind,
            ThrushStatement::Cast { cast: kind, .. } => kind,

            any => {
                logging::log(
                    LoggingType::Panic,
                    &format!("Unable to get type of stmt: '{}'", any),
                );

                unreachable!()
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
            ThrushStatement::CastRaw { span, .. } => *span,
            ThrushStatement::Cast { span, .. } => *span,
            ThrushStatement::Deref { span, .. } => *span,
            ThrushStatement::CastPtr { span, .. } => *span,
            ThrushStatement::AsmValue { span, .. } => *span,

            ThrushStatement::Str { span, .. } => *span,
            ThrushStatement::Boolean { span, .. } => *span,
            ThrushStatement::Array { span, .. } => *span,
            ThrushStatement::Index { span, .. } => *span,
            ThrushStatement::Char { span, .. } => *span,
            ThrushStatement::Address { span, .. } => *span,
            ThrushStatement::Constructor { span, .. } => *span,
            ThrushStatement::Load { span, .. } => *span,
            ThrushStatement::Property { span, .. } => *span,
            ThrushStatement::NullPtr { span } => *span,
            ThrushStatement::Write { span, .. } => *span,
            ThrushStatement::Const { span, .. } => *span,
            ThrushStatement::This { span, .. } => *span,
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
            ThrushStatement::Block { span, .. } => *span,
            ThrushStatement::Loop { span, .. } => *span,
            ThrushStatement::EntryPoint { span, .. } => *span,
            ThrushStatement::Function { span, .. } => *span,
            ThrushStatement::AssemblerFunction { span, .. } => *span,
            ThrushStatement::AssemblerFunctionParameter { span, .. } => *span,
            ThrushStatement::Alloc { span, .. } => *span,

            ThrushStatement::MethodCall { span, .. } => *span,
        }
    }

    pub fn is_mutable(&self) -> bool {
        if let ThrushStatement::Local { is_mutable, .. } = self {
            return *is_mutable;
        }

        if let ThrushStatement::Reference {
            is_mutable, kind, ..
        } = self
        {
            return *is_mutable || kind.is_ptr_type() || kind.is_address_type();
        }

        if let ThrushStatement::Property { reference, .. } = self {
            return reference.is_mutable();
        }

        if let ThrushStatement::Index { reference, .. } = self {
            return reference.is_mutable();
        }

        false
    }

    pub fn as_asm_function_representation(&self) -> AssemblerFunctionRepresentation {
        if let ThrushStatement::AssemblerFunction {
            name,
            ascii_name,
            assembler,
            constraints,
            parameters_types,
            parameters,
            return_type,
            attributes,
            ..
        } = self
        {
            return (
                name,
                ascii_name,
                assembler,
                constraints,
                return_type,
                parameters,
                parameters_types,
                attributes,
            );
        }

        unreachable!()
    }

    pub fn as_function_representation(&self) -> FunctionRepresentation {
        if let ThrushStatement::Function {
            name,
            ascii_name,
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
                ascii_name,
                return_type,
                parameters,
                parameter_types,
                body,
                attributes,
            );
        }

        unreachable!()
    }

    pub fn get_reference_type(&self) -> Result<ThrushType, ThrushCompilerIssue> {
        if let ThrushStatement::Reference { kind, .. } = self {
            return Ok(kind.clone());
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Reference not caught"),
            String::from("Expected a reference."),
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

    pub fn get_str_content(&self) -> Result<&str, ThrushCompilerIssue> {
        if let ThrushStatement::Str { bytes, .. } = self {
            if let Ok(content) = std::str::from_utf8(bytes) {
                return Ok(content);
            }

            return Err(ThrushCompilerIssue::Bug(
                String::from("Not parsed"),
                String::from("Could not process a str as a utf-8 value."),
                self.get_span(),
                CompilationPosition::Parser,
                line!(),
            ));
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Str not caught"),
            String::from("Expected a str value."),
            self.get_span(),
            CompilationPosition::Parser,
            line!(),
        ))
    }

    pub fn get_unwrapped_reference_name(&self) -> &str {
        if let ThrushStatement::Reference { name, .. } = self {
            return name;
        }

        logging::log(LoggingType::Bug, "Unable to get a reference name.");
        unreachable!()
    }

    pub fn get_integer_value(&self) -> Result<u64, ThrushCompilerIssue> {
        if let ThrushStatement::Integer { value, .. } = self {
            return Ok(*value);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Integer not caught"),
            String::from("Expected a integer value"),
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
    pub fn cast_signess(&mut self, operator: TokenType) {
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
    pub const fn is_reference(&self) -> bool {
        matches!(self, ThrushStatement::Reference { .. })
    }

    #[inline]
    pub fn is_constant_array(&self) -> bool {
        match self {
            ThrushStatement::Array { items, .. } => {
                items.iter().all(|item| item.is_constant_array())
            }
            ThrushStatement::Reference { .. } => false,
            _ => true,
        }
    }

    #[inline]
    pub const fn is_allocated_reference(&self) -> bool {
        matches!(
            self,
            ThrushStatement::Reference {
                is_allocated: true,
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

    pub const fn is_asm_function(&self) -> bool {
        matches!(self, ThrushStatement::AssemblerFunction { .. })
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
    pub const fn is_str(&self) -> bool {
        matches!(self, ThrushStatement::Str { .. })
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
