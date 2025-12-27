use crate::front_end::lexer::tokentype::TokenType;

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
