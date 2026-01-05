use crate::{
    Type,
    traits::{TypeCodeLocation, TypeExtensions, TypeIsExtensions},
};

impl TypeIsExtensions for Type {
    #[inline(always)]
    fn is_char_type(&self) -> bool {
        matches!(self, Type::Char(..))
    }

    #[inline(always)]
    fn is_void_type(&self) -> bool {
        if let Type::Const(subtype, ..) = self {
            return subtype.is_void_type();
        }

        if let Type::Ptr(Some(subtype), ..) = self {
            return subtype.is_void_type();
        }

        matches!(self, Type::Void(..))
    }

    #[inline(always)]
    fn is_bool_type(&self) -> bool {
        matches!(self, Type::Bool(..))
    }

    #[inline(always)]
    fn is_struct_type(&self) -> bool {
        matches!(self, Type::Struct(..))
    }

    #[inline(always)]
    fn is_fixed_array_type(&self) -> bool {
        matches!(self, Type::FixedArray(..))
    }

    #[inline(always)]
    fn is_array_type(&self) -> bool {
        matches!(self, Type::Array(..))
    }

    #[inline(always)]
    fn is_float_type(&self) -> bool {
        matches!(
            self,
            Type::F32(..) | Type::F64(..) | Type::F128(..) | Type::FX8680(..) | Type::FPPC128(..)
        )
    }

    #[inline(always)]
    fn is_ptr_type(&self) -> bool {
        matches!(self, Type::Ptr(..))
    }

    #[inline(always)]
    fn is_ptr_like_type(&self) -> bool {
        matches!(
            self,
            Type::Ptr(..) | Type::Addr(..) | Type::Array(..) | Type::Fn(..)
        )
    }

    #[inline(always)]
    fn is_address_type(&self) -> bool {
        matches!(self, Type::Addr(..))
    }

    #[inline(always)]
    fn is_const_type(&self) -> bool {
        matches!(self, Type::Const(..))
    }

    #[inline(always)]
    fn is_fnref_type(&self) -> bool {
        matches!(self, Type::Fn(..))
    }

    #[inline(always)]
    fn is_numeric_type(&self) -> bool {
        self.is_integer_type() || self.is_float_type() || self.is_char_type() || self.is_bool_type()
    }

    #[inline(always)]
    fn is_unsigned_integer_type(&self) -> bool {
        matches!(
            self,
            Type::U8(..)
                | Type::U16(..)
                | Type::U32(..)
                | Type::U64(..)
                | Type::U128(..)
                | Type::USize(..)
        )
    }

    #[inline(always)]
    fn is_signed_integer_type(&self) -> bool {
        matches!(
            self,
            Type::S8(..) | Type::S16(..) | Type::S32(..) | Type::S64(..) | Type::SSize(..)
        )
    }

    #[inline(always)]
    fn is_lesseq_unsigned32bit_integer(&self) -> bool {
        matches!(self, Type::U8(..) | Type::U16(..) | Type::U32(..))
    }

    #[inline(always)]
    fn is_integer_type(&self) -> bool {
        matches!(
            self,
            Type::S8(..)
                | Type::S16(..)
                | Type::S32(..)
                | Type::S64(..)
                | Type::SSize(..)
                | Type::U8(..)
                | Type::U16(..)
                | Type::U32(..)
                | Type::U64(..)
                | Type::U128(..)
                | Type::USize(..)
                | Type::Char(..)
        )
    }
}

impl TypeExtensions for Type {
    #[inline]
    fn is_value(&self) -> bool {
        self.is_numeric_type()
            || self.is_fixed_array_type()
            || self.is_struct_type()
            || self.is_const_value()
    }

    #[inline]
    fn is_const_value(&self) -> bool {
        if let Type::Const(inner, ..) = self {
            return inner.is_const_value();
        }

        self.is_numeric_type() || self.is_fixed_array_type() || self.is_struct_type()
    }

    fn get_type_with_depth(&self, base_depth: usize) -> &Type {
        if base_depth == 0 {
            return self;
        }

        match self {
            Type::FixedArray(element_type, ..) => element_type.get_type_with_depth(base_depth - 1),
            Type::Array(element_type, ..) => element_type.get_type_with_depth(base_depth - 1),
            Type::Const(inner_type, ..) => inner_type.get_type_with_depth(base_depth - 1),
            Type::Ptr(Some(inner_type), ..) => inner_type.get_type_with_depth(base_depth - 1),
            Type::Struct(..) => self,
            Type::S8(..)
            | Type::S16(..)
            | Type::S32(..)
            | Type::S64(..)
            | Type::SSize(..)
            | Type::U8(..)
            | Type::U16(..)
            | Type::U32(..)
            | Type::U64(..)
            | Type::U128(..)
            | Type::USize(..)
            | Type::F32(..)
            | Type::F64(..)
            | Type::F128(..)
            | Type::FX8680(..)
            | Type::FPPC128(..)
            | Type::Bool(..)
            | Type::Char(..)
            | Type::Addr(..)
            | Type::Void(..)
            | Type::Ptr(None, ..)
            | Type::Fn(..) => self,
        }
    }

    #[inline]
    fn get_type_ref(&self) -> Type {
        if self.is_ptr_like_type() {
            self.clone()
        } else {
            Type::Ptr(Some(self.clone().into()), self.get_span())
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Fn(lhs, retlhs, mod1, ..), Type::Fn(rhs, retrhs, mod2, ..)) => {
                lhs.len() == rhs.len()
                    && lhs.iter().zip(lhs.iter()).all(|(f1, f2)| f1 == f2)
                    && retlhs == retrhs
                    && mod1 == mod2
            }

            (Type::Struct(a, fields1, mod1, ..), Type::Struct(b, fields2, mod2, ..)) => {
                fields1.len() == fields2.len()
                    && a == b
                    && fields1.iter().zip(fields2.iter()).all(|(f1, f2)| f1 == f2)
                    && mod1 == mod2
            }

            (Type::FixedArray(type_a, size_a, ..), Type::FixedArray(type_b, size_b, ..)) => {
                type_a == type_b && size_a == size_b
            }

            (Type::Array(target, ..), Type::Array(from, ..)) => target == from,
            (Type::Const(target, ..), Type::Const(from, ..)) => target == from,

            (Type::Char(..), Type::Char(..)) => true,
            (Type::S8(..), Type::S8(..)) => true,
            (Type::S16(..), Type::S16(..)) => true,
            (Type::S32(..), Type::S32(..)) => true,
            (Type::S64(..), Type::S64(..)) => true,
            (Type::SSize(..), Type::SSize(..)) => true,
            (Type::U8(..), Type::U8(..)) => true,
            (Type::U16(..), Type::U16(..)) => true,
            (Type::U32(..), Type::U32(..)) => true,
            (Type::U64(..), Type::U64(..)) => true,
            (Type::U128(..), Type::U128(..)) => true,
            (Type::USize(..), Type::USize(..)) => true,
            (Type::F32(..), Type::F32(..)) => true,
            (Type::F64(..), Type::F64(..)) => true,
            (Type::F128(..), Type::F128(..)) => true,
            (Type::FX8680(..), Type::FX8680(..)) => true,
            (Type::FPPC128(..), Type::FPPC128(..)) => true,
            (Type::Ptr(None, ..), Type::Ptr(None, ..)) => true,
            (Type::Ptr(Some(lhs), ..), Type::Ptr(Some(rhs), ..)) => lhs == rhs,
            (Type::Ptr(..), Type::Ptr(..)) => true,
            (Type::Void(..), Type::Void(..)) => true,
            (Type::Bool(..), Type::Bool(..)) => true,
            (Type::Addr(..), Type::Addr(..)) => true,

            _ => false,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::S8(..) => write!(f, "s8"),
            Type::S16(..) => write!(f, "s16"),
            Type::S32(..) => write!(f, "s32"),
            Type::S64(..) => write!(f, "s64"),
            Type::SSize(..) => write!(f, "ssize"),
            Type::U8(..) => write!(f, "u8"),
            Type::U16(..) => write!(f, "u16"),
            Type::U32(..) => write!(f, "u32"),
            Type::U64(..) => write!(f, "u64"),
            Type::U128(..) => write!(f, "u128"),
            Type::USize(..) => write!(f, "usize"),
            Type::F32(..) => write!(f, "f32"),
            Type::F64(..) => write!(f, "f64"),
            Type::F128(..) => write!(f, "f128"),
            Type::FX8680(..) => write!(f, "fx86_80"),
            Type::FPPC128(..) => write!(f, "fppc_128"),
            Type::Bool(..) => write!(f, "bool"),
            Type::Char(..) => write!(f, "char"),
            Type::Fn(params, kind, modificator, ..) => {
                let has_llvm_ignore: &str = if modificator.llvm().has_ignore() {
                    "<ignore>"
                } else {
                    ""
                };

                write!(
                    f,
                    "Fn{}[{}] -> {}",
                    has_llvm_ignore,
                    params
                        .iter()
                        .map(|param| param.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    kind
                )
            }
            Type::Const(inner_type, ..) => write!(f, "const {}", inner_type),
            Type::FixedArray(kind, size, ..) => {
                write!(f, "array[{}; {}]", kind, size)
            }
            Type::Array(kind, ..) => {
                write!(f, "array[{}]", kind)
            }
            Type::Struct(name, fields, modificator, ..) => {
                let is_llvm_packed: &str = if modificator.llvm().is_packed() {
                    "<packed>"
                } else {
                    ""
                };

                write!(f, "struct {}{} {{ ", name, is_llvm_packed)?;

                fields.iter().for_each(|field| {
                    let _ = write!(f, "{} ", field);
                });

                write!(f, "}}")
            }
            Type::Ptr(nested_type, ..) => {
                if let Some(nested_type) = nested_type {
                    let _ = write!(f, "ptr[");
                    let _ = write!(f, "{}", nested_type);

                    return write!(f, "]");
                }

                write!(f, "ptr")
            }
            Type::Addr(..) => {
                write!(f, "memory address")
            }
            Type::Void(..) => write!(f, "void"),
        }
    }
}
