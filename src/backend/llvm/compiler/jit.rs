use std::env;
use std::path::PathBuf;

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::values::GlobalValue;

use crate::backend::llvm::types::LLVMJITCompilerFunctions;
use crate::backend::llvm::types::LLVMJITCompilerGlobals;
use crate::core::compiler::backends::llvm::jit::JITConfiguration;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

#[derive(Debug)]
pub struct LLVMJITCompiler<'ctx> {
    engine: ExecutionEngine<'ctx>,
    config: &'ctx JITConfiguration,
    modules: Vec<Module<'ctx>>,
}

impl<'ctx> LLVMJITCompiler<'ctx> {
    #[inline]
    pub fn new(
        engine: ExecutionEngine<'ctx>,
        config: &'ctx JITConfiguration,
        modules: Vec<Module<'ctx>>,
    ) -> Self {
        Self {
            engine,
            modules,
            config,
        }
    }
}

impl<'ctx> LLVMJITCompiler<'ctx> {
    pub fn compile_and_run(&self) -> Result<i32, ()> {
        self.setup_all_modules();

        let functions: LLVMJITCompilerFunctions<'ctx> = self.get_external_functions();
        let globals: LLVMJITCompilerGlobals<'ctx> = self.get_external_globals();

        self.load_with_libc(&functions, &globals)?;
        self.load_with_external_libraries(&functions, &globals);

        let entrypoint: FunctionValue = self.get_entrypoint()?;

        let program_path: PathBuf = env::current_exe().unwrap_or_default();
        let start_path: &str = program_path.to_str().unwrap_or_default();

        let program_args: &[String] = self.get_config().get_args();

        let mut args: Vec<String> = vec![start_path.into()];
        args.extend(program_args.iter().cloned());

        let jit_result: i32 = unsafe { self.engine.run_function_as_main(entrypoint, &args) };

        Ok(jit_result)
    }
}

impl LLVMJITCompiler<'_> {
    #[inline]
    fn get_config(&self) -> &JITConfiguration {
        self.config
    }
}

impl LLVMJITCompiler<'_> {
    fn setup_all_modules(&self) {
        self.modules.iter().for_each(|module| {
            let _ = self.engine.add_module(module);
        });
    }
}

impl<'ctx> LLVMJITCompiler<'ctx> {
    fn get_external_functions(&self) -> LLVMJITCompilerFunctions<'ctx> {
        let mut symbols: LLVMJITCompilerFunctions = Vec::with_capacity(100_000);

        self.modules.iter().for_each(|module| {
            module.get_functions().for_each(|function| {
                if function.get_linkage() == Linkage::External
                    && function.get_last_basic_block().is_none()
                {
                    symbols.push((function, function.get_name().to_string_lossy().to_string()));
                }
            });
        });

        symbols
    }

    fn get_external_globals(&self) -> LLVMJITCompilerGlobals<'ctx> {
        let mut symbols: LLVMJITCompilerGlobals = Vec::with_capacity(100_000);

        self.modules.iter().for_each(|module| {
            module.get_globals().for_each(|global| {
                if global.get_linkage() == Linkage::External && global.get_initializer().is_none() {
                    symbols.push((global, global.get_name().to_string_lossy().to_string()));
                }
            });
        });

        symbols
    }
}

impl<'ctx> LLVMJITCompiler<'ctx> {
    fn load_with_libc(
        &self,
        functions: &LLVMJITCompilerFunctions,
        globals: &LLVMJITCompilerGlobals,
    ) -> Result<(), ()> {
        let libc_path: &PathBuf = self.get_config().get_libc_path();

        let libc: Result<libloading::Library, libloading::Error> =
            unsafe { libloading::Library::new(libc_path) };

        if let Ok(libc) = libc {
            functions.iter().for_each(|symbol| {
                let symbol_name: &[u8] = symbol.1.as_bytes();
                let symbol_addr: Result<libloading::Symbol<'_, usize>, libloading::Error> =
                    unsafe { libc.get::<usize>(symbol_name) };

                let value: FunctionValue = symbol.0;

                if let Ok(symbol_addr) = symbol_addr {
                    self.engine.add_global_mapping(&value, *symbol_addr);
                }
            });

            globals.iter().for_each(|symbol| {
                let symbol_name: &[u8] = symbol.1.as_bytes();
                let symbol_addr: Result<libloading::Symbol<'_, usize>, libloading::Error> =
                    unsafe { libc.get::<usize>(symbol_name) };

                let value: GlobalValue = symbol.0;

                if let Ok(symbol_addr) = symbol_addr {
                    self.engine.add_global_mapping(&value, *symbol_addr);
                }
            });

            return Ok(());
        }

        logging::print_error(
            LoggingType::JITCompiler,
            &format!("The C runtime couldn't be loaded: '{}'.", libc.unwrap_err()),
        );

        Err(())
    }

    fn load_with_external_libraries(
        &self,
        functions: &LLVMJITCompilerFunctions,
        globals: &LLVMJITCompilerGlobals,
    ) {
        self.get_config()
            .get_libraries()
            .iter()
            .for_each(|library_path| {
                let library_path: &PathBuf = library_path;

                let library: Result<libloading::Library, libloading::Error> =
                    unsafe { libloading::Library::new(library_path) };

                if let Ok(ref lib) = library {
                    functions.iter().for_each(|symbol| {
                        let symbol_name: &[u8] = symbol.1.as_bytes();
                        let symbol_addr: Result<libloading::Symbol<'_, usize>, libloading::Error> =
                            unsafe { lib.get::<usize>(symbol_name) };

                        let value: FunctionValue = symbol.0;

                        if let Ok(symbol_addr) = symbol_addr {
                            self.engine.add_global_mapping(&value, *symbol_addr);
                        }
                    });

                    globals.iter().for_each(|symbol| {
                        let symbol_name: &[u8] = symbol.1.as_bytes();
                        let symbol_addr: Result<libloading::Symbol<'_, usize>, libloading::Error> =
                            unsafe { lib.get::<usize>(symbol_name) };

                        let value: GlobalValue = symbol.0;

                        if let Ok(symbol_addr) = symbol_addr {
                            self.engine.add_global_mapping(&value, *symbol_addr);
                        }
                    });
                }

                logging::print_warn(
                    LoggingType::Warning,
                    &format!(
                        "The dynamic library '{}' could not be loaded: '{}'.",
                        library_path.display(),
                        library.unwrap_err()
                    ),
                );
            });
    }
}

impl<'ctx> LLVMJITCompiler<'ctx> {
    fn get_entrypoint(&self) -> Result<FunctionValue<'ctx>, ()> {
        for module in &self.modules {
            for function in module.get_functions() {
                if function.get_name().to_string_lossy() == "main" {
                    return Ok(function);
                }
            }
        }

        logging::print_error(
            LoggingType::JITCompiler,
            "The program entrypoint 'main' couldn't be found.",
        );

        Err(())
    }
}
