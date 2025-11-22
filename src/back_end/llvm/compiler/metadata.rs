use inkwell::support;
use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::MetadataValue;

use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;

use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::options::CompilerOptions;
use crate::core::constants::COMPILER_VERSION;

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

        if !llvm_backend.get_optimization().is_opt() && !llvm_backend.omit_frame_pointer() {
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
            .metadata_string(&format!("thrushc v{}", COMPILER_VERSION));

        let llvm_major: u32 = support::get_llvm_version().0;
        let llvm_minor: u32 = support::get_llvm_version().1;
        let llvm_patch: u32 = support::get_llvm_version().2;

        let llvm_version: MetadataValue =
            self.get_context()
                .get_llvm_context()
                .metadata_string(&format!(
                    "LLVM v{}.{}.{}",
                    llvm_major, llvm_minor, llvm_patch
                ));

        let node: MetadataValue = self
            .get_context()
            .get_llvm_context()
            .metadata_node(&[version.into(), llvm_version.into()]);

        let _ = self
            .get_context()
            .get_llvm_module()
            .add_global_metadata("compiler.info", &node);
    }
}

impl<'a, 'ctx> LLVMMetadata<'a, 'ctx> {
    #[inline]
    fn get_context(&self) -> &'a LLVMCodeGenContext<'a, 'ctx> {
        self.context
    }
}
