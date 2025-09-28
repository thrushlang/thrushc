use std::sync::Arc;

use inkwell::{context::Context, targets::TargetData, types::BasicTypeEnum};

use crate::{
    backends::classical::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    frontends::classical::typesystem::{
        traits::{
            CastTypeExtensions, DereferenceExtensions, IndexTypeExtensions, LLVMTypeExtensions,
            TypeExtensions, TypeMutableExtensions, TypePointerExtensions, TypeStructExtensions,
        },
        types::Type,
    },
};

impl LLVMTypeExtensions for Type {
    #[inline]
    fn llvm_is_same_bit_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &Type) -> bool {
        let llvm_context: &Context = context.get_llvm_context();

        let a_llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, self);
        let b_llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, other);

        let target_data: &TargetData = context.get_target_data();

        target_data.get_bit_size(&a_llvm_type) == target_data.get_bit_size(&b_llvm_type)
    }

    #[inline]
    fn llvm_is_ptr_type(&self) -> bool {
        matches!(
            self,
            Type::Ptr(_) | Type::Mut(_) | Type::Addr | Type::Array(_)
        )
    }

    #[inline]
    fn llvm_is_int_type(&self) -> bool {
        self.is_integer_type() || self.is_bool_type() || self.is_char_type()
    }

    #[inline]
    fn llvm_is_float_type(&self) -> bool {
        self.is_float_type()
    }
}

impl TypeExtensions for Type {
    fn get_type_with_depth(&self, base_depth: usize) -> &Type {
        if base_depth == 0 {
            return self;
        }

        match self {
            Type::FixedArray(element_type, _) => element_type.get_type_with_depth(base_depth - 1),
            Type::Array(element_type) => element_type.get_type_with_depth(base_depth - 1),
            Type::Mut(inner_type) => inner_type.get_type_with_depth(base_depth - 1),
            Type::Const(inner_type) => inner_type.get_type_with_depth(base_depth - 1),
            Type::Ptr(Some(inner_type)) => inner_type.get_type_with_depth(base_depth - 1),
            Type::Struct(_, _) => self,
            Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Char
            | Type::Str
            | Type::Addr
            | Type::Void
            | Type::Ptr(None) => self,
        }
    }
}

impl IndexTypeExtensions for Type {
    fn get_aprox_type(&self, depth: usize) -> &Type {
        if depth == 0 {
            return self;
        }

        match self {
            Type::FixedArray(element_type, _) => element_type.get_aprox_type(depth),
            Type::Array(element_type) => element_type.get_aprox_type(depth),
            Type::Mut(inner_type) => inner_type.get_aprox_type(depth),
            Type::Const(inner_type) => inner_type.get_aprox_type(depth),
            Type::Ptr(Some(inner_type)) => inner_type.get_aprox_type(depth - 1),
            Type::Struct(_, _) => self,
            Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Char
            | Type::Str
            | Type::Addr
            | Type::Void
            | Type::Ptr(None) => self,
        }
    }
}

impl TypePointerExtensions for Type {
    #[inline]
    fn is_all_ptr_type(&self) -> bool {
        if let Type::Ptr(Some(ptr)) = self {
            return ptr.is_all_ptr_type();
        }

        if let Type::Ptr(None) = self {
            return true;
        }

        false
    }

    #[inline]
    fn is_typed_ptr_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_typed_ptr_type();
        }

        if let Type::Ptr(None) = self {
            return false;
        }

        true
    }

    #[inline]
    fn is_ptr_struct_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_struct_type();
        }

        false
    }

    #[inline]
    fn is_ptr_fixed_array_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_fixed_array_type();
        }

        false
    }
}

impl TypeStructExtensions for Type {
    #[inline]
    fn get_struct_fields(&self) -> &[Arc<Type>] {
        if let Type::Struct(_, fields) = self {
            return fields;
        }

        &[]
    }
}

impl TypeMutableExtensions for Type {
    #[inline]
    fn is_mut_fixed_array_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_fixed_array_type();
        }

        false
    }

    #[inline]
    fn is_mut_array_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_array_type();
        }

        false
    }

    #[inline]
    fn is_mut_struct_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_struct_type();
        }

        false
    }
}

impl CastTypeExtensions for Type {
    fn narrowing(&self) -> Type {
        match self {
            Type::U8 => Type::S8,
            Type::U16 => Type::S16,
            Type::U32 => Type::S32,
            Type::U64 => Type::S64,

            Type::S8 => Type::U8,
            Type::S16 => Type::U16,
            Type::S32 => Type::U32,
            Type::S64 => Type::U64,

            _ => self.clone(),
        }
    }

    fn precompute(&self, other: &Type) -> Type {
        match (self, other) {
            (Type::S64, _) | (_, Type::S64) => Type::S64,
            (Type::S32, _) | (_, Type::S32) => Type::S32,
            (Type::S16, _) | (_, Type::S16) => Type::S16,
            (Type::S8, _) | (_, Type::S8) => Type::S8,

            (Type::U64, _) | (_, Type::U64) => Type::U64,
            (Type::U32, _) | (_, Type::U32) => Type::U32,
            (Type::U16, _) | (_, Type::U16) => Type::U16,
            (Type::U8, _) | (_, Type::U8) => Type::U8,

            (Type::F64, _) | (_, Type::F64) => Type::F64,
            (Type::F32, _) | (_, Type::F32) => Type::F32,

            (Type::Mut(lhs), Type::Mut(rhs)) => lhs.precompute(rhs),
            (Type::Const(lhs), Type::Const(rhs)) => lhs.precompute(rhs),

            _ => self.clone(),
        }
    }
}

impl DereferenceExtensions for Type {
    fn dereference(&self) -> Type {
        if let Type::Ptr(Some(any)) = self {
            return (**any).clone();
        }

        if let Type::Mut(any) = self {
            return (**any).clone();
        }

        if let Type::Const(any) = self {
            return (**any).clone();
        }

        self.clone()
    }

    fn dereference_high_level_type(&self) -> Type {
        if let Type::Mut(inner_type) = self {
            return (**inner_type).clone();
        }

        if let Type::Const(inner_type) = self {
            return inner_type.dereference_high_level_type();
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

            Type::Mut(inner_type) => write!(f, "mut {}", inner_type),
            Type::Const(inner_type) => write!(f, "const {}", inner_type),

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
