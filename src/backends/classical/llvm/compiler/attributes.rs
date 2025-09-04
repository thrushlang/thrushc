#![allow(clippy::upper_case_acronyms)]

use crate::backends::classical::llvm::compiler::conventions::CallConvention;
use crate::frontends::classical::lexer::span::Span;

use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    values::FunctionValue,
};

#[derive(Debug, Clone, Copy)]
pub enum LLVMAttribute<'ctx> {
    // Function Attributes
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
    pub fn is_extern_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Extern(..))
    }

    #[inline]
    pub fn is_hot_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Hot(..))
    }

    #[inline]
    pub fn is_ignore_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Ignore(..))
    }

    #[inline]
    pub fn is_public_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Public(..))
    }

    #[inline]
    pub fn is_noinline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::NoInline(..))
    }

    #[inline]
    pub fn is_inline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::InlineHint(..))
    }

    #[inline]
    pub fn is_alwaysinline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AlwaysInline(..))
    }

    #[inline]
    pub fn is_minsize_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::MinSize(..))
    }

    #[inline]
    pub fn is_heap_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Heap(..))
    }

    #[inline]
    pub fn is_stack_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Stack(..))
    }

    #[inline]
    pub fn is_asmsideeffects_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmSideEffects(..))
    }

    #[inline]
    pub fn is_asmthrow_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmThrow(..))
    }

    #[inline]
    pub fn is_asmalingstack_attribute(&self) -> bool {
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
    llvm_context: &'ctx Context,
    attributes: &'ctx [LLVMAttribute<'ctx>],
    attribute_applicant: LLVMAttributeApplicant<'ctx>,
}

impl<'ctx> AttributeBuilder<'ctx> {
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

    pub fn add_function_attributes(&mut self, call_convention: &mut u32) {
        match self.attribute_applicant {
            LLVMAttributeApplicant::Function(function) => {
                self.attributes
                    .iter()
                    .for_each(|attribute| match attribute {
                        LLVMAttribute::AlwaysInline(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.llvm_context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("alwaysinline"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::InlineHint(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self::create_inline_hint_attribute(self.llvm_context),
                            );
                        }

                        LLVMAttribute::NoInline(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.llvm_context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("noinline"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::Hot(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.llvm_context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("hot"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::MinSize(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.llvm_context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("optsize"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::SafeStack(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.llvm_context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("safestack"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::WeakStack(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.llvm_context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("ssp"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::StrongStack(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.llvm_context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("sspstrong"),
                                    0,
                                ),
                            );
                        }

                        LLVMAttribute::PreciseFloats(..) => {
                            function.add_attribute(
                                AttributeLoc::Function,
                                self.llvm_context.create_enum_attribute(
                                    Attribute::get_named_enum_kind_id("strictfp"),
                                    0,
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

#[inline]
pub fn create_inline_hint_attribute(llvm_context: &Context) -> Attribute {
    llvm_context.create_enum_attribute(Attribute::get_named_enum_kind_id("inlinehint"), 0)
}

#[inline]
pub fn create_always_inline_attribute(llvm_context: &Context) -> Attribute {
    llvm_context.create_enum_attribute(Attribute::get_named_enum_kind_id("alwaysinline"), 0)
}

#[inline]
pub fn create_minsize_attribute(llvm_context: &Context) -> Attribute {
    llvm_context.create_enum_attribute(Attribute::get_named_enum_kind_id("optsize"), 0)
}
