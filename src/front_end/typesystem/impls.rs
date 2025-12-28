use crate::core::diagnostic::span::Span;
use crate::front_end::typesystem::traits::IndexExtensions;
use crate::front_end::typesystem::traits::TypeCodeLocation;
use crate::front_end::typesystem::traits::TypeExtensions;
use crate::front_end::typesystem::traits::TypeIsExtensions;
use crate::front_end::typesystem::types::Type;

impl TypeIsExtensions for Type {
    #[inline(always)]
    fn is_char_type(&self) -> bool {
        matches!(self, Type::Char(..))
    }

    #[inline(always)]
    fn is_void_type(&self) -> bool {
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

impl TypeCodeLocation for Type {
    fn get_span(&self) -> Span {
        match self {
            Type::Char(span)
            | Type::S8(span)
            | Type::S16(span)
            | Type::S32(span)
            | Type::S64(span)
            | Type::SSize(span)
            | Type::U8(span)
            | Type::U16(span)
            | Type::U32(span)
            | Type::U64(span)
            | Type::U128(span)
            | Type::USize(span)
            | Type::F32(span)
            | Type::F64(span)
            | Type::F128(span)
            | Type::FX8680(span)
            | Type::FPPC128(span)
            | Type::Bool(span)
            | Type::Void(span)
            | Type::Addr(span)
            | Type::Array(_, span)
            | Type::FixedArray(_, _, span)
            | Type::Const(_, span)
            | Type::Ptr(_, span)
            | Type::Struct(_, _, _, span)
            | Type::Fn(_, _, _, span) => *span,
        }
    }
}

impl IndexExtensions for Type {
    fn calculate_index_type(&self, depth: usize) -> &Type {
        if depth == 0 {
            return self;
        }

        match self {
            Type::FixedArray(inner_type, ..) => inner_type.get_type_with_depth(depth - 1),
            Type::Array(inner_type, ..) => inner_type.get_type_with_depth(depth - 1),
            Type::Const(inner_type, ..) => inner_type.get_type_with_depth(depth - 1),
            Type::Ptr(Some(inner_type), ..) if !inner_type.is_ptr_like_type() => {
                inner_type.get_type_with_depth(depth)
            }
            Type::Ptr(Some(inner_type), ..) => inner_type.get_type_with_depth(depth - 1),
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
