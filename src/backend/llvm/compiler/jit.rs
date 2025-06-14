#![allow(clippy::upper_case_acronyms)]

use std::process;

use inkwell::{
    builder::Builder, context::Context, execution_engine::ExecutionEngine, module::Module,
};

use libloading::Library;

use crate::core::console::logging::{self, LoggingType};

#[derive(Debug)]
pub struct LLVMJIT<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    engine: &'ctx ExecutionEngine<'ctx>,
    libc: Library,
}

impl<'a, 'ctx> LLVMJIT<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        builder: &'ctx Builder<'ctx>,
        engine: &'ctx ExecutionEngine<'ctx>,
    ) -> Self {
        Self {
            module,
            context,
            builder,
            engine,
            libc: LLVMJIT::get_libc(),
        }
    }

    pub fn get_libc() -> Library {
        let libc: Result<Library, libloading::Error> = if cfg!(target_os = "linux") {
            unsafe { Library::new("libc.so") }
        } else {
            unsafe { Library::new("msvcrt.dll") }
        };

        if let Ok(library) = libc {
            return library;
        }

        logging::log(
            LoggingType::Error,
            "Unable to locate C Standard Library Interface for the LLVM Just In Time Compiler.",
        );

        process::exit(1);
    }
}
