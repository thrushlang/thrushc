use std::sync::Arc;

use inkwell::{context::Context, targets::TargetData, types::BasicTypeEnum};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    frontend::typesystem::{
        traits::{
            LLVMTypeExtensions, TypeMutableExtensions, TypePointerExtensions, TypeStructExtensions,
        },
        types::Type,
    },
};

impl LLVMTypeExtensions for Type {
    fn llvm_is_same_bit_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &Type) -> bool {
        let llvm_context: &Context = context.get_llvm_context();

        let a_llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, self);
        let b_llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, other);

        let target_data: &TargetData = context.get_target_data();

        target_data.get_bit_size(&a_llvm_type) == target_data.get_bit_size(&b_llvm_type)
    }
}

impl TypePointerExtensions for Type {
    fn is_all_ptr_type(&self) -> bool {
        if let Type::Ptr(Some(ptr)) = self {
            return ptr.is_all_ptr_type();
        }

        if let Type::Ptr(None) = self {
            return true;
        }

        false
    }

    fn is_typed_ptr_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_typed_ptr_type();
        }

        if let Type::Ptr(None) = self {
            return false;
        }

        true
    }

    fn is_ptr_struct_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_struct_type();
        }

        false
    }

    fn is_ptr_fixed_array_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_fixed_array_type();
        }

        false
    }
}

impl TypeStructExtensions for Type {
    fn get_struct_fields(&self) -> &[Arc<Type>] {
        if let Type::Struct(_, fields) = self {
            return fields;
        }

        &[]
    }
}

impl TypeMutableExtensions for Type {
    fn is_mut_fixed_array_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_fixed_array_type();
        }

        false
    }

    fn is_mut_array_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_array_type();
        }

        false
    }

    fn is_mut_struct_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_struct_type();
        }

        false
    }

    fn is_mut_numeric_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            if inner.is_integer_type()
                || inner.is_bool_type()
                || inner.is_char_type()
                || inner.is_float_type()
            {
                return true;
            }
        }

        false
    }

    fn defer_mut_all(&self) -> Type {
        if let Type::Mut(inner_type) = self {
            return inner_type.defer_mut_all();
        }

        self.clone()
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::S8 => write!(f, "s8"),
            Type::S16 => write!(f, "s16"),
            Type::S32 => write!(f, "s32"),
            Type::S64 => write!(f, "s64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Char => write!(f, "char"),
            Type::Mut(any_type) => write!(f, "mut {}", any_type),
            Type::FixedArray(kind, size) => {
                write!(f, "[{}; {}]", kind, size)
            }
            Type::Array(kind) => {
                write!(f, "[{}]", kind)
            }
            Type::Struct(name, fields) => {
                write!(f, "struct {} {{ ", name)?;

                fields.iter().for_each(|field| {
                    let _ = write!(f, "{} ", field);
                });

                write!(f, "}}")
            }
            Type::Ptr(nested_type) => {
                if let Some(nested_type) = nested_type {
                    let _ = write!(f, "ptr[");
                    let _ = write!(f, "{}", nested_type);

                    return write!(f, "]");
                }

                write!(f, "ptr")
            }
            Type::Addr => {
                write!(f, "memory address")
            }
            Type::Void => write!(f, "void"),
        }
    }
}
