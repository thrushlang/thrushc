#![allow(clippy::upper_case_acronyms)]

use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    values::FunctionValue,
};

use super::conventions::CallConvention;

#[derive(Debug, Clone)]
pub enum LLVMAttribute<'ctx> {
    FFI(&'ctx str),
    Convention(CallConvention),
    Public,
    Ignore,
    Hot,
    NoInline,
    InlineHint,
    MinSize,
    AlwaysInline,
    SafeStack,
    StrongStack,
    WeakStack,
    PreciseFloats,
}

impl LLVMAttribute<'_> {
    #[inline]
    pub const fn is_ffi_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::FFI(_))
    }

    #[inline]
    pub const fn is_ignore_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Ignore)
    }

    #[inline]
    pub const fn is_public_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Public)
    }
}

pub enum LLVMAttributeApplicant<'ctx> {
    Function(FunctionValue<'ctx>),
}

pub struct AttributeBuilder<'ctx> {
    context: &'ctx Context,
    attributes: &'ctx [LLVMAttribute<'ctx>],
    attribute_applicant: LLVMAttributeApplicant<'ctx>,
}

impl<'ctx> AttributeBuilder<'ctx> {
    pub fn new(
        context: &'ctx Context,
        attributes: &'ctx [LLVMAttribute<'ctx>],
        attribute_applicant: LLVMAttributeApplicant<'ctx>,
    ) -> Self {
        Self {
            context,
            attributes,
            attribute_applicant,
        }
    }

    pub fn add_function_attributes(&mut self, call_convention: &mut u32) {
        match self.attribute_applicant {
            LLVMAttributeApplicant::Function(function) => {
                self.attributes
                    .iter()
                    .for_each(|attribute| match attribute {
                        LLVMAttribute::AlwaysInline => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("alwaysinline"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::InlineHint => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("inlinehint"),
                                    1,
                                ),
                            );
                        }

                        LLVMAttribute::NoInline => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("noinline"),
                                    4,
                                ),
                            );
                        }

                        LLVMAttribute::Hot => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("hot"),
                                    2,
                                ),
                            );
                        }

                        LLVMAttribute::MinSize => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("optsize"),
                                    3,
                                ),
                            );
                        }

                        LLVMAttribute::SafeStack => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("safestack"),
                                    5,
                                ),
                            );
                        }

                        LLVMAttribute::WeakStack => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("ssp"),
                                    5,
                                ),
                            );
                        }

                        LLVMAttribute::StrongStack => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("sspstrong"),
                                    5,
                                ),
                            );
                        }

                        LLVMAttribute::PreciseFloats => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("strictfp"),
                                    5,
                                ),
                            );
                        }

                        LLVMAttribute::Convention(new_call_convention) => {
                            *call_convention = *new_call_convention as u32;
                        }

                        _ => (),
                    })
            }
        }
    }
}
