use thrushc_attributes::ThrushAttributes;
use thrushc_modificators::Modificators;
use thrushc_span::Span;
use thrushc_token::tokentype::TokenType;
use thrushc_typesystem::Type;

use crate::{
    builitins::ThrushBuiltin,
    data::{ConstructorData, EnumData, StructureData},
    metadata::{
        CastingMetadata, ConstantMetadata, DereferenceMetadata, FunctionParameterMetadata,
        IndexMetadata, LocalMetadata, PropertyMetadata, ReferenceMetadata, StaticMetadata,
    },
};

pub mod builitins;
pub mod data;
mod getters;
mod impls;
pub mod metadata;
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
        kind: Type,
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
        data: StructureData<'ctx>,
        kind: Type,
        span: Span,
        attributes: ThrushAttributes,
    },

    Constructor {
        name: &'ctx str,
        data: ConstructorData<'ctx>,
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
    If {
        condition: std::boxed::Box<Ast<'ctx>>,
        block: std::boxed::Box<Ast<'ctx>>,
        elseif: std::vec::Vec<Ast<'ctx>>,
        anyway: Option<std::boxed::Box<Ast<'ctx>>>,
        kind: Type,
        span: Span,
    },
    Elif {
        condition: std::boxed::Box<Ast<'ctx>>,
        block: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },
    Else {
        block: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Loops
    For {
        local: std::boxed::Box<Ast<'ctx>>,
        condition: std::boxed::Box<Ast<'ctx>>,
        actions: std::boxed::Box<Ast<'ctx>>,
        block: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },
    While {
        condition: std::boxed::Box<Ast<'ctx>>,
        block: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },
    Loop {
        block: std::boxed::Box<Ast<'ctx>>,
        kind: Type,
        span: Span,
    },

    // Loop control flow
    Continue {
        kind: Type,
        span: Span,
    },
    Break {
        kind: Type,
        span: Span,
    },
    ContinueAll {
        kind: Type,
        span: Span,
    },
    BreakAll {
        kind: Type,
        span: Span,
    },

    // Code block
    Block {
        nodes: std::vec::Vec<Ast<'ctx>>,
        kind: Type,
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
        data: EnumData<'ctx>,
        attributes: ThrushAttributes,
        kind: Type,
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
        modificators: Modificators,
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
        modificators: Modificators,
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
        modificators: Modificators,
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
        modificators: Modificators,
        metadata: DereferenceMetadata,
        span: Span,
    },

    // Casts
    As {
        from: std::boxed::Box<Ast<'ctx>>,
        cast: Type,
        metadata: CastingMetadata,
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
        kind: Type,
    },
    // C Import
    ImportC {
        span: Span,
        kind: Type,
    },

    // Unreachable
    Unreachable {
        span: Span,
        kind: Type,
    },

    // Invalid
    Invalid {
        kind: Type,
        span: Span,
    },
}

impl<'ctx> Ast<'ctx> {
    #[inline]
    pub fn new_float(kind: Type, value: f64, signed: bool, span: Span) -> Ast<'ctx> {
        Ast::Float {
            kind,
            value,
            signed,
            span,
        }
    }

    #[inline]
    pub fn new_integer(kind: Type, value: u64, signed: bool, span: Span) -> Ast<'ctx> {
        Ast::Integer {
            kind,
            value,
            signed,
            span,
        }
    }

    #[inline]
    pub fn new_boolean(kind: Type, value: u64, span: Span) -> Ast<'ctx> {
        Ast::Boolean { kind, value, span }
    }

    #[inline]
    pub fn new_char(kind: Type, byte: u64, span: Span) -> Ast<'ctx> {
        Ast::Char { kind, byte, span }
    }

    #[inline]
    pub fn new_str(bytes: Vec<u8>, kind: Type, span: Span) -> Ast<'ctx> {
        Ast::Str { bytes, kind, span }
    }

    #[inline]
    pub fn new_nullptr(span: Span) -> Ast<'ctx> {
        Ast::NullPtr {
            span,
            kind: Type::Ptr(None, span),
        }
    }

    #[inline]
    pub fn invalid_ast(span: Span) -> Ast<'ctx> {
        Ast::Invalid {
            kind: Type::Void(span),
            span,
        }
    }
}
