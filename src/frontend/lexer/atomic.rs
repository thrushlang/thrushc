use crate::{frontend::lexer::tokentype::TokenType, lazy_static};

use ahash::AHashMap as HashMap;

lazy_static! {
    pub static ref ATOMIC: HashMap<&'static str, TokenType> = {
        let mut atomic: HashMap<&'static str, TokenType> = HashMap::with_capacity(100);

        atomic.insert("volatile", TokenType::Volatile);
        atomic.insert("lazythread", TokenType::LazyThread);

        atomic.insert("atomnone", TokenType::AtomNone);
        atomic.insert("atomfree", TokenType::AtomFree);
        atomic.insert("atomrelax", TokenType::AtomRelax);
        atomic.insert("atomgrab", TokenType::AtomGrab);
        atomic.insert("atomdrop", TokenType::AtomDrop);
        atomic.insert("atomsync", TokenType::AtomSync);
        atomic.insert("atomstrict", TokenType::AtomStrict);

        atomic.insert("threadinit", TokenType::ThreadInit);
        atomic.insert("threaddyn", TokenType::ThreadDynamic);
        atomic.insert("threadexec", TokenType::ThreadExec);

        atomic
    };
}
