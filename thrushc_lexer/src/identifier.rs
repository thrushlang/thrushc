use thrushc_errors::CompilationIssue;
use thrushc_token::tokentype::TokenType;

use crate::Lexer;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ATOMIC: HashMap<&'static [u8], TokenType> = {
        let mut atomic: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        atomic.insert(b"volatile", TokenType::Volatile);
        atomic.insert(b"lazythread", TokenType::LazyThread);

        atomic.insert(b"atomnone", TokenType::AtomNone);
        atomic.insert(b"atomfree", TokenType::AtomFree);
        atomic.insert(b"atomrelax", TokenType::AtomRelax);
        atomic.insert(b"atomgrab", TokenType::AtomGrab);
        atomic.insert(b"atomdrop", TokenType::AtomDrop);
        atomic.insert(b"atomsync", TokenType::AtomSync);
        atomic.insert(b"atomstrict", TokenType::AtomStrict);

        atomic.insert(b"threadinit", TokenType::ThreadInit);
        atomic.insert(b"threaddyn", TokenType::ThreadDynamic);
        atomic.insert(b"threadexec", TokenType::ThreadExec);
        atomic.insert(b"threadldyn", TokenType::ThreadLDynamic);

        atomic
    };
}

lazy_static! {
    pub static ref ATTRIBUTES: HashMap<&'static [u8], TokenType> = {
        let mut attributes: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        attributes.insert(b"@asmalignstack", TokenType::AsmAlignStack);
        attributes.insert(b"@asmsyntax", TokenType::AsmSyntax);
        attributes.insert(b"@asmthrow", TokenType::AsmThrow);
        attributes.insert(b"@asmeffects", TokenType::AsmSideEffects);

        attributes.insert(b"@optfuzzing", TokenType::OptFuzzing);
        attributes.insert(b"@nounwind", TokenType::NoUnwind);
        attributes.insert(b"@packed", TokenType::Packed);
        attributes.insert(b"@heap", TokenType::Heap);
        attributes.insert(b"@stack", TokenType::Stack);
        attributes.insert(b"@public", TokenType::Public);
        attributes.insert(b"@linkage", TokenType::Linkage);
        attributes.insert(b"@extern", TokenType::Extern);
        attributes.insert(b"@ignore", TokenType::Ignore);
        attributes.insert(b"@hot", TokenType::Hot);
        attributes.insert(b"@minsize", TokenType::MinSize);
        attributes.insert(b"@alwaysinline", TokenType::AlwaysInline);
        attributes.insert(b"@noinline", TokenType::NoInline);
        attributes.insert(b"@inline", TokenType::InlineHint);
        attributes.insert(b"@safestack", TokenType::SafeStack);
        attributes.insert(b"@weakstack", TokenType::WeakStack);
        attributes.insert(b"@strongstack", TokenType::StrongStack);
        attributes.insert(b"@precisefp", TokenType::PreciseFloats);
        attributes.insert(b"@convention", TokenType::Convention);
        attributes.insert(b"@constructor", TokenType::Constructor);
        attributes.insert(b"@destructor", TokenType::Destructor);

        attributes
    };
}

lazy_static! {
    pub static ref BUILTINS: HashMap<&'static [u8], TokenType> = {
        let mut builtins: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        builtins.insert(b"halloc", TokenType::Halloc);
        builtins.insert(b"size_of", TokenType::SizeOf);
        builtins.insert(b"memset", TokenType::MemSet);
        builtins.insert(b"memmove", TokenType::MemMove);
        builtins.insert(b"memcpy", TokenType::MemCpy);
        builtins.insert(b"align_of", TokenType::AlignOf);
        builtins.insert(b"abi_size_of", TokenType::AbiSizeOf);
        builtins.insert(b"bit_size_of", TokenType::BitSizeOf);
        builtins.insert(b"abi_align_of", TokenType::AbiAlignOf);

        builtins
    };
}

lazy_static! {
    pub static ref TYPES: HashMap<&'static [u8], TokenType> = {
        let mut types: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        types.insert(b"s8", TokenType::S8);
        types.insert(b"s16", TokenType::S16);
        types.insert(b"s32", TokenType::S32);
        types.insert(b"s64", TokenType::S64);
        types.insert(b"ssize", TokenType::Ssize);
        types.insert(b"u8", TokenType::U8);
        types.insert(b"u16", TokenType::U16);
        types.insert(b"u32", TokenType::U32);
        types.insert(b"u64", TokenType::U64);
        types.insert(b"u128", TokenType::U128);
        types.insert(b"usize", TokenType::Usize);
        types.insert(b"f32", TokenType::F32);
        types.insert(b"f64", TokenType::F64);
        types.insert(b"f128", TokenType::F128);
        types.insert(b"fx86_80", TokenType::FX8680);
        types.insert(b"fppc_128", TokenType::FPPC128);
        types.insert(b"bool", TokenType::Bool);
        types.insert(b"char", TokenType::Char);
        types.insert(b"ptr", TokenType::Ptr);
        types.insert(b"array", TokenType::Array);
        types.insert(b"void", TokenType::Void);
        types.insert(b"Fn", TokenType::FnRef);

        types
    };
}

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static [u8], TokenType> = {
        let mut keywords: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        keywords.insert(b"local", TokenType::Local);
        keywords.insert(b"fn", TokenType::Fn);
        keywords.insert(b"if", TokenType::If);
        keywords.insert(b"elif", TokenType::Elif);
        keywords.insert(b"else", TokenType::Else);
        keywords.insert(b"for", TokenType::For);
        keywords.insert(b"while", TokenType::While);
        keywords.insert(b"loop", TokenType::Loop);
        keywords.insert(b"true", TokenType::True);
        keywords.insert(b"false", TokenType::False);
        keywords.insert(b"or", TokenType::Or);
        keywords.insert(b"and", TokenType::And);
        keywords.insert(b"const", TokenType::Const);
        keywords.insert(b"struct", TokenType::Struct);
        keywords.insert(b"return", TokenType::Return);
        keywords.insert(b"break", TokenType::Break);
        keywords.insert(b"continue", TokenType::Continue);
        keywords.insert(b"pass", TokenType::Pass);
        keywords.insert(b"instr", TokenType::Instr);
        keywords.insert(b"mut", TokenType::Mut);
        keywords.insert(b"nullptr", TokenType::NullPtr);
        keywords.insert(b"as", TokenType::As);
        keywords.insert(b"asmfn", TokenType::AsmFn);
        keywords.insert(b"asm", TokenType::Asm);
        keywords.insert(b"global_asm", TokenType::GlobalAsm);
        keywords.insert(b"deref", TokenType::Deref);
        keywords.insert(b"type", TokenType::Type);
        keywords.insert(b"enum", TokenType::Enum);
        keywords.insert(b"alloc", TokenType::Alloc);
        keywords.insert(b"address", TokenType::Address);
        keywords.insert(b"addr", TokenType::Addr);
        keywords.insert(b"load", TokenType::Load);
        keywords.insert(b"write", TokenType::Write);
        keywords.insert(b"fixed", TokenType::Fixed);
        keywords.insert(b"ref", TokenType::DirectRef);
        keywords.insert(b"static", TokenType::Static);
        keywords.insert(b"indirect", TokenType::Indirect);
        keywords.insert(b"unreachable", TokenType::Unreachable);
        keywords.insert(b"intrinsic", TokenType::Intrinsic);
        keywords.insert(b"import", TokenType::Import);
        keywords.insert(b"importC", TokenType::ImportC);
        keywords.insert(b"new", TokenType::New);

        keywords
    };
}

pub fn lex(lexer: &mut Lexer) -> Result<(), CompilationIssue> {
    while lexer.is_identifier_boundary(lexer.peek()) {
        lexer.advance_only();
    }

    let bytes: Vec<u8> = lexer.lexeme_bytes();

    if let Some(keyword) = KEYWORDS.get(bytes.as_slice()) {
        lexer.make(*keyword);
    } else if let Some(atomic_stuff) = ATOMIC.get(bytes.as_slice()) {
        lexer.make(*atomic_stuff);
    } else if let Some(attribute) = ATTRIBUTES.get(bytes.as_slice()) {
        lexer.make(*attribute);
    } else if let Some(builtin) = BUILTINS.get(bytes.as_slice()) {
        lexer.make(*builtin);
    } else if let Some(r#type) = TYPES.get(bytes.as_slice()) {
        lexer.make(*r#type);
    } else {
        lexer.make(TokenType::Identifier);
    }

    Ok(())
}
