use crate::back_end::llvm_codegen::attributes::LLVMAttribute;

use inkwell::attributes::Attribute;
use inkwell::attributes::AttributeLoc;
use inkwell::context::Context;
use inkwell::values::FunctionValue;
use inkwell::values::GlobalValue;

#[derive(Debug)]
pub enum LLVMAttributeApplicant<'ctx> {
    Function(FunctionValue<'ctx>),
    Global(GlobalValue<'ctx>),
}

#[derive(Debug)]
pub struct AttributeBuilder<'ctx> {
    llvm_context: &'ctx Context,
    attributes: &'ctx [LLVMAttribute<'ctx>],
    attribute_applicant: LLVMAttributeApplicant<'ctx>,
}

impl<'ctx> AttributeBuilder<'ctx> {
    #[inline]
    pub fn new(
        llvm_context: &'ctx Context,
        attributes: &'ctx [LLVMAttribute<'ctx>],
        attribute_applicant: LLVMAttributeApplicant<'ctx>,
    ) -> Self {
        Self {
            llvm_context,
            attributes,
            attribute_applicant,
        }
    }
}

impl<'ctx> AttributeBuilder<'ctx> {
    pub fn add_function_attributes(&self) {
        if let LLVMAttributeApplicant::Function(function) = self.attribute_applicant {
            self.attributes
                .iter()
                .for_each(|attribute| match attribute {
                    LLVMAttribute::Linkage(linkage) => {
                        function.set_linkage(*linkage);
                    }

                    LLVMAttribute::AlwaysInline => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("alwaysinline"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::InlineHint => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self::create_inline_hint_attribute(self.llvm_context),
                        );
                    }

                    LLVMAttribute::NoInline => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("noinline"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::Hot => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context
                                .create_enum_attribute(Attribute::get_named_enum_kind_id("hot"), 0),
                        );
                    }

                    LLVMAttribute::MinSize => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("optsize"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::SafeStack => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("safestack"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::WeakStack => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context
                                .create_enum_attribute(Attribute::get_named_enum_kind_id("ssp"), 0),
                        );
                    }

                    LLVMAttribute::StrongStack => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("sspstrong"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::PreciseFloats => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("strictfp"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::NoUnwind => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("nounwind"),
                                0,
                            ),
                        );
                    }

                    LLVMAttribute::OptFuzzing => {
                        function.add_attribute(
                            AttributeLoc::Function,
                            self.llvm_context.create_enum_attribute(
                                Attribute::get_named_enum_kind_id("optforfuzzing"),
                                0,
                            ),
                        );
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

#[inline]
pub fn create_inline_hint_attribute(llvm_context: &Context) -> Attribute {
    llvm_context.create_enum_attribute(Attribute::get_named_enum_kind_id("inlinehint"), 0)
}
