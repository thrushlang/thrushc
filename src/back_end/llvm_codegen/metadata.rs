use crate::back_end::llvm_codegen::codemodel::LLVMCodeModelExtensions;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::relocmodel::LLVMRelocModeExtensions;
use crate::back_end::llvm_codegen::targettriple::LLVMTargetTriple;

use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::options::CompilerOptions;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;
use crate::core::constants::COMPILER_ID;

use inkwell::debug_info;
use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::MetadataValue;

#[derive(Debug)]
pub struct LLVMMetadata<'a, 'ctx> {
    context: &'a LLVMCodeGenContext<'a, 'ctx>,
}

impl<'a, 'ctx> LLVMMetadata<'a, 'ctx> {
    #[inline]
    pub fn setup(context: &'a LLVMCodeGenContext<'a, 'ctx>) {
        let inner: LLVMMetadata<'a, 'ctx> = Self { context };

        inner.setup_metadata();
    }
}

impl<'a, 'ctx> LLVMMetadata<'a, 'ctx> {
    fn setup_metadata(&self) {
        self.setup_llvm_module_flags();
        self.setup_compiler_info();
        self.setup_build_id();
    }

    fn setup_llvm_module_flags(&self) {
        let options: &CompilerOptions = self.get_context().get_compiler_options();
        let llvm_backend: &LLVMBackend = options.get_llvm_backend_options();

        let lvl_max: BasicMetadataValueEnum = self
            .get_context()
            .get_llvm_context()
            .i32_type()
            .const_int(7, false)
            .into();

        let lvl_min: BasicMetadataValueEnum = self
            .get_context()
            .get_llvm_context()
            .i32_type()
            .const_int(8, false)
            .into();

        let lvl_error: BasicMetadataValueEnum = self
            .get_context()
            .get_llvm_context()
            .i32_type()
            .const_int(1, false)
            .into();

        let lvl_warn: BasicMetadataValueEnum = self
            .get_context()
            .get_llvm_context()
            .i32_type()
            .const_int(2, false)
            .into();

        {
            if llvm_backend.get_debug_config().is_debug_mode() {
                let dwarf_version: u64 = llvm_backend.get_debug_config().get_dwarf_version();
                let debug_info_version: u32 = debug_info::debug_metadata_version();

                let dwarf_v: MetadataValue =
                    self.get_context().get_llvm_context().metadata_node(&[
                        lvl_max,
                        self.get_context()
                            .get_llvm_context()
                            .metadata_string("Dwarf Version")
                            .into(),
                        self.get_context()
                            .get_llvm_context()
                            .i32_type()
                            .const_int(dwarf_version, false)
                            .into(),
                    ]);

                self.get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &dwarf_v)
                    .unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "'Dwarf Version' metadata failed to set up.",
                        );
                    });

                let debug_info_v: MetadataValue =
                    self.get_context().get_llvm_context().metadata_node(&[
                        lvl_warn,
                        self.get_context()
                            .get_llvm_context()
                            .metadata_string("Debug Info Version")
                            .into(),
                        self.get_context()
                            .get_llvm_context()
                            .i32_type()
                            .const_int(debug_info_version as u64, false)
                            .into(),
                    ]);

                self.get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &debug_info_v)
                    .unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "'Debug Info Version' metadata failed to set up.",
                        );
                    });
            }
        }

        {
            let repr: u64 = llvm_backend.get_reloc_mode().to_metadata_repr();

            let pic_level: MetadataValue = self.get_context().get_llvm_context().metadata_node(&[
                lvl_min,
                self.get_context()
                    .get_llvm_context()
                    .metadata_string("PIC Level")
                    .into(),
                self.get_context()
                    .get_llvm_context()
                    .i32_type()
                    .const_int(repr, false)
                    .into(),
            ]);

            self.get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &pic_level)
                .unwrap_or_else(|_| {
                    logging::print_warn(
                        LoggingType::Warning,
                        "'PIC Level' metadata failed to set up.",
                    );
                });
        }

        {
            let repr: u64 = llvm_backend.get_reloc_mode().to_metadata_repr();

            let pie_level: MetadataValue = self.get_context().get_llvm_context().metadata_node(&[
                lvl_max,
                self.get_context()
                    .get_llvm_context()
                    .metadata_string("PIE Level")
                    .into(),
                self.get_context()
                    .get_llvm_context()
                    .i32_type()
                    .const_int(repr, false)
                    .into(),
            ]);

            self.get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &pie_level)
                .unwrap_or_else(|_| {
                    logging::print_warn(
                        LoggingType::Warning,
                        "'PIE Level' metadata failed to set up.",
                    );
                });
        }

        {
            let repr: u64 = llvm_backend.get_code_model().to_metadata_repr();

            let code_level: MetadataValue = self.get_context().get_llvm_context().metadata_node(&[
                lvl_error,
                self.get_context()
                    .get_llvm_context()
                    .metadata_string("Code Model")
                    .into(),
                self.get_context()
                    .get_llvm_context()
                    .i32_type()
                    .const_int(repr, false)
                    .into(),
            ]);

            self.get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &code_level)
                .unwrap_or_else(|_| {
                    logging::print_warn(
                        LoggingType::Warning,
                        "'Code Model' metadata failed to set up.",
                    );
                });
        }

        #[cfg(target_vendor = "apple")]
        {
            let lvl_warning: BasicMetadataValueEnum = self
                .get_context()
                .get_llvm_context()
                .i32_type()
                .const_int(2, false)
                .into();

            if let Some(sdk_macos_version) = llvm_backend.get_target().get_macos_version() {
                let major: u64 = sdk_macos_version.0;
                let minor: u64 = sdk_macos_version.1;
                let patch: u64 = sdk_macos_version.2;

                let sdk_v: MetadataValue = self.get_context().get_llvm_context().metadata_node(&[
                    lvl_warning,
                    self.get_context()
                        .get_llvm_context()
                        .metadata_string("SDK Version")
                        .into(),
                    self.get_context()
                        .get_llvm_context()
                        .i32_type()
                        .const_array(&[
                            self.get_context()
                                .get_llvm_context()
                                .i32_type()
                                .const_int(major, false),
                            self.get_context()
                                .get_llvm_context()
                                .i32_type()
                                .const_int(minor, false),
                            self.get_context()
                                .get_llvm_context()
                                .i32_type()
                                .const_int(patch, false),
                        ])
                        .into(),
                ]);

                self.get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &sdk_v)
                    .unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "'MacOS SDK Version' metadata failed to set up.",
                        );
                    });
            }

            if let Some(sdk_ios_version) = llvm_backend.get_target().get_ios_version() {
                let major: u64 = sdk_ios_version.0;
                let minor: u64 = sdk_ios_version.1;
                let patch: u64 = sdk_ios_version.2;

                let sdk_v: MetadataValue = self.get_context().get_llvm_context().metadata_node(&[
                    lvl_warning,
                    self.get_context()
                        .get_llvm_context()
                        .metadata_string("SDK Version")
                        .into(),
                    self.get_context()
                        .get_llvm_context()
                        .i32_type()
                        .const_array(&[
                            self.get_context()
                                .get_llvm_context()
                                .i32_type()
                                .const_int(major, false),
                            self.get_context()
                                .get_llvm_context()
                                .i32_type()
                                .const_int(minor, false),
                            self.get_context()
                                .get_llvm_context()
                                .i32_type()
                                .const_int(patch, false),
                        ])
                        .into(),
                ]);

                self.get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &sdk_v)
                    .unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "'IOS SDK Version' metadata failed to set up.",
                        );
                    });
            }
        }

        {
            let triple: LLVMTargetTriple =
                LLVMTargetTriple::new(self.get_context().get_target_triple());

            let abi: &str = triple.get_abi();

            if abi != "unknown" {
                let metadata: MetadataValue =
                    self.get_context().get_llvm_context().metadata_node(&[
                        lvl_error,
                        self.get_context()
                            .get_llvm_context()
                            .metadata_string("target-abi")
                            .into(),
                        self.get_context()
                            .get_llvm_context()
                            .metadata_string(abi)
                            .into(),
                    ]);

                self.get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &metadata)
                    .unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "'Target ABI' metadata failed to set up.",
                        );
                    });
            }
        }

        {
            if llvm_backend.get_reloc_mode().is_no_pic()
                || llvm_backend.is_jit() && !llvm_backend.omit_direct_access_external_data()
            {
                let direct_access_external_data: MetadataValue =
                    self.get_context().get_llvm_context().metadata_node(&[
                        lvl_max,
                        self.get_context()
                            .get_llvm_context()
                            .metadata_string("direct-access-external-data")
                            .into(),
                        self.get_context()
                            .get_llvm_context()
                            .i32_type()
                            .const_int(1, false)
                            .into(),
                    ]);

                self.get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &direct_access_external_data)
                    .unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "'Direct Access External Data' metadata failed to set up.",
                        );
                    });
            }
        }

        {
            if let Some(target_triple_darwin_variant) =
                llvm_backend.get_target().get_triple_darwin_variant()
            {
                let code_level: MetadataValue =
                    self.get_context().get_llvm_context().metadata_node(&[
                        lvl_error,
                        self.get_context()
                            .get_llvm_context()
                            .metadata_string("darwin.target_variant.triple")
                            .into(),
                        self.get_context()
                            .get_llvm_context()
                            .metadata_string(
                                target_triple_darwin_variant
                                    .as_str()
                                    .to_string_lossy()
                                    .as_ref(),
                            )
                            .into(),
                    ]);

                self.get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &code_level)
                    .unwrap_or_else(|_| {
                        logging::print_warn(
                            LoggingType::Warning,
                            "'Darwin Target Triple' metadata failed to set up.",
                        );
                    });
            }
        }

        {
            if !llvm_backend.omit_rtlibusegot() {
                let triple: LLVMTargetTriple =
                    LLVMTargetTriple::new(self.get_context().get_target_triple());

                if triple.get_arch().contains("arm")
                    && llvm_backend.get_reloc_mode().is_pic()
                    && triple.has_posix_thread_model()
                {
                    let rt_lib_use_got: MetadataValue =
                        self.get_context().get_llvm_context().metadata_node(&[
                            lvl_error,
                            self.get_context()
                                .get_llvm_context()
                                .metadata_string("RtLibUseGOT")
                                .into(),
                            self.get_context()
                                .get_llvm_context()
                                .i32_type()
                                .const_int(1, false)
                                .into(),
                        ]);

                    self.get_context()
                        .get_llvm_module()
                        .add_global_metadata("llvm.module.flags", &rt_lib_use_got)
                        .unwrap_or_else(|_| {
                            logging::print_warn(
                                LoggingType::Warning,
                                "'RtLibUseGOT' metadata failed to set up.",
                            );
                        });
                }
            }
        }

        if !llvm_backend.get_optimization().is_high_opt() && !llvm_backend.omit_frame_pointer() {
            let frame_pointer: MetadataValue =
                self.get_context().get_llvm_context().metadata_node(&[
                    lvl_max,
                    self.get_context()
                        .get_llvm_context()
                        .metadata_string("frame-pointer")
                        .into(),
                    self.get_context()
                        .get_llvm_context()
                        .i32_type()
                        .const_int(2, false)
                        .into(),
                ]);

            self.get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &frame_pointer)
                .unwrap_or_else(|_| {
                    logging::print_warn(
                        LoggingType::Warning,
                        "'Frame Pointer' metadata failed to set up.",
                    );
                });
        }

        if !llvm_backend.omit_uwtable() {
            let uwtable: MetadataValue = self.get_context().get_llvm_context().metadata_node(&[
                lvl_max,
                self.get_context()
                    .get_llvm_context()
                    .metadata_string("uwtable")
                    .into(),
                self.get_context()
                    .get_llvm_context()
                    .i32_type()
                    .const_int(2, false)
                    .into(),
            ]);

            self.get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &uwtable)
                .unwrap_or_else(|_| {
                    logging::print_warn(
                        LoggingType::Warning,
                        "'Unwind Table' metadata failed to set up.",
                    );
                });
        }
    }

    fn setup_compiler_info(&self) {
        let version: MetadataValue = self
            .get_context()
            .get_llvm_context()
            .metadata_string(COMPILER_ID);

        let node: MetadataValue = self
            .get_context()
            .get_llvm_context()
            .metadata_node(&[version.into()]);

        let _ = self
            .get_context()
            .get_llvm_module()
            .add_global_metadata("llvm.ident", &node);
    }

    fn setup_build_id(&self) {
        let options: &CompilerOptions = self.get_context().get_compiler_options();
        let id: String = options.get_build_id().to_string();

        let build_id: MetadataValue = self.get_context().get_llvm_context().metadata_string(&id);

        let llvm_major: u32 = inkwell::support::get_llvm_version().0;
        let llvm_minor: u32 = inkwell::support::get_llvm_version().1;
        let llvm_patch: u32 = inkwell::support::get_llvm_version().2;

        let llvm_v: MetadataValue =
            self.get_context()
                .get_llvm_context()
                .metadata_string(&format!(
                    "LLVM {}.{}.{}",
                    llvm_major, llvm_minor, llvm_patch
                ));

        let node: MetadataValue = self
            .get_context()
            .get_llvm_context()
            .metadata_node(&[build_id.into(), llvm_v.into()]);

        self.get_context()
            .get_llvm_module()
            .add_global_metadata("build", &node)
            .unwrap_or_else(|_| {
                logging::print_warn(
                    LoggingType::Warning,
                    "'Build Compiler Info' metadata failed to set up.",
                );
            });
    }
}

impl<'a, 'ctx> LLVMMetadata<'a, 'ctx> {
    #[inline]
    fn get_context(&self) -> &'a LLVMCodeGenContext<'a, 'ctx> {
        self.context
    }
}
