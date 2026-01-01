use crate::core::compiler::backends::llvm::jit::JITConfiguration;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use std::env;
use std::path::PathBuf;

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use ahash::AHashSet as HashSet;

#[derive(Debug)]
pub struct LLVMJITCompiler<'ctx> {
    engine: ExecutionEngine<'ctx>,
    config: &'ctx JITConfiguration,
    modules: Vec<Module<'ctx>>,

    mapped_symbols: HashSet<Vec<u8>>,
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

            mapped_symbols: HashSet::with_capacity(100_000),
        }
    }
}

impl<'ctx> LLVMJITCompiler<'ctx> {
    pub fn compile_and_run(mut self) -> Result<i32, ()> {
        self.setup_all_modules();

        self.load_with_libc()?;
        self.load_with_external_libraries();

        let entrypoint_v: FunctionValue = self.get_entrypoint()?;

        let program_path: PathBuf = env::current_exe().unwrap_or_default();
        let start_path: &str = program_path.to_str().unwrap_or_default();

        let mut args: Vec<String> = vec![start_path.into()];
        args.extend(self.config.get_args().iter().cloned());

        self.engine.run_static_constructors();
        let result: i32 = unsafe { self.engine.run_function_as_main(entrypoint_v, &args) };
        self.engine.run_static_destructors();

        Ok(result)
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
    fn load_with_libc(&mut self) -> Result<(), ()> {
        let libc: libloading::Library =
            unsafe { libloading::Library::new(self.config.get_libc_path()) }.map_err(|e| {
                logging::print_error(
                    LoggingType::JITCompiler,
                    &format!("The C runtime couldn't be loaded: '{}'.", e),
                );
            })?;

        self.modules
            .iter()
            .flat_map(|module| module.get_functions())
            .for_each(|function| {
                if function.get_linkage() == Linkage::External
                    && function.get_last_basic_block().is_none()
                {
                    let name: &[u8] = function.get_name().to_bytes();

                    if !self.mapped_symbols.contains(name) {
                        if let Ok(addr) = unsafe { libc.get::<usize>(name) } {
                            self.engine.add_global_mapping(&function, *addr);
                            self.mapped_symbols.insert(name.to_vec());
                        }
                    }
                }
            });

        self.modules
            .iter()
            .flat_map(|module| module.get_globals())
            .for_each(|global| {
                if global.get_linkage() == Linkage::External && global.get_initializer().is_none() {
                    let name: &[u8] = global.get_name().to_bytes();

                    if !self.mapped_symbols.contains(name) {
                        if let Ok(addr) = unsafe { libc.get::<usize>(name) } {
                            self.engine.add_global_mapping(&global, *addr);
                            self.mapped_symbols.insert(name.to_vec());
                        }
                    }
                }
            });

        Ok(())
    }

    fn load_with_external_libraries(&mut self) {
        for library_path in self.config.get_libraries() {
            match unsafe { libloading::Library::new(library_path) } {
                Ok(lib) => {
                    self.modules
                        .iter()
                        .flat_map(|module| module.get_functions())
                        .for_each(|function| {
                            if function.get_linkage() == Linkage::External
                                && function.get_last_basic_block().is_none()
                            {
                                let name: &[u8] = function.get_name().to_bytes();

                                if !self.mapped_symbols.contains(name) {
                                    if let Ok(addr) = unsafe { lib.get::<usize>(name) } {
                                        self.engine.add_global_mapping(&function, *addr);
                                        self.mapped_symbols.insert(name.to_vec());
                                    }
                                }
                            }
                        });

                    self.modules
                        .iter()
                        .flat_map(|module| module.get_globals())
                        .for_each(|global| {
                            if global.get_linkage() == Linkage::External
                                && global.get_initializer().is_none()
                            {
                                let name: &[u8] = global.get_name().to_bytes();

                                if !self.mapped_symbols.contains(name) {
                                    if let Ok(addr) = unsafe { lib.get::<usize>(name) } {
                                        self.engine.add_global_mapping(&global, *addr);
                                        self.mapped_symbols.insert(name.to_vec());
                                    }
                                }
                            }
                        });
                }
                Err(e) => {
                    logging::print_warn(
                        LoggingType::Warning,
                        &format!(
                            "The dynamic library '{}' could not be loaded: '{}'.",
                            library_path.display(),
                            e
                        ),
                    );
                }
            }
        }
    }
}

impl<'ctx> LLVMJITCompiler<'ctx> {
    fn get_entrypoint(&self) -> Result<FunctionValue<'ctx>, ()> {
        let entrypoint_name: &[u8] = self.config.get_entry();

        self.modules
            .iter()
            .flat_map(|module| module.get_functions())
            .find(|function| function.get_name().to_bytes() == entrypoint_name)
            .ok_or_else(|| {
                logging::print_error(
                    LoggingType::Error,
                    "The program entrypoint couldn't be found.",
                );
            })
    }
}
