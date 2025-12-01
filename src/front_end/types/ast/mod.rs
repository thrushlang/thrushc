#![allow(clippy::upper_case_acronyms)]

use crate::back_end::llvm_codegen::builtins::Builtin;

use crate::core::diagnostic::span::Span;

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
use crate::middle_end::mir::attributes::ThrushAttributes;

use std::rc::Rc;

pub mod get;
pub mod is;
pub mod metadata;
pub mod new;
pub mod repr;
pub mod traits;

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
        source: Rc<Ast<'ctx>>,
        index: Rc<Ast<'ctx>>,
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
        source: Rc<Ast<'ctx>>,
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
        condition: Rc<Ast<'ctx>>,
        actions: Rc<Ast<'ctx>>,
        block: Rc<Ast<'ctx>>,
        span: Span,
    },
    While {
        condition: Rc<Ast<'ctx>>,
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
        nodes: Vec<Ast<'ctx>>,
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
        value: Rc<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Functions
    Intrinsic {
        name: &'ctx str,
        external_name: &'ctx str,
        parameters: Vec<Ast<'ctx>>,
        parameters_types: Vec<Type>,
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
        parameters: Vec<Ast<'ctx>>,
        parameters_types: Vec<Type>,
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
        parameters: Vec<Ast<'ctx>>,
        parameter_types: Vec<Type>,
        body: Option<Rc<Ast<'ctx>>>,
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
        expression: Option<Rc<Ast<'ctx>>>,
        kind: Type,
        span: Span,
    },

    // Static
    Static {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: Option<Rc<Ast<'ctx>>>,
        attributes: ThrushAttributes,
        metadata: StaticMetadata,
        span: Span,
    },

    // Constants
    Const {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: Rc<Ast<'ctx>>,
        attributes: ThrushAttributes,
        metadata: ConstantMetadata,
        span: Span,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: Type,
        value: Option<Rc<Ast<'ctx>>>,
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
        source: Rc<Ast<'ctx>>,
        value: Rc<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Low Level Instruction
    LLI {
        name: &'ctx str,
        kind: Type,
        expr: Rc<Ast<'ctx>>,
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
        source: Rc<Ast<'ctx>>,
        indexes: Vec<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    Write {
        source: Rc<Ast<'ctx>>,
        write_value: Rc<Ast<'ctx>>,
        write_type: Type,
        span: Span,
    },
    Load {
        source: Rc<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Pointer Manipulation
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
    DirectRef {
        expr: Rc<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

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
        attributes: ThrushAttributes,
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
    Builtin {
        builtin: Builtin<'ctx>,
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
