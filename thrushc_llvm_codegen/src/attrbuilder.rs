use inkwell::attributes::Attribute;
use inkwell::attributes::AttributeLoc;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::values::FunctionValue;
use inkwell::values::GlobalValue;
use thrushc_llvm_attributes::LLVMAttribute;
use thrushc_llvm_attributes::traits::LLVMAttributesExtensions;

use crate::context::LLVMCodeGenContext;

#[derive(Debug)]
pub enum LLVMAttributeApplicant<'ctx> {
    Function(FunctionValue<'ctx>),
    Global(GlobalValue<'ctx>),
}

#[derive(Debug)]
pub struct AttributeBuilder<'ctx> {
    attributes: Vec<LLVMAttribute<'ctx>>,
    attribute_applicant: LLVMAttributeApplicant<'ctx>,
}

impl<'ctx> AttributeBuilder<'ctx> {
    #[inline]
    pub fn new(
        attributes: Vec<LLVMAttribute<'ctx>>,
        attribute_applicant: LLVMAttributeApplicant<'ctx>,
    ) -> Self {
        Self {
            attributes,
            attribute_applicant,
        }
    }
}

impl<'ctx> AttributeBuilder<'ctx> {
    pub fn add_function_attributes(&self, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
        if let LLVMAttributeApplicant::Function(function) = self.attribute_applicant {
            let llvm_context: &Context = context.get_llvm_context();

            let is_public: bool = self.attributes.has_public_attribute();

            if !is_public {
                function.set_linkage(Linkage::LinkerPrivate);
            }

            self.attributes
                .iter()
                .for_each(|attribute| match attribute {
                    LLVMAttribute::Linkage(linkage) => {
                        function.set_linkage(*linkage);
                    }

                    LLVMAttribute::AlwaysInline => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("alwaysinline"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::InlineHint => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("inlinehint"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::NoInline => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("noinline"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::Hot => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context
                                .create_enum_attribute(Attribute::get_named_enum_kind_id("hot"), 0),
                        );
                    }

                    LLVMAttribute::MinSize => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("optsize"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::SafeStack => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("safestack"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::WeakStack => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context
                                .create_enum_attribute(Attribute::get_named_enum_kind_id("ssp"), 0),
                        );
                    }

                    LLVMAttribute::StrongStack => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("sspstrong"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::PreciseFloats => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("strictfp"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::NoUnwind => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("nounwind"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::OptFuzzing => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("optforfuzzing"),
                                0,
                            ),
                        );
                    }
                    LLVMAttribute::Pure => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("naked"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::Constructor => {
                        context.add_ctor(function.as_global_value().as_pointer_value());
                    }

                    LLVMAttribute::Destructor => {
                        context.add_dtor(function.as_global_value().as_pointer_value());
                    }

                    _ => (),
                })
        }
    }

    pub fn add_global_attributes(&self) {
        if let LLVMAttributeApplicant::Global(global) = self.attribute_applicant {
            self.attributes.iter().for_each(|attr| {
                if let LLVMAttribute::Linkage(linkage) = attr {
                    global.set_linkage(*linkage);
                }
            });
        }
    }
}
