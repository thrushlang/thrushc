use crate::back_end::llvm_codegen::codemodel::LLVMCodeModelExtensions;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::relocmodel::LLVMRelocModeExtensions;
use crate::back_end::llvm_codegen::targettriple::LLVMTargetTriple;

use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::options::CompilerOptions;
use crate::core::constants::COMPILER_ID;

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

            let _ = self
                .get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &pic_level);
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

            let _ = self
                .get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &pie_level);
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

            let _ = self
                .get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &code_level);
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

                let _ = self
                    .get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &sdk_v);
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

                let _ = self
                    .get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &sdk_v);
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

                let _ = self
                    .get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &metadata);
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

                let _ = self
                    .get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &direct_access_external_data);
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

                let _ = self
                    .get_context()
                    .get_llvm_module()
                    .add_global_metadata("llvm.module.flags", &code_level);
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

                    let _ = self
                        .get_context()
                        .get_llvm_module()
                        .add_global_metadata("llvm.module.flags", &rt_lib_use_got);
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

            let _ = self
                .get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &frame_pointer);
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

            let _ = self
                .get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &uwtable);
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

        let _ = self
            .get_context()
            .get_llvm_module()
            .add_global_metadata("build", &node);
    }
}

impl<'a, 'ctx> LLVMMetadata<'a, 'ctx> {
    #[inline]
    fn get_context(&self) -> &'a LLVMCodeGenContext<'a, 'ctx> {
        self.context
    }
}
