#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // --- Operators ---
    LParen,     // ' ( '
    RParen,     // ' ) '
    LBrace,     // ' { '
    RBrace,     // ' } '
    Comma,      // ' , '
    Dot,        // ' . '
    Minus,      // ' - '
    Plus,       // ' + '
    Slash,      // ' / '
    Star,       // ' * '
    Xor,        // ' ^ '
    Not,        // ' ~ '
    Bor,        // ' | '
    BAnd,       // ' & '
    Colon,      // ' : '
    SemiColon,  // ' ; '
    RBracket,   // ' ] '
    LBracket,   // ' [ '
    Arith,      // ' % ',
    Bang,       // ' ! '
    Range,      // ' .. '
    Pass,       // ' ... '
    ColonColon, // ' :: '
    BangEq,     // ' != '
    Eq,         // ' = '
    EqEq,       // ' == '
    Greater,    // ' > '
    GreaterEq,  // ' >= '
    Less,       // ' < '
    LessEq,     // ' <= '
    PlusPlus,   // ' ++ '
    MinusMinus, // ' -- '
    MinusEq,    // -=
    PlusEq,     // +=
    LShift,     // ' << '
    RShift,     // ' >> '
    Arrow,      // ->

    // --- Literals ---
    Identifier,
    Integer,
    Float,

    // --- Attributes ---
    Heap,
    Stack,
    Extern,
    Ignore,
    Public,
    MinSize,
    NoInline,
    AlwaysInline,
    InlineHint,
    Hot,
    SafeStack,
    WeakStack,
    StrongStack,
    PreciseFloats,
    Convention,
    NoUnwind,
    Packed,
    AsmAlignStack,
    AsmSyntax,
    AsmThrow,
    AsmSideEffects,
    OptFuzzing,

    // --- Special ---
    Unreachable,

    // --- Modificators ---
    Volatile,
    LazyThread,

    AtomNone,
    AtomFree,
    AtomRelax,
    AtomGrab,
    AtomDrop,
    AtomSync,
    AtomStrict,

    ThreadDynamic,
    ThreadExec,
    ThreadInit,

    // --- LLI ---
    Alloc,
    Address,
    Instr,
    Load,
    Write,

    // -- Indirect Call
    Indirect,

    // --- Keywords ---
    AsmFn,
    Asm,
    GlobalAsm,
    Defer,
    As,
    Static,
    New,
    Fixed,
    Import,
    SizeOf,
    Mut,
    Type,
    Enum,
    And,
    Struct,
    Else,
    Fn,
    For,
    Continue,
    Break,
    If,
    Elif,
    Or,
    Return,
    Local,
    Const,
    While,
    Loop,
    DirectRef,

    // --- Literals ---
    True,
    False,
    NullPtr,

    // -- Builtins --
    AlignOf,
    Halloc,
    MemCpy,
    MemMove,
    MemSet,

    // --- Types ---
    S8,
    S16,
    S32,
    S64,

    U8,
    U16,
    U32,
    U64,
    U128,

    F32,
    F64,
    FX8680,
    F128,
    FPPC128,

    Bool,
    Char,
    Str,
    Ptr,
    Void,
    Addr,
    Array,

    FnRef,

    Eof,
}
