#![allow(clippy::upper_case_acronyms)]

use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    values::FunctionValue,
};

use crate::frontend::lexer::span::Span;

use super::conventions::CallConvention;

#[derive(Debug, Clone, Copy)]
pub enum LLVMAttribute<'ctx> {
    Extern(&'ctx str, Span),
    Convention(CallConvention, Span),
    Public(Span),
    Ignore(Span),
    Hot(Span),
    NoInline(Span),
    InlineHint(Span),
    MinSize(Span),
    AlwaysInline(Span),
    SafeStack(Span),
    StrongStack(Span),
    WeakStack(Span),
    PreciseFloats(Span),

    // Memory Management
    Stack(Span),
    Heap(Span),

    // Assembler Attributes
    AsmThrow(Span),
    AsmSyntax(&'ctx str, Span),
    AsmAlignStack(Span),
    AsmSideEffects(Span),
}

impl LLVMAttribute<'_> {
    #[inline]
    pub const fn is_extern_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Extern(..))
    }

    #[inline]
    pub const fn is_hot_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Hot(..))
    }

    #[inline]
    pub const fn is_ignore_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Ignore(..))
    }

    #[inline]
    pub const fn is_public_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Public(..))
    }

    #[inline]
    pub const fn is_noinline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::NoInline(..))
    }

    #[inline]
    pub const fn is_inline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::InlineHint(..))
    }

    #[inline]
    pub const fn is_alwaysinline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AlwaysInline(..))
    }

    #[inline]
    pub const fn is_minsize_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::MinSize(..))
    }

    #[inline]
    pub const fn is_heap_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Heap(..))
    }

    #[inline]
    pub const fn is_stack_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Stack(..))
    }

    #[inline]
    pub const fn is_asmsideeffects_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmSideEffects(..))
    }

    #[inline]
    pub const fn is_asmthrow_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmThrow(..))
    }

    #[inline]
    pub const fn is_asmalingstack_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmAlignStack(..))
    }

    #[inline]
    pub fn get_span(&self) -> Span {
        match self {
            LLVMAttribute::Extern(_, span) => *span,
            LLVMAttribute::Convention(_, span) => *span,
            LLVMAttribute::Public(span) => *span,
            LLVMAttribute::Ignore(span) => *span,
            LLVMAttribute::Hot(span) => *span,
            LLVMAttribute::NoInline(span) => *span,
            LLVMAttribute::InlineHint(span) => *span,
            LLVMAttribute::MinSize(span) => *span,
            LLVMAttribute::AlwaysInline(span) => *span,
            LLVMAttribute::SafeStack(span) => *span,
            LLVMAttribute::StrongStack(span) => *span,
            LLVMAttribute::WeakStack(span) => *span,
            LLVMAttribute::PreciseFloats(span) => *span,
            LLVMAttribute::AsmThrow(span) => *span,
            LLVMAttribute::AsmSyntax(_, span) => *span,
            LLVMAttribute::AsmSideEffects(span) => *span,
            LLVMAttribute::AsmAlignStack(span) => *span,
            LLVMAttribute::Stack(span) => *span,
            LLVMAttribute::Heap(span) => *span,
        }
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
                        LLVMAttribute::AlwaysInline(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("alwaysinline"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::InlineHint(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("inlinehint"),
                                    1,
                                ),
                            );
                        }

                        LLVMAttribute::NoInline(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("noinline"),
                                    4,
                                ),
                            );
                        }

                        LLVMAttribute::Hot(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("hot"),
                                    2,
                                ),
                            );
                        }

                        LLVMAttribute::MinSize(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("optsize"),
                                    3,
                                ),
                            );
                        }

                        LLVMAttribute::SafeStack(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("safestack"),
                                    5,
                                ),
                            );
                        }

                        LLVMAttribute::WeakStack(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("ssp"),
                                    5,
                                ),
                            );
                        }

                        LLVMAttribute::StrongStack(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("sspstrong"),
                                    5,
                                ),
                            );
                        }

                        LLVMAttribute::PreciseFloats(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("strictfp"),
                                    5,
                                ),
                            );
                        }

                        LLVMAttribute::Convention(new_call_convention, ..) => {
                            *call_convention = *new_call_convention as u32;
                        }

                        _ => (),
                    })
            }
        }
    }
}
