#![allow(clippy::enum_variant_names)]

use crate::backends::classical::llvm::compiler::alloc::atomic::LLVMAtomicModificators;
use crate::backends::classical::llvm::compiler::{abort, typegen};
use crate::frontends::classical::types::ast::metadata::dereference::LLVMDereferenceMetadata;
use crate::frontends::classical::typesystem::traits::LLVMTypeExtensions;
use crate::{
    backends::classical::llvm::compiler::alloc::atomic, frontends::classical::lexer::span::Span,
};

use crate::frontends::classical::types::ast::metadata::constant::LLVMConstantMetadata;
use crate::frontends::classical::types::ast::metadata::local::LLVMLocalMetadata;
use crate::frontends::classical::types::ast::metadata::staticvar::LLVMStaticMetadata;
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use std::fmt::Display;
use std::path::PathBuf;

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum, IntValue, PointerValue},
};

use super::context::LLVMCodeGenContext;

#[derive(Debug, Clone, Copy)]
pub enum SymbolAllocated<'ctx> {
    Local {
        ptr: PointerValue<'ctx>,
        kind: &'ctx Type,
        metadata: LLVMLocalMetadata,
        span: Span,
    },
    Static {
        ptr: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
        metadata: LLVMStaticMetadata,
        span: Span,
    },
    Constant {
        ptr: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
        metadata: LLVMConstantMetadata,
        span: Span,
    },
    LowLevelInstruction {
        value: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
        span: Span,
    },
    Parameter {
        value: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
        span: Span,
    },
    Function {
        ptr: PointerValue<'ctx>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum SymbolToAllocate {
    Parameter,
    LowLevelInstruction,
}

#[derive(Debug, Clone, Copy)]
pub enum LLVMAllocationSite {
    Heap,
    Stack,
    Static,
}

impl<'ctx> SymbolAllocated<'ctx> {
    #[inline]
    pub fn new(
        allocate: SymbolToAllocate,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
        span: Span,
    ) -> Self {
        match allocate {
            SymbolToAllocate::Parameter => Self::Parameter { value, kind, span },
            SymbolToAllocate::LowLevelInstruction => {
                Self::LowLevelInstruction { value, kind, span }
            }
        }
    }

    #[inline]
    pub fn new_function(ptr: PointerValue<'ctx>, span: Span) -> Self {
        Self::Function { ptr, span }
    }

    #[inline]
    pub fn new_local(
        ptr: PointerValue<'ctx>,
        kind: &'ctx Type,
        metadata: LLVMLocalMetadata,
        span: Span,
    ) -> Self {
        Self::Local {
            ptr,
            kind,
            metadata,
            span,
        }
    }

    #[inline]
    pub fn new_constant(
        ptr: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
        metadata: LLVMConstantMetadata,
        span: Span,
    ) -> Self {
        Self::Constant {
            ptr: ptr.into_pointer_value(),
            value,
            kind,
            metadata,
            span,
        }
    }

    #[inline]
    pub fn new_static(
        ptr: BasicValueEnum<'ctx>,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
        metadata: LLVMStaticMetadata,
        span: Span,
    ) -> Self {
        Self::Static {
            ptr: ptr.into_pointer_value(),
            value,
            kind,
            metadata,
            span,
        }
    }
}

impl<'ctx> SymbolAllocated<'ctx> {
    pub fn load(&self, context: &mut LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = context.get_target_data();

        let kind: &Type = self.get_type();

        if kind.llvm_is_ptr_type() {
            return self.get_ptr().into();
        }

        let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);
        let alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

        if let Self::Local { ptr, metadata, .. } = self {
            if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, *ptr, "") {
                if let Some(instr) = loaded_value.as_instruction_value() {
                    atomic::try_set_atomic_modificators(
                        instr,
                        LLVMAtomicModificators {
                            atomic_volatile: metadata.volatile,
                            atomic_ord: metadata.atomic_ord,
                        },
                    );

                    let _ = instr.set_alignment(alignment);
                }

                return loaded_value;
            }
        }

        if let Self::Constant { ptr, metadata, .. } = self {
            if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, *ptr, "") {
                if let Some(instr) = loaded_value.as_instruction_value() {
                    atomic::try_set_atomic_modificators(
                        instr,
                        LLVMAtomicModificators {
                            atomic_volatile: metadata.volatile,
                            atomic_ord: metadata.atomic_ord,
                        },
                    );

                    let _ = instr.set_alignment(alignment);
                }

                return loaded_value;
            }
        }

        if let Self::Static { ptr, metadata, .. } = self {
            if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, *ptr, "") {
                if let Some(instr) = loaded_value.as_instruction_value() {
                    atomic::try_set_atomic_modificators(
                        instr,
                        LLVMAtomicModificators {
                            atomic_volatile: metadata.volatile,
                            atomic_ord: metadata.atomic_ord,
                        },
                    );

                    let _ = instr.set_alignment(alignment);
                }

                return loaded_value;
            }
        }

        if let Self::LowLevelInstruction { value, .. } = self {
            return *value;
        }

        if let Self::Parameter { value, .. } = self {
            if value.is_pointer_value() {
                let ptr: PointerValue = value.into_pointer_value();

                if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, ptr, "") {
                    if let Some(instr) = loaded_value.as_instruction_value() {
                        let _ = instr.set_alignment(alignment);
                    }

                    return loaded_value;
                }
            } else {
                return *value;
            }
        }

        abort::abort_codegen(
            context,
            "Failed to load a value from memory!",
            self.get_span(),
            PathBuf::from(file!()),
            line!(),
        );
    }

    pub fn store(
        &self,
        context: &mut LLVMCodeGenContext<'_, 'ctx>,
        new_value: BasicValueEnum<'ctx>,
    ) {
        let llvm_context: &Context = context.get_llvm_context();
        let llvm_builder: &Builder = context.get_llvm_builder();

        let target_data: &TargetData = context.get_target_data();

        let thrush_type: &Type = self.get_type();
        let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, thrush_type);

        let alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

        if let Self::Local { ptr, .. } = self {
            if let Ok(store) = llvm_builder.build_store(*ptr, new_value) {
                let _ = store.set_alignment(alignment);
                return;
            }
        }

        if let Self::LowLevelInstruction { value, .. } | Self::Parameter { value, .. } = self {
            if let Ok(store) = llvm_builder.build_store(value.into_pointer_value(), new_value) {
                let _ = store.set_alignment(alignment);
                return;
            }
        }

        abort::abort_codegen(
            context,
            "Failed to store a value in memory!",
            self.get_span(),
            PathBuf::from(file!()),
            line!(),
        );
    }
}

impl<'ctx> SymbolAllocated<'ctx> {
    #[inline]
    pub fn get_span(&self) -> Span {
        match self {
            Self::Local { span, .. } => *span,
            Self::Constant { span, .. } => *span,
            Self::Static { span, .. } => *span,
            Self::Parameter { span, .. } => *span,
            Self::LowLevelInstruction { span, .. } => *span,
            Self::Function { span, .. } => *span,
        }
    }

    #[inline]
    pub fn get_type(&self) -> &'ctx Type {
        match self {
            Self::Local { kind, .. } => kind,
            Self::Constant { kind, .. } => kind,
            Self::Static { kind, .. } => kind,
            Self::Parameter { kind, .. } => kind,
            Self::LowLevelInstruction { kind, .. } => kind,

            _ => {
                self::codegen_abort("Unable to get type of a allocated symbol.");
            }
        }
    }

    #[inline]
    pub fn get_ptr(&self) -> PointerValue<'ctx> {
        match self {
            Self::Function { ptr, .. } => *ptr,
            Self::Local { ptr, .. } => *ptr,
            Self::Constant { ptr, .. } => *ptr,
            Self::Static { ptr, .. } => *ptr,
            Self::Parameter { value, .. } => value.into_pointer_value(),
            Self::LowLevelInstruction { value, .. } => value.into_pointer_value(),
        }
    }

    #[inline]
    pub fn get_value(&self) -> BasicValueEnum<'ctx> {
        match self {
            Self::Local { ptr, .. } => (*ptr).into(),
            Self::Function { ptr, .. } => (*ptr).into(),
            Self::Constant { value, .. } => *value,
            Self::Static { value, .. } => *value,
            Self::Parameter { value, .. } => *value,
            Self::LowLevelInstruction { value, .. } => *value,
        }
    }
}

pub fn store_anon<'ctx>(
    context: &mut LLVMCodeGenContext<'_, '_>,
    ptr: PointerValue<'ctx>,
    value: BasicValueEnum<'ctx>,
    span: Span,
) {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let target_data: &TargetData = context.get_target_data();

    let alignment: u32 = target_data.get_preferred_alignment(&value.get_type());

    if let Ok(store) = llvm_builder.build_store(ptr, value) {
        let _ = store.set_alignment(alignment);
        return;
    }

    abort::abort_codegen(
        context,
        "Failed to store a value in memory!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn load_anon<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ptr: PointerValue<'ctx>,
    ptr_type: &Type,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let llvm_type: BasicTypeEnum = typegen::generate(llvm_context, ptr_type);

    let preferred_alignment: u32 = context
        .get_target_data()
        .get_preferred_alignment(&ptr.get_type());

    if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, ptr, "") {
        if let Some(instr) = loaded_value.as_instruction_value() {
            let _ = instr.set_alignment(preferred_alignment);
        }

        return loaded_value;
    }

    abort::abort_codegen(
        context,
        "Failed to load a value from memory!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn deference<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ptr: PointerValue<'ctx>,
    ptr_type: &Type,
    metadata: LLVMDereferenceMetadata,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let llvm_type: BasicTypeEnum = typegen::generate(llvm_context, ptr_type);

    let preferred_alignment: u32 = context
        .get_target_data()
        .get_preferred_alignment(&ptr.get_type());

    if let Ok(loaded_value) = llvm_builder.build_load(llvm_type, ptr, "") {
        if let Some(instr) = loaded_value.as_instruction_value() {
            atomic::try_set_atomic_modificators(
                instr,
                LLVMAtomicModificators {
                    atomic_volatile: metadata.volatile,
                    atomic_ord: metadata.atomic_ord,
                },
            );

            let _ = instr.set_alignment(preferred_alignment);
        }

        return loaded_value;
    }

    abort::abort_codegen(
        context,
        "Failed to deference a pointer!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn alloc_anon<'ctx>(
    site: LLVMAllocationSite,
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
) -> PointerValue<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

    let alignment: u32 = context
        .get_target_data()
        .get_preferred_alignment(&llvm_type);

    match site {
        LLVMAllocationSite::Stack => {
            if let Ok(ptr) = llvm_builder.build_alloca(llvm_type, "") {
                if let Some(instruction) = ptr.as_instruction() {
                    let _ = instruction.set_alignment(alignment);
                }

                return ptr;
            }

            self::codegen_abort(format!("Cannot assign type to stack: '{}'.", kind));
        }

        LLVMAllocationSite::Heap => {
            if let Ok(ptr) = llvm_builder.build_malloc(llvm_type, "") {
                return ptr;
            }

            self::codegen_abort(format!("Cannot assign type to heap: '{}'.", kind));
        }

        LLVMAllocationSite::Static => llvm_module
            .add_global(llvm_type, Some(AddressSpace::default()), "")
            .as_pointer_value(),
    }
}

pub fn gep_struct_anon<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ptr: PointerValue<'ctx>,
    ptr_type: &Type,
    index: u32,
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let Ok(ptr) = llvm_builder.build_struct_gep(
        typegen::generate_gep(llvm_context, ptr_type),
        ptr,
        index,
        "",
    ) {
        return ptr;
    }

    self::codegen_abort("Unable to get pointer element at memory manipulation.");
}

pub fn gep_anon<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ptr: PointerValue<'ctx>,
    ptr_type: &Type,
    indexes: &[IntValue<'ctx>],
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let Ok(ptr) = unsafe {
        llvm_builder.build_gep(
            typegen::generate_gep(llvm_context, ptr_type),
            ptr,
            indexes,
            "",
        )
    } {
        return ptr;
    }

    self::codegen_abort("Unable to get pointer element at memory manipulation.");
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
