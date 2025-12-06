use crate::back_end::llvm_codegen::codemodel::LLVMCodeModelExtensions;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::reloc::LLVMRelocModeExtensions;

use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::options::CompilerOptions;
use crate::core::constants::COMPILER_VERSION;

use inkwell::support;
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

        let lvl_warning: BasicMetadataValueEnum = self
            .get_context()
            .get_llvm_context()
            .i32_type()
            .const_int(2, false)
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

        {
            let llvm_major: u32 = support::get_llvm_version().0;
            let llvm_minor: u32 = support::get_llvm_version().1;
            let llvm_patch: u32 = support::get_llvm_version().2;

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
                            .const_int(llvm_major.into(), false),
                        self.get_context()
                            .get_llvm_context()
                            .i32_type()
                            .const_int(llvm_minor.into(), false),
                        self.get_context()
                            .get_llvm_context()
                            .i32_type()
                            .const_int(llvm_patch.into(), false),
                    ])
                    .into(),
            ]);

            let _ = self
                .get_context()
                .get_llvm_module()
                .add_global_metadata("llvm.module.flags", &sdk_v);
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
            .metadata_string(&format!("thrushc version {}", COMPILER_VERSION));

        let node: MetadataValue = self
            .get_context()
            .get_llvm_context()
            .metadata_node(&[version.into()]);

        let _ = self
            .get_context()
            .get_llvm_module()
            .add_global_metadata("llvm.ident", &node);
    }
}

impl<'a, 'ctx> LLVMMetadata<'a, 'ctx> {
    #[inline]
    fn get_context(&self) -> &'a LLVMCodeGenContext<'a, 'ctx> {
        self.context
    }
}
