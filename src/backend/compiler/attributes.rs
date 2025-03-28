#![allow(clippy::upper_case_acronyms)]

use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    values::FunctionValue,
};

use super::conventions::CallConvention;

#[derive(Debug, Clone)]
pub enum CompilerAttribute<'ctx> {
    FFI(&'ctx str),
    Convention(CallConvention),
    Public(bool),
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

impl CompilerAttribute<'_> {
    #[inline(always)]
    pub const fn is_ffi_attribute(&self) -> bool {
        matches!(self, CompilerAttribute::FFI(_))
    }

    #[inline(always)]
    pub const fn is_ignore_attribute(&self) -> bool {
        matches!(self, CompilerAttribute::Ignore)
    }
}

pub enum CompilerAttributeApplicant<'ctx> {
    Function(FunctionValue<'ctx>),
}

pub struct AttributeBuilder<'ctx> {
    context: &'ctx Context,
    attributes: &'ctx [CompilerAttribute<'ctx>],
    attribute_applicant: CompilerAttributeApplicant<'ctx>,
}

impl<'ctx> AttributeBuilder<'ctx> {
    pub fn new(
        context: &'ctx Context,
        attributes: &'ctx [CompilerAttribute<'ctx>],
        attribute_applicant: CompilerAttributeApplicant<'ctx>,
    ) -> Self {
        Self {
            context,
            attributes,
            attribute_applicant,
        }
    }

    pub fn add_attributes(&mut self, call_convention: &mut u32) {
        match self.attribute_applicant {
            CompilerAttributeApplicant::Function(function) => {
                self.attributes
                    .iter()
                    .for_each(|attribute| match attribute {
                        CompilerAttribute::AlwaysInline => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("alwaysinline"),
                                    0,
                                ),
                            );
                        }

                        CompilerAttribute::InlineHint => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("inlinehint"),
                                    1,
                                ),
                            );
                        }

                        CompilerAttribute::NoInline => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("noinline"),
                                    4,
                                ),
                            );
                        }

                        CompilerAttribute::Hot => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("hot"),
                                    2,
                                ),
                            );
                        }

                        CompilerAttribute::MinSize => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("optsize"),
                                    3,
                                ),
                            );
                        }

                        CompilerAttribute::SafeStack => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("safestack"),
                                    5,
                                ),
                            );
                        }

                        CompilerAttribute::WeakStack => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("ssp"),
                                    5,
                                ),
                            );
                        }

                        CompilerAttribute::StrongStack => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("sspstrong"),
                                    5,
                                ),
                            );
                        }

                        CompilerAttribute::PreciseFloats => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("strictfp"),
                                    5,
                                ),
                            );
                        }

                        CompilerAttribute::Convention(new_call_convention) => {
                            *call_convention = *new_call_convention as u32;
                        }

                        _ => (),
                    })
            }
        }
    }
}
