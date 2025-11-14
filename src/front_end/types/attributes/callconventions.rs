use crate::back_end::llvm::compiler::conventions::CallConvention;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CALL_CONVENTIONS: HashMap<&'static [u8], CallConvention> = {
        let mut call_conventions: HashMap<&'static [u8], CallConvention> =
            HashMap::with_capacity(10);

        call_conventions.insert(b"C", CallConvention::Standard);
        call_conventions.insert(b"fast", CallConvention::Fast);
        call_conventions.insert(b"tail", CallConvention::Tail);
        call_conventions.insert(b"cold", CallConvention::Cold);
        call_conventions.insert(b"weakReg", CallConvention::PreserveMost);
        call_conventions.insert(b"strongReg", CallConvention::PreserveAll);
        call_conventions.insert(b"Swift", CallConvention::Swift);
        call_conventions.insert(b"Haskell", CallConvention::GHC);
        call_conventions.insert(b"Erlang", CallConvention::HiPE);

        call_conventions
    };
}

#[inline]
pub fn get_call_convention(name: &[u8]) -> CallConvention {
    CALL_CONVENTIONS
        .get(name)
        .copied()
        .unwrap_or(CallConvention::Standard)
}
