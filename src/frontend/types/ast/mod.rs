#![allow(clippy::upper_case_acronyms)]

mod cast;
mod get;
mod is;
mod new;
mod repr;
pub mod types;

use std::rc::Rc;

use crate::{
    backend::llvm::compiler::builtins::Builtin,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        types::{
            ast::types::AstEitherExpression,
            lexer::ThrushType,
            parser::stmts::{
                ident::ReferenceIdentificator,
                sites::LLIAllocationSite,
                types::{Constructor, EnumFields, StructFields, ThrushAttributes},
            },
        },
    },
};

#[derive(Debug, Clone)]
pub enum Ast<'ctx> {
    Str {
        bytes: Vec<u8>,
        kind: ThrushType,
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

    // Fixed Array
    FixedArray {
        items: Vec<Ast<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    // Array
    Array {
        items: Vec<Ast<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    Index {
        index_to: AstEitherExpression<'ctx>,
        indexes: Vec<Ast<'ctx>>,
        is_mutable: bool,
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

    Property {
        name: &'ctx str,
        reference: Rc<Ast<'ctx>>,
        indexes: Vec<(ThrushType, u32)>,
        kind: ThrushType,
        span: Span,
    },

    // Conditionals
    If {
        cond: Rc<Ast<'ctx>>,
        block: Rc<Ast<'ctx>>,
        elfs: Vec<Ast<'ctx>>,
        otherwise: Option<Rc<Ast<'ctx>>>,
        span: Span,
    },
    Elif {
        cond: Rc<Ast<'ctx>>,
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
        span: Span,
    },
    EnumValue {
        name: String,
        value: Rc<Ast<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    // Functions

    // Entrypoint -> fn main() {}
    EntryPoint {
        body: Rc<Ast<'ctx>>,
        span: Span,
    },
    AssemblerFunction {
        name: &'ctx str,
        ascii_name: &'ctx str,
        parameters: Vec<Ast<'ctx>>,
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
        parameters: Vec<Ast<'ctx>>,
        parameter_types: Vec<ThrushType>,
        body: Rc<Ast<'ctx>>,
        return_type: ThrushType,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },
    FunctionParameter {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: ThrushType,
        position: u32,
        is_mutable: bool,
        span: Span,
    },
    Return {
        expression: Option<Rc<Ast<'ctx>>>,
        kind: ThrushType,
        span: Span,
    },

    // Constants
    Const {
        name: &'ctx str,
        kind: ThrushType,
        value: Rc<Ast<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },

    // Locals variables
    Local {
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: ThrushType,
        value: Rc<Ast<'ctx>>,
        attributes: ThrushAttributes<'ctx>,
        undefined: bool,
        is_mutable: bool,
        span: Span,
    },

    // Reference
    Reference {
        name: &'ctx str,
        kind: ThrushType,
        span: Span,
        identificator: ReferenceIdentificator,
        is_mutable: bool,
        is_allocated: bool,
    },

    // Mutation
    Mut {
        source: AstEitherExpression<'ctx>,
        value: Rc<Ast<'ctx>>,
        kind: ThrushType,
        cast_type: ThrushType,
        span: Span,
    },

    // Low Level Instruction
    LLI {
        name: &'ctx str,
        kind: ThrushType,
        value: Rc<Ast<'ctx>>,
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
        address_to: AstEitherExpression<'ctx>,
        indexes: Vec<Ast<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    Write {
        write_to: AstEitherExpression<'ctx>,
        write_value: Rc<Ast<'ctx>>,
        write_type: ThrushType,
        span: Span,
    },

    Load {
        value: AstEitherExpression<'ctx>,
        kind: ThrushType,
        span: Span,
    },

    Deref {
        value: Rc<Ast<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    // Casts
    As {
        from: Rc<Ast<'ctx>>,
        cast: ThrushType,
        span: Span,
    },

    // Expressions
    Call {
        name: &'ctx str,
        args: Vec<Ast<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    AsmValue {
        assembler: String,
        constraints: String,
        args: Vec<Ast<'ctx>>,
        kind: ThrushType,
        attributes: ThrushAttributes<'ctx>,
        span: Span,
    },

    BinaryOp {
        left: Rc<Ast<'ctx>>,
        operator: TokenType,
        right: Rc<Ast<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    UnaryOp {
        operator: TokenType,
        kind: ThrushType,
        expression: Rc<Ast<'ctx>>,
        is_pre: bool,
        span: Span,
    },

    Group {
        expression: Rc<Ast<'ctx>>,
        kind: ThrushType,
        span: Span,
    },

    // Builtins
    SizeOf {
        sizeof: ThrushType,
        kind: ThrushType,
        span: Span,
    },

    Builtin {
        builtin: Builtin<'ctx>,
        kind: ThrushType,
        span: Span,
    },

    // Extra
    Pass {
        span: Span,
    },

    Null {
        span: Span,
    },
}
