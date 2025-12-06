#![allow(clippy::upper_case_acronyms)]

use crate::core::diagnostic::span::Span;

use crate::middle_end::mir::attributes::ThrushAttributes;
use crate::middle_end::mir::builtins::ThrushBuiltin;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::ast::metadata::cast::CastMetadata;
use crate::front_end::types::ast::metadata::constant::ConstantMetadata;
use crate::front_end::types::ast::metadata::dereference::DereferenceMetadata;
use crate::front_end::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::front_end::types::ast::metadata::index::IndexMetadata;
use crate::front_end::types::ast::metadata::local::LocalMetadata;
use crate::front_end::types::ast::metadata::property::PropertyMetadata;
use crate::front_end::types::ast::metadata::reference::ReferenceMetadata;
use crate::front_end::types::ast::metadata::staticvar::StaticMetadata;
use crate::front_end::types::parser::stmts::sites::AllocationSite;
use crate::front_end::types::parser::stmts::types::Constructor;
use crate::front_end::types::parser::stmts::types::EnumFields;
use crate::front_end::types::parser::stmts::types::StructFields;
use crate::front_end::typesystem::types::Type;

pub mod get;
pub mod impls;
pub mod is;
pub mod metadata;
pub mod new;
pub mod repr;
pub mod traits;

#[derive(Debug, Clone)]
pub enum Ast<'ctx> {
    Str {
        bytes: std::vec::Vec<u8>,
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

    NullPtr {
        span: Span,
        kind: Type,
    },

    // Global Assembler
    GlobalAssembler {
        asm: String,
        span: Span,
    },

    // Fixed Array
    FixedArray {
        items: std::vec::Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Array
    Array {
        items: std::vec::Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },
    Index {
        source: std::boxed::Box<Ast<'ctx>>,
        index: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        metadata: IndexMetadata,
        span: Span,
    },

    // Structures
    Struct {
        name: &'ctx str,
        fields: StructFields<'ctx>,
        kind: Type,
        span: Span,
        attributes: ThrushAttributes,
    },

    Constructor {
        name: &'ctx str,
        args: Constructor<'ctx>,
        kind: Type,
        span: Span,
    },

    Property {
        source: std::boxed::Box<Ast<'ctx>>,
        indexes: std::vec::Vec<(Type, u32)>,
        metadata: PropertyMetadata,
        kind: Type,
        span: Span,
    },

    // Conditionals
    If {
        condition: std::boxed::Box<Ast<'ctx>>,
        block: std::boxed::Box<Ast<'ctx>>,
        elseif: std::vec::Vec<Ast<'ctx>>,
        anyway: Option<std::boxed::Box<Ast<'ctx>>>,
        span: Span,
    },
    Elif {
        condition: std::boxed::Box<Ast<'ctx>>,
        block: std::boxed::Box<Ast<'ctx>>,
        span: Span,
    },
    Else {
        block: std::boxed::Box<Ast<'ctx>>,
        span: Span,
    },

    // Loops
    For {
        local: std::boxed::Box<Ast<'ctx>>,
        condition: std::boxed::Box<Ast<'ctx>>,
        actions: std::boxed::Box<Ast<'ctx>>,
        block: std::boxed::Box<Ast<'ctx>>,
        span: Span,
    },
    While {
        condition: std::boxed::Box<Ast<'ctx>>,
        block: std::boxed::Box<Ast<'ctx>>,
        span: Span,
    },
    Loop {
        block: std::boxed::Box<Ast<'ctx>>,
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
        nodes: std::vec::Vec<Ast<'ctx>>,
        span: Span,
    },

    // Custom Type
    CustomType {
        kind: Type,
        span: Span,
    },

    // Enums
    Enum {
        name: &'ctx str,
        fields: EnumFields<'ctx>,
        attributes: ThrushAttributes,
        span: Span,
    },
    EnumValue {
        name: String,
        value: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Functions
    Intrinsic {
        name: &'ctx str,
        external_name: &'ctx str,
        parameters: std::vec::Vec<Ast<'ctx>>,
        parameters_types: std::vec::Vec<Type>,
        return_type: Type,
        attributes: ThrushAttributes,
        span: Span,
    },
    IntrinsicParameter {
        kind: Type,
        span: Span,
    },
    AssemblerFunction {
        name: &'ctx str,
        ascii_name: &'ctx str,
        parameters: std::vec::Vec<Ast<'ctx>>,
        parameters_types: std::vec::Vec<Type>,
        assembler: String,
        constraints: String,
        return_type: Type,
        attributes: ThrushAttributes,
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
        parameters: std::vec::Vec<Ast<'ctx>>,
        parameter_types: std::vec::Vec<Type>,
        body: Option<std::boxed::Box<Ast<'ctx>>>,
        return_type: Type,
        attributes: ThrushAttributes,
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
        expression: Option<std::boxed::Box<Ast<'ctx>>>,
        kind: Type,
        span: Span,
    },

    // Static
    Static {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: Option<std::boxed::Box<Ast<'ctx>>>,
        attributes: ThrushAttributes,
        metadata: StaticMetadata,
        span: Span,
    },

    // Constants
    Const {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: std::boxed::Box<Ast<'ctx>>,
        attributes: ThrushAttributes,
        metadata: ConstantMetadata,
        span: Span,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: Option<std::boxed::Box<Ast<'ctx>>>,
        attributes: ThrushAttributes,
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
        source: std::boxed::Box<Ast<'ctx>>,
        value: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Low Level Instruction
    LLI {
        name: &'ctx str,
        kind: Type,
        expr: std::boxed::Box<Ast<'ctx>>,
        span: Span,
    },

    // LLI
    Alloc {
        alloc: Type,
        site_allocation: AllocationSite,
        attributes: ThrushAttributes,
        span: Span,
    },

    Address {
        source: std::boxed::Box<Ast<'ctx>>,
        indexes: std::vec::Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    Write {
        source: std::boxed::Box<Ast<'ctx>>,
        write_value: std::boxed::Box<Ast<'ctx>>,
        write_type: Type,
        span: Span,
    },
    Load {
        source: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Pointer Manipulation
    Deref {
        value: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        metadata: DereferenceMetadata,
        span: Span,
    },

    // Casts
    As {
        from: std::boxed::Box<Ast<'ctx>>,
        cast: Type,
        metadata: CastMetadata,
        span: Span,
    },

    // Expressions
    DirectRef {
        expr: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    Call {
        name: &'ctx str,
        args: std::vec::Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    Indirect {
        function: std::boxed::Box<Ast<'ctx>>,
        function_type: Type,
        args: std::vec::Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    AsmValue {
        assembler: String,
        constraints: String,
        args: std::vec::Vec<Ast<'ctx>>,
        kind: Type,
        attributes: ThrushAttributes,
        span: Span,
    },

    BinaryOp {
        left: std::boxed::Box<Ast<'ctx>>,
        operator: TokenType,
        right: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    UnaryOp {
        operator: TokenType,
        kind: Type,
        expression: std::boxed::Box<Ast<'ctx>>,
        is_pre: bool,
        span: Span,
    },

    Group {
        expression: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Builtins
    Builtin {
        builtin: ThrushBuiltin<'ctx>,
        kind: Type,
        span: Span,
    },

    // Module Import
    Import {
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
}
