use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CALL_CONVENTIONS: HashMap<&'static [u8], crate::back_end::llvm_codegen::conventions::CallConvention> = {
        let mut call_conventions: HashMap<
            &'static [u8],
            crate::back_end::llvm_codegen::conventions::CallConvention,
        > = HashMap::with_capacity(10);

        call_conventions.insert(
            b"C",
            crate::back_end::llvm_codegen::conventions::CallConvention::Standard,
        );
        call_conventions.insert(
            b"fast",
            crate::back_end::llvm_codegen::conventions::CallConvention::Fast,
        );
        call_conventions.insert(
            b"tail",
            crate::back_end::llvm_codegen::conventions::CallConvention::Tail,
        );
        call_conventions.insert(
            b"cold",
            crate::back_end::llvm_codegen::conventions::CallConvention::Cold,
        );
        call_conventions.insert(
            b"weakReg",
            crate::back_end::llvm_codegen::conventions::CallConvention::PreserveMost,
        );
        call_conventions.insert(
            b"strongReg",
            crate::back_end::llvm_codegen::conventions::CallConvention::PreserveAll,
        );
        call_conventions.insert(
            b"Swift",
            crate::back_end::llvm_codegen::conventions::CallConvention::Swift,
        );
        call_conventions.insert(
            b"Haskell",
            crate::back_end::llvm_codegen::conventions::CallConvention::GHC,
        );
        call_conventions.insert(
            b"Erlang",
            crate::back_end::llvm_codegen::conventions::CallConvention::HiPE,
        );

        call_conventions
    };
}

#[inline]
pub fn get_call_convention(
    name: &[u8],
) -> crate::back_end::llvm_codegen::conventions::CallConvention {
    CALL_CONVENTIONS
        .get(name)
        .copied()
        .unwrap_or(crate::back_end::llvm_codegen::conventions::CallConvention::Standard)
}
