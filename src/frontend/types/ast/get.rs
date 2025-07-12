use crate::{
    core::{
        console::logging::{self, LoggingType},
        errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    },
    frontend::{lexer::span::Span, types::ast::Ast, typesystem::types::Type},
};

impl Ast<'_> {
    pub fn get_any_type(&self) -> Result<&Type, ThrushCompilerIssue> {
        match self {
            // Primitive Types & Literals
            Ast::Integer { kind, .. } => Ok(kind),
            Ast::Float { kind, .. } => Ok(kind),
            Ast::Boolean { kind, .. } => Ok(kind),
            Ast::Char { kind, .. } => Ok(kind),
            Ast::Str { kind, .. } => Ok(kind),
            Ast::NullPtr { .. } => Ok(&Type::Ptr(None)),
            Ast::Null { .. } => Ok(&Type::Void),

            // Static
            Ast::Static { kind, .. } => Ok(kind),

            // Variables & Memory Operations
            Ast::Local { kind, .. } => Ok(kind),
            Ast::Mut { kind, .. } => Ok(kind),
            Ast::Reference { kind, .. } => Ok(kind),
            Ast::Address { kind, .. } => Ok(kind),
            Ast::Load { kind, .. } => Ok(kind),
            Ast::Alloc { alloc: kind, .. } => Ok(kind),
            Ast::Deref { kind, .. } => Ok(kind),
            Ast::Write {
                write_type: kind, ..
            } => Ok(kind),

            // Function-Related Operations
            Ast::FunctionParameter { kind, .. } => Ok(kind),
            Ast::AssemblerFunctionParameter { kind, .. } => Ok(kind),
            Ast::Call { kind, .. } => Ok(kind),
            Ast::Return { kind, .. } => Ok(kind),
            Ast::EntryPoint { .. } => Ok(&Type::Void),
            Ast::Function { return_type, .. } => Ok(return_type),
            Ast::AssemblerFunction { return_type, .. } => Ok(return_type),

            // Expressions & Operators
            Ast::BinaryOp { kind, .. } => Ok(kind),
            Ast::UnaryOp { kind, .. } => Ok(kind),
            Ast::Group { kind, .. } => Ok(kind),
            Ast::Index { kind, .. } => Ok(kind),
            Ast::AsmValue { kind, .. } => Ok(kind),

            // Builtins
            Ast::SizeOf { kind, .. } => Ok(kind),
            Ast::Builtin { kind, .. } => Ok(kind),

            // Composite Types
            Ast::Constructor { kind, .. } => Ok(kind),
            Ast::Property { kind, .. } => Ok(kind),
            Ast::EnumValue { kind, .. } => Ok(kind),
            Ast::FixedArray { kind, .. } => Ok(kind),
            Ast::Array { kind, .. } => Ok(kind),
            Ast::Struct { kind, .. } => Ok(kind),
            Ast::Enum { .. } => Ok(&Type::Void),

            // Type Conversions
            Ast::As { cast, .. } => Ok(cast),

            // Control Flow
            Ast::If { .. } => Ok(&Type::Void),
            Ast::Elif { .. } => Ok(&Type::Void),
            Ast::Else { .. } => Ok(&Type::Void),
            Ast::For { .. } => Ok(&Type::Void),
            Ast::While { .. } => Ok(&Type::Void),
            Ast::Loop { .. } => Ok(&Type::Void),
            Ast::Break { .. } => Ok(&Type::Void),
            Ast::Continue { .. } => Ok(&Type::Void),
            Ast::Block { .. } => Ok(&Type::Void),

            // Constants
            Ast::Const { kind, .. } => Ok(kind),

            // Low-Level Instructions
            Ast::LLI { kind, .. } => Ok(kind),

            // Global Assembler
            Ast::GlobalAssembler { .. } => Ok(&Type::Void),

            // Ignored
            Ast::Pass { .. } => Ok(&Type::Void),
        }
    }

    pub fn get_value_type(&self) -> Result<&Type, ThrushCompilerIssue> {
        match self {
            // Primitive values
            Ast::Integer { kind, .. } => Ok(kind),
            Ast::Float { kind, .. } => Ok(kind),
            Ast::Boolean { kind, .. } => Ok(kind),
            Ast::Char { kind, .. } => Ok(kind),
            Ast::Str { kind, .. } => Ok(kind),
            Ast::NullPtr { .. } => Ok(&Type::Ptr(None)),

            // Variables and references
            Ast::Local { kind, .. } => Ok(kind),
            Ast::Mut { kind, .. } => Ok(kind),
            Ast::Reference { kind, .. } => Ok(kind),
            Ast::FunctionParameter { kind, .. } => Ok(kind),
            Ast::AssemblerFunctionParameter { kind, .. } => Ok(kind),

            // Memory operations
            Ast::Load { kind, .. } => Ok(kind),
            Ast::Address { kind, .. } => Ok(kind),
            Ast::Deref { kind, .. } => Ok(kind),
            Ast::Alloc { alloc: kind, .. } => Ok(kind),

            // Composite types
            Ast::FixedArray { kind, .. } => Ok(kind),
            Ast::Array { kind, .. } => Ok(kind),
            Ast::Constructor { kind, .. } => Ok(kind),
            Ast::Property { kind, .. } => Ok(kind),
            Ast::EnumValue { kind, .. } => Ok(kind),

            // Expressions
            Ast::Call { kind, .. } => Ok(kind),
            Ast::BinaryOp { kind, .. } => Ok(kind),
            Ast::UnaryOp { kind, .. } => Ok(kind),
            Ast::Group { kind, .. } => Ok(kind),
            Ast::Index { kind, .. } => Ok(kind),

            // Type operations
            Ast::As { cast: kind, .. } => Ok(kind),

            // Builtins
            Ast::SizeOf { kind, .. } => Ok(kind),
            Ast::Builtin { kind, .. } => Ok(kind),

            // ASM Code Block
            Ast::AsmValue { kind, .. } => Ok(kind),

            // Global Assembler
            Ast::GlobalAssembler { .. } => Ok(&Type::Void),

            // Ignored
            Ast::Pass { .. } => Ok(&Type::Void),

            _ => Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Expected a value to get a type."),
                None,
                self.get_span(),
            )),
        }
    }

    pub fn get_type_unwrapped(&self) -> &Type {
        match self {
            // Primitive values
            Ast::Integer { kind, .. } => kind,
            Ast::Float { kind, .. } => kind,
            Ast::Boolean { kind, .. } => kind,
            Ast::Char { kind, .. } => kind,
            Ast::Str { kind, .. } => kind,
            Ast::NullPtr { .. } => &Type::Ptr(None),

            // Static
            Ast::Static { kind, .. } => kind,

            // Variables and references
            Ast::Local { kind, .. } => kind,
            Ast::Mut { kind, .. } => kind,
            Ast::Reference { kind, .. } => kind,
            Ast::FunctionParameter { kind, .. } => kind,
            Ast::AssemblerFunctionParameter { kind, .. } => kind,

            // Memory operations
            Ast::Load { kind, .. } => kind,
            Ast::Address { kind, .. } => kind,
            Ast::Deref { kind, .. } => kind,
            Ast::Alloc { alloc: kind, .. } => kind,

            // Composite types
            Ast::FixedArray { kind, .. } => kind,
            Ast::Constructor { kind, .. } => kind,
            Ast::Property { kind, .. } => kind,
            Ast::EnumValue { kind, .. } => kind,

            // Expressions
            Ast::Call { kind, .. } => kind,
            Ast::BinaryOp { kind, .. } => kind,
            Ast::UnaryOp { kind, .. } => kind,
            Ast::Group { kind, .. } => kind,
            Ast::Index { kind, .. } => kind,

            // Type operations
            Ast::As { cast: kind, .. } => kind,

            // Builtins
            Ast::SizeOf { kind, .. } => kind,
            Ast::Builtin { kind, .. } => kind,

            // ASM Code Block
            Ast::AsmValue { kind, .. } => kind,

            // Global Assembler
            Ast::GlobalAssembler { .. } => &Type::Void,

            // Ignored
            Ast::Pass { .. } => &Type::Void,

            any => {
                logging::log(
                    LoggingType::Panic,
                    &format!("Unable to get type of stmt: '{}'.", any),
                );

                unreachable!()
            }
        }
    }

    pub fn get_span(&self) -> Span {
        match self {
            // Primitive values and literals
            Ast::Integer { span, .. } => *span,
            Ast::Float { span, .. } => *span,
            Ast::Boolean { span, .. } => *span,
            Ast::Char { span, .. } => *span,
            Ast::Str { span, .. } => *span,
            Ast::NullPtr { span, .. } => *span,
            Ast::Null { span, .. } => *span,

            // Static
            Ast::Static { span, .. } => *span,

            // Variables and declarations
            Ast::Local { span, .. } => *span,
            Ast::Const { span, .. } => *span,
            Ast::FunctionParameter { span, .. } => *span,
            Ast::AssemblerFunctionParameter { span, .. } => *span,

            // Memory operations
            Ast::Mut { span, .. } => *span,
            Ast::Reference { span, .. } => *span,
            Ast::Address { span, .. } => *span,
            Ast::Load { span, .. } => *span,
            Ast::Deref { span, .. } => *span,
            Ast::Write { span, .. } => *span,
            Ast::Alloc { span, .. } => *span,

            // Composite types
            Ast::FixedArray { span, .. } => *span,
            Ast::Array { span, .. } => *span,

            Ast::Struct { span, .. } => *span,
            Ast::Enum { span, .. } => *span,
            Ast::EnumValue { span, .. } => *span,
            Ast::Constructor { span, .. } => *span,
            Ast::Property { span, .. } => *span,

            // Expressions and operators
            Ast::Call { span, .. } => *span,
            Ast::BinaryOp { span, .. } => *span,
            Ast::UnaryOp { span, .. } => *span,
            Ast::Group { span, .. } => *span,
            Ast::Index { span, .. } => *span,

            // Type conversions
            Ast::As { span, .. } => *span,

            // Builtins
            Ast::SizeOf { span, .. } => *span,
            Ast::Builtin { span, .. } => *span,

            // Control flow
            Ast::If { span, .. } => *span,
            Ast::Elif { span, .. } => *span,
            Ast::Else { span, .. } => *span,
            Ast::While { span, .. } => *span,
            Ast::For { span, .. } => *span,
            Ast::Loop { span, .. } => *span,
            Ast::Break { span, .. } => *span,
            Ast::Continue { span, .. } => *span,
            Ast::Block { span, .. } => *span,

            // Functions
            Ast::Function { span, .. } => *span,
            Ast::AssemblerFunction { span, .. } => *span,
            Ast::EntryPoint { span, .. } => *span,
            Ast::Return { span, .. } => *span,

            // Low-level and special operations
            Ast::AsmValue { span, .. } => *span,
            Ast::LLI { span, .. } => *span,
            Ast::Pass { span, .. } => *span,

            // Global Assembler
            Ast::GlobalAssembler { span, .. } => *span,
        }
    }

    pub fn get_str_content(&self, span: Span) -> Result<&str, ThrushCompilerIssue> {
        if let Ast::Str { bytes, .. } = self {
            if let Ok(content) = std::str::from_utf8(bytes) {
                return Ok(content);
            }

            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected string literal.".into(),
                None,
                span,
            ));
        }

        Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Expected string literal.".into(),
            None,
            span,
        ))
    }

    pub fn get_integer_value(&self) -> Result<u64, ThrushCompilerIssue> {
        if let Ast::Integer { value, .. } = self {
            return Ok(*value);
        }

        Err(ThrushCompilerIssue::FrontEndBug(
            String::from("Integer not caught"),
            String::from("Expected a integer value"),
            self.get_span(),
            CompilationPosition::Parser,
            line!(),
        ))
    }
}
