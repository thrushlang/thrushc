use crate::frontends::classical::typesystem::{traits::TypeExtensions, types::Type};

impl Type {
    #[inline(always)]
    pub fn is_char_type(&self) -> bool {
        matches!(self, Type::Char)
    }

    #[inline(always)]
    pub fn is_void_type(&self) -> bool {
        matches!(self, Type::Void)
    }

    #[inline(always)]
    pub fn is_bool_type(&self) -> bool {
        matches!(self, Type::Bool)
    }

    #[inline(always)]
    pub fn is_struct_type(&self) -> bool {
        matches!(self, Type::Struct(..))
    }

    #[inline(always)]
    pub fn is_fixed_array_type(&self) -> bool {
        matches!(self, Type::FixedArray(..))
    }

    #[inline(always)]
    pub fn is_array_type(&self) -> bool {
        matches!(self, Type::Array(..))
    }

    #[inline(always)]
    pub fn is_float_type(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    #[inline(always)]
    pub fn is_ptr_type(&self) -> bool {
        matches!(self, Type::Ptr(_))
    }

    #[inline(always)]
    pub fn is_address_type(&self) -> bool {
        matches!(self, Type::Addr)
    }

    #[inline(always)]
    pub fn is_mut_type(&self) -> bool {
        matches!(self, Type::Mut(_))
    }

    #[inline(always)]
    pub fn is_const_type(&self) -> bool {
        matches!(self, Type::Const(_))
    }

    #[inline(always)]
    pub fn is_fnref_type(&self) -> bool {
        matches!(self, Type::Fn(..))
    }

    #[inline(always)]
    pub fn is_numeric(&self) -> bool {
        self.is_integer_type() || self.is_float_type() || self.is_char_type() || self.is_bool_type()
    }

    #[inline(always)]
    pub fn is_signed_integer_type(&self) -> bool {
        matches!(self, Type::S8 | Type::S16 | Type::S32 | Type::S64)
    }

    #[inline(always)]
    pub fn is_integer_type(&self) -> bool {
        matches!(
            self,
            Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Char
        )
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
            Type::Struct(..) => self,
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
            | Type::Addr
            | Type::Void
            | Type::Ptr(None)
            | Type::Fn(..) => self,
        }
    }

    fn get_type_fn_ref(&self) -> &Type {
        if let Type::Fn(_, kind, ..) = self {
            return kind;
        }

        self
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Fn(lhs, retlhs, mod1), Type::Fn(rhs, retrhs, mod2)) => {
                lhs.len() == rhs.len()
                    && lhs.iter().zip(lhs.iter()).all(|(f1, f2)| f1 == f2)
                    && retlhs == retrhs
                    && mod1 == mod2
            }

            (Type::Struct(a, fields1, mod1), Type::Struct(b, fields2, mod2)) => {
                fields1.len() == fields2.len()
                    && a == b
                    && fields1.iter().zip(fields2.iter()).all(|(f1, f2)| f1 == f2)
                    && mod1 == mod2
            }

            (Type::FixedArray(type_a, size_a), Type::FixedArray(type_b, size_b)) => {
                type_a == type_b && size_a == size_b
            }

            (Type::Mut(target), Type::Mut(from)) => target == from,
            (Type::Array(target), Type::Array(from)) => target == from,
            (Type::Const(target), Type::Const(from)) => target == from,

            (Type::Char, Type::Char) => true,
            (Type::S8, Type::S8) => true,
            (Type::S16, Type::S16) => true,
            (Type::S32, Type::S32) => true,
            (Type::S64, Type::S64) => true,
            (Type::U8, Type::U8) => true,
            (Type::U16, Type::U16) => true,
            (Type::U32, Type::U32) => true,
            (Type::U64, Type::U64) => true,
            (Type::F32, Type::F32) => true,
            (Type::F64, Type::F64) => true,
            (Type::Ptr(None), Type::Ptr(None)) => true,
            (Type::Ptr(Some(target)), Type::Ptr(Some(from))) => target == from,
            (Type::Void, Type::Void) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Addr, Type::Addr) => true,

            _ => false,
        }
    }
}
