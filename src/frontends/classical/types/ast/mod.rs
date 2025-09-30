#![allow(clippy::upper_case_acronyms)]

mod get;
mod impls;
mod is;
pub mod metadata;
mod new;
mod repr;
pub mod traits;
pub mod types;

use std::rc::Rc;

use crate::{
    backends::classical::llvm::compiler::builtins::Builtin,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        types::{
            ast::{
                metadata::{
                    cast::CastMetadata, constant::ConstantMetadata,
                    dereference::DereferenceMetadata, fnparam::FunctionParameterMetadata,
                    index::IndexMetadata, local::LocalMetadata, property::PropertyMetadata,
                    reference::ReferenceMetadata, staticvar::StaticMetadata,
                },
                types::AstEitherExpression,
            },
            parser::stmts::{
                sites::AllocationSite,
                types::{Constructor, EnumFields, StructFields, ThrushAttributes},
            },
        },
        typesystem::types::Type,
    },
};

#[derive(Debug, Clone)]
pub enum Ast<'ctx> {
    Str {
        bytes: Vec<u8>,
        kind: Type,
        span: Span,
    },

    Char {
        kind: Type,
        byte: u64,
        span: Span,
    },

    Boolean {
        kind: Type,
        value: u64,
        span: Span,
    },

    Integer {
        kind: Type,
        value: u64,
        signed: bool,
        span: Span,
    },

    Float {
        kind: Type,
        value: f64,
        signed: bool,
        span: Span,
    },

    // Global Assembler
    GlobalAssembler {
        asm: String,
        span: Span,
    },

    // Fixed Array
    FixedArray {
        items: Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Array
    Array {
        items: Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    Index {
        source: AstEitherExpression<'ctx>,
        indexes: Vec<Ast<'ctx>>,
        kind: Type,
        metadata: IndexMetadata,
        span: Span,
    },

    NullPtr {
        span: Span,
    },

    // Structures
    Struct {
        name: &'ctx str,
        fields: StructFields<'ctx>,
        kind: Type,
        span: Span,
        attributes: ThrushAttributes<'ctx>,
    },

    Constructor {
        name: &'ctx str,
        args: Constructor<'ctx>,
        kind: Type,
        span: Span,
    },

    Property {
        source: AstEitherExpression<'ctx>,
        indexes: Vec<(Type, u32)>,
        metadata: PropertyMetadata,
        kind: Type,
        span: Span,
    },

    // Conditionals
    If {
        condition: Rc<Ast<'ctx>>,
        block: Rc<Ast<'ctx>>,
        elseif: Vec<Ast<'ctx>>,
        anyway: Option<Rc<Ast<'ctx>>>,
        span: Span,
    },
    Elif {
        condition: Rc<Ast<'ctx>>,
        block: Rc<Ast<'ctx>>,
        span: Span,
    },
    Else {
        block: Rc<Ast<'ctx>>,
        span: Span,
    },

    // Loops
    For {
        local: Rc<Ast<'ctx>>,
        cond: Rc<Ast<'ctx>>,
        actions: Rc<Ast<'ctx>>,
        block: Rc<Ast<'ctx>>,
        span: Span,
    },
    While {
        cond: Rc<Ast<'ctx>>,
        block: Rc<Ast<'ctx>>,
        span: Span,
    },
    Loop {
        block: Rc<Ast<'ctx>>,
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
        stmts: Vec<Ast<'ctx>>,
        span: Span,
    },

    // Enums
    Enum {
        name: &'ctx str,
        fields: EnumFields<'ctx>,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },
    EnumValue {
        name: String,
        value: Rc<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Functions

    // Entrypoint
    EntryPoint {
        body: Rc<Ast<'ctx>>,
        parameters: Vec<Ast<'ctx>>,
        span: Span,
    },
    AssemblerFunction {
        name: &'ctx str,
        ascii_name: &'ctx str,
        parameters: Vec<Ast<'ctx>>,
        parameters_types: Vec<Type>,
        assembler: String,
        constraints: String,
        return_type: Type,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },
    AssemblerFunctionParameter {
        name: &'ctx str,
        kind: Type,
        position: u32,
        span: Span,
    },
    Function {
        name: &'ctx str,
        ascii_name: &'ctx str,
        parameters: Vec<Ast<'ctx>>,
        parameter_types: Vec<Type>,
        body: Rc<Ast<'ctx>>,
        return_type: Type,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },
    FunctionParameter {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        position: u32,
        metadata: FunctionParameterMetadata,
        span: Span,
    },
    Return {
        expression: Option<Rc<Ast<'ctx>>>,
        kind: Type,
        span: Span,
    },

    // Static
    Static {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: Rc<Ast<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
        metadata: StaticMetadata,
        span: Span,
    },

    // Constants
    Const {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: Rc<Ast<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
        metadata: ConstantMetadata,
        span: Span,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: Rc<Ast<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
        metadata: LocalMetadata,
        span: Span,
    },

    // Reference
    Reference {
        name: &'ctx str,
        kind: Type,
        metadata: ReferenceMetadata,
        span: Span,
    },

    // Mutation
    Mut {
        source: Rc<Ast<'ctx>>,
        value: Rc<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Low Level Instruction
    LLI {
        name: &'ctx str,
        kind: Type,
        value: Rc<Ast<'ctx>>,
        span: Span,
    },

    // Pointer Manipulation
    Alloc {
        alloc: Type,
        site_allocation: AllocationSite,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },

    Address {
        source: AstEitherExpression<'ctx>,
        indexes: Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    Write {
        source: AstEitherExpression<'ctx>,
        write_value: Rc<Ast<'ctx>>,
        write_type: Type,
        span: Span,
    },

    Load {
        source: AstEitherExpression<'ctx>,
        kind: Type,
        span: Span,
    },

    Deref {
        value: Rc<Ast<'ctx>>,
        kind: Type,
        metadata: DereferenceMetadata,
        span: Span,
    },

    // Casts
    As {
        from: Rc<Ast<'ctx>>,
        cast: Type,
        metadata: CastMetadata,
        span: Span,
    },

    // Expressions
    Call {
        name: &'ctx str,
        args: Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    Indirect {
        function: Rc<Ast<'ctx>>,
        function_type: Type,
        args: Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    AsmValue {
        assembler: String,
        constraints: String,
        args: Vec<Ast<'ctx>>,
        kind: Type,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },

    BinaryOp {
        left: Rc<Ast<'ctx>>,
        operator: TokenType,
        right: Rc<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    UnaryOp {
        operator: TokenType,
        kind: Type,
        expression: Rc<Ast<'ctx>>,
        is_pre: bool,
        span: Span,
    },

    Group {
        expression: Rc<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Builtins
    SizeOf {
        sizeof: Type,
        kind: Type,
        span: Span,
    },

    Builtin {
        builtin: Builtin<'ctx>,
        kind: Type,
        span: Span,
    },

    // Extra
    Pass {
        span: Span,
    },

    // Unreachable
    Unreachable {
        span: Span,
    },

    Null {
        span: Span,
    },
}
