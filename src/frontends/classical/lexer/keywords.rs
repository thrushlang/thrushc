use crate::{frontends::classical::lexer::tokentype::TokenType, lazy_static};

use ahash::AHashMap as HashMap;

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut keywords: HashMap<&'static str, TokenType> = HashMap::with_capacity(100);

        keywords.insert("local", TokenType::Local);
        keywords.insert("fn", TokenType::Fn);
        keywords.insert("if", TokenType::If);
        keywords.insert("elif", TokenType::Elif);
        keywords.insert("else", TokenType::Else);
        keywords.insert("for", TokenType::For);
        keywords.insert("while", TokenType::While);
        keywords.insert("loop", TokenType::Loop);
        keywords.insert("true", TokenType::True);
        keywords.insert("false", TokenType::False);
        keywords.insert("or", TokenType::Or);
        keywords.insert("and", TokenType::And);
        keywords.insert("const", TokenType::Const);
        keywords.insert("struct", TokenType::Struct);
        keywords.insert("return", TokenType::Return);
        keywords.insert("break", TokenType::Break);
        keywords.insert("continue", TokenType::Continue);
        keywords.insert("pass", TokenType::Pass);
        keywords.insert("instr", TokenType::Instr);
        keywords.insert("mut", TokenType::Mut);
        keywords.insert("nullptr", TokenType::NullPtr);
        keywords.insert("as", TokenType::As);
        keywords.insert("asmfn", TokenType::AsmFn);
        keywords.insert("asm", TokenType::Asm);
        keywords.insert("global_asm", TokenType::GlobalAsm);
        keywords.insert("deref", TokenType::Deref);
        keywords.insert("type", TokenType::Type);
        keywords.insert("enum", TokenType::Enum);
        keywords.insert("alloc", TokenType::Alloc);
        keywords.insert("address", TokenType::Address);
        keywords.insert("addr", TokenType::Addr);
        keywords.insert("load", TokenType::Load);
        keywords.insert("write", TokenType::Write);
        keywords.insert("fixed", TokenType::Fixed);
        keywords.insert("static", TokenType::Static);

        keywords.insert("volatile", TokenType::Volatile);
        keywords.insert("lazythread", TokenType::LazyThread);

        keywords.insert("atomnone", TokenType::AtomNone);
        keywords.insert("atomfree", TokenType::AtomFree);
        keywords.insert("atomrelax", TokenType::AtomRelax);
        keywords.insert("atomgrab", TokenType::AtomGrab);
        keywords.insert("atomdrop", TokenType::AtomDrop);
        keywords.insert("atomsync", TokenType::AtomSync);
        keywords.insert("atomstrict", TokenType::AtomStrict);

        keywords.insert("threadinit", TokenType::ThreadInit);
        keywords.insert("threaddyn", TokenType::ThreadDynamic);
        keywords.insert("threadexec", TokenType::ThreadExec);

        keywords.insert("unreachable", TokenType::Unreachable);

        keywords.insert("halloc", TokenType::Halloc);
        keywords.insert("sizeof", TokenType::SizeOf);
        keywords.insert("memset", TokenType::MemSet);
        keywords.insert("memmove", TokenType::MemMove);
        keywords.insert("memcpy", TokenType::MemCpy);
        keywords.insert("alignof", TokenType::AlignOf);

        keywords.insert("import", TokenType::Import);

        keywords.insert("@asmalingstack", TokenType::AsmAlignStack);
        keywords.insert("@asmsyntax", TokenType::AsmSyntax);
        keywords.insert("@asmthrow", TokenType::AsmThrow);
        keywords.insert("@asmeffects", TokenType::AsmSideEffects);

        keywords.insert("@optfuzzing", TokenType::OptFuzzing);
        keywords.insert("@nounwind", TokenType::NoUnwind);
        keywords.insert("@packed", TokenType::Packed);
        keywords.insert("@heap", TokenType::Heap);
        keywords.insert("@stack", TokenType::Stack);
        keywords.insert("@public", TokenType::Public);
        keywords.insert("@extern", TokenType::Extern);
        keywords.insert("@ignore", TokenType::Ignore);
        keywords.insert("@hot", TokenType::Hot);
        keywords.insert("@minsize", TokenType::MinSize);
        keywords.insert("@alwaysinline", TokenType::AlwaysInline);
        keywords.insert("@noinline", TokenType::NoInline);
        keywords.insert("@inline", TokenType::InlineHint);
        keywords.insert("@safestack", TokenType::SafeStack);
        keywords.insert("@weakstack", TokenType::WeakStack);
        keywords.insert("@strongstack", TokenType::StrongStack);
        keywords.insert("@precisefp", TokenType::PreciseFloats);
        keywords.insert("@convention", TokenType::Convention);

        keywords.insert("new", TokenType::New);

        keywords.insert("s8", TokenType::S8);
        keywords.insert("s16", TokenType::S16);
        keywords.insert("s32", TokenType::S32);
        keywords.insert("s64", TokenType::S64);
        keywords.insert("u8", TokenType::U8);
        keywords.insert("u16", TokenType::U16);
        keywords.insert("u32", TokenType::U32);
        keywords.insert("u64", TokenType::U64);
        keywords.insert("f32", TokenType::F32);
        keywords.insert("f64", TokenType::F64);
        keywords.insert("bool", TokenType::Bool);
        keywords.insert("char", TokenType::Char);
        keywords.insert("ptr", TokenType::Ptr);
        keywords.insert("str", TokenType::Str);
        keywords.insert("array", TokenType::Array);
        keywords.insert("void", TokenType::Void);

        keywords
    };
}
