#![allow(clippy::field_reassign_with_default)]

use thrustc_llvm_target_triple::LLVMTargetTriple;

use ahash::AHashMap as HashMap;
use either::Either;

use super::Type;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layout {
    pub width: u32,
    pub align: u32,
    pub sizeof: u32,
    pub field_offsets: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TypeLayout {
    pub width: u32,
    pub align: u32,
    pub sizeof: u32,
}

impl TypeLayout {
    pub fn into_layout(self) -> Layout {
        Layout {
            width: self.width,
            align: self.align,
            sizeof: self.sizeof,
            field_offsets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructTypeLayout {
    pub width: u32,
    pub align: u32,
    pub sizeof: u32,
    pub field_offsets: Vec<u32>,
}

impl StructTypeLayout {
    pub fn into_layout(self) -> Layout {
        Layout {
            width: self.width,
            align: self.align,
            sizeof: self.sizeof,
            field_offsets: self.field_offsets,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TargetInfo {
    bool_width: u32,
    bool_align: u32,

    i8_width: u32,
    i16_width: u32,
    i32_width: u32,
    i64_width: u32,
    i128_width: u32,
    isize_width: u32,
    usize_width: u32,

    i8_align: u32,
    i16_align: u32,
    i32_align: u32,
    i64_align: u32,
    i128_align: u32,
    isize_align: u32,
    usize_align: u32,

    f16_width: u32,
    f16_align: u32,

    f32_width: u32,
    f32_align: u32,

    f64_width: u32,
    f64_align: u32,

    f128_width: u32,
    f128_align: u32,

    ptr_width: u32,
    ptr_align: u32,

    type_cached: HashMap<Type, Either<TypeLayout, StructTypeLayout>>,
}

impl TargetInfo {
    pub fn new(triple: LLVMTargetTriple) -> Self {
        let mut target_info: TargetInfo = TargetInfo::default();

        let is_64_bit: bool = triple.is_64_bit();

        target_info.bool_width = 8;
        target_info.bool_align = 8;

        target_info.i8_width = 8;
        target_info.i8_align = 8;

        target_info.i16_width = 16;
        target_info.i16_align = 16;

        target_info.i32_width = 32;
        target_info.i32_align = 32;

        target_info.i64_width = 64;
        target_info.i64_align = 64;

        target_info.i128_width = 128;
        target_info.i128_align = 128;

        target_info.isize_width = if is_64_bit { 64 } else { 32 };
        target_info.isize_align = if is_64_bit { 64 } else { 32 };

        target_info.usize_width = if is_64_bit { 64 } else { 32 };
        target_info.usize_align = if is_64_bit { 64 } else { 32 };

        target_info.f16_width = 16;
        target_info.f16_align = 16;

        target_info.f32_width = 32;
        target_info.f32_align = 32;

        target_info.f64_width = 64;
        target_info.f64_align = 64;

        target_info.f128_width = 128;
        target_info.f128_align = 128;

        target_info.ptr_width = if is_64_bit { 64 } else { 32 };
        target_info.ptr_align = if is_64_bit { 64 } else { 32 };

        target_info
    }
}

impl TargetInfo {
    #[inline]
    pub fn bool_width(&self) -> u32 {
        self.bool_width
    }

    #[inline]
    pub fn i8_width(&self) -> u32 {
        self.i8_width
    }

    #[inline]
    pub fn i16_width(&self) -> u32 {
        self.i16_width
    }

    #[inline]
    pub fn i32_width(&self) -> u32 {
        self.i32_width
    }

    #[inline]
    pub fn i64_width(&self) -> u32 {
        self.i64_width
    }

    #[inline]
    pub fn i128_width(&self) -> u32 {
        self.i128_width
    }

    #[inline]
    pub fn isize_width(&self) -> u32 {
        self.isize_width
    }

    #[inline]
    pub fn usize_width(&self) -> u32 {
        self.usize_width
    }

    #[inline]
    pub fn f16_width(&self) -> u32 {
        self.f16_width
    }

    #[inline]
    pub fn f32_width(&self) -> u32 {
        self.f32_width
    }

    #[inline]
    pub fn f64_width(&self) -> u32 {
        self.f64_width
    }

    #[inline]
    pub fn f128_width(&self) -> u32 {
        self.f128_width
    }

    #[inline]
    pub fn ptr_width(&self) -> u32 {
        self.ptr_width
    }
}

impl TargetInfo {
    #[inline]
    pub fn bool_align(&self) -> u32 {
        self.bool_align
    }

    #[inline]
    pub fn i8_align(&self) -> u32 {
        self.i8_align
    }

    #[inline]
    pub fn i16_align(&self) -> u32 {
        self.i16_align
    }

    #[inline]
    pub fn i32_align(&self) -> u32 {
        self.i32_align
    }

    #[inline]
    pub fn i64_align(&self) -> u32 {
        self.i64_align
    }

    #[inline]
    pub fn i128_align(&self) -> u32 {
        self.i128_align
    }

    #[inline]
    pub fn f16_align(&self) -> u32 {
        self.f16_align
    }

    #[inline]
    pub fn f32_align(&self) -> u32 {
        self.f32_align
    }

    #[inline]
    pub fn f64_align(&self) -> u32 {
        self.f64_align
    }

    #[inline]
    pub fn f128_align(&self) -> u32 {
        self.f128_align
    }

    #[inline]
    pub fn isize_align(&self) -> u32 {
        self.isize_align
    }

    #[inline]
    pub fn usize_align(&self) -> u32 {
        self.usize_align
    }

    #[inline]
    pub fn ptr_align(&self) -> u32 {
        self.ptr_align
    }
}

impl TargetInfo {
    pub fn get_type_layout(
        &mut self,
        r#type: &Type,
    ) -> either::Either<TypeLayout, StructTypeLayout> {
        if let Some(cached) = self.type_cached.get(r#type).cloned() {
            return cached;
        }

        let mut type_info: TypeLayout = TypeLayout::default();

        let layout: Either<TypeLayout, StructTypeLayout> = match r#type {
            Type::Const(subtype, ..) => self.get_type_layout(subtype),

            Type::Bool(..) => {
                type_info.width = self.bool_width();
                type_info.align = self.bool_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::S8(..) | Type::U8(..) | Type::Char(..) => {
                type_info.width = self.i8_width();
                type_info.align = self.i8_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::S16(..) | Type::U16(..) => {
                type_info.width = self.i16_width();
                type_info.align = self.i16_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::S32(..) | Type::U32(..) => {
                type_info.width = self.i32_width();
                type_info.align = self.i32_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::S64(..) | Type::U64(..) => {
                type_info.width = self.i64_width();
                type_info.align = self.i64_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::U128(..) => {
                type_info.width = self.i128_width();
                type_info.align = self.i128_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::SSize(..) => {
                type_info.width = self.isize_width();
                type_info.align = self.isize_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::USize(..) => {
                type_info.width = self.usize_width();
                type_info.align = self.usize_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::FX8680(..) => {
                type_info.width = self.f16_width();
                type_info.align = self.f16_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::F32(..) => {
                type_info.width = self.f32_width();
                type_info.align = self.f32_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::F64(..) => {
                type_info.width = self.f64_width();
                type_info.align = self.f64_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::F128(..) | Type::FPPC128(..) => {
                type_info.width = self.f128_width();
                type_info.align = self.f128_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::Void(..) | Type::Unresolved { .. } => {
                type_info.width = 1;
                type_info.align = 8;
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::Ptr(..) | Type::Fn(..) | Type::Addr(..) => {
                type_info.width = self.ptr_width();
                type_info.align = self.ptr_align();
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::FixedArray(element_type, size, ..) => {
                let element_width: u32 = match self.get_type_layout(element_type) {
                    either::Either::Left(left) => left.width,
                    either::Either::Right(right) => right.width,
                };

                let element_align: u32 = match self.get_type_layout(element_type) {
                    either::Either::Left(left) => left.align,
                    either::Either::Right(right) => right.align,
                };

                type_info.width = element_width * size;
                type_info.align = element_align;
                type_info.sizeof = type_info.width / self.i8_width;

                either::Either::Left(type_info)
            }

            Type::Array {
                base_type: element_type,
                infered_type,
                ..
            } => {
                let size: u32 = if let Some((subtype, _)) = infered_type {
                    if let Type::FixedArray(_, size, ..) = &**subtype {
                        *size
                    } else {
                        0
                    }
                } else {
                    0
                };

                if size == 0 {
                    type_info.width = self.ptr_width();
                    type_info.align = self.ptr_align();
                    type_info.sizeof = type_info.width / self.i8_width;

                    either::Either::Left(type_info)
                } else {
                    let element_width: u32 = match self.get_type_layout(element_type) {
                        either::Either::Left(lft) => lft.width,
                        either::Either::Right(rht) => rht.width,
                    };

                    let element_align: u32 = match self.get_type_layout(element_type) {
                        either::Either::Left(lft) => lft.align,
                        either::Either::Right(rht) => rht.align,
                    };

                    type_info.width = element_width * size;
                    type_info.align = element_align;
                    type_info.sizeof = type_info.width / self.i8_width;

                    either::Either::Left(type_info)
                }
            }

            Type::Struct(_, types, _, _) => {
                let mut current_offset_bits: u32 = 0;
                let mut max_align_bits: u32 = 1;
                let mut field_offsets_bits: Vec<u32> = Vec::with_capacity(types.len());

                for field in types {
                    let layout: Either<TypeLayout, StructTypeLayout> = self.get_type_layout(field);

                    let (f_width, f_align) = match layout {
                        Either::Left(l) => (l.width, l.align),
                        Either::Right(r) => (r.width, r.align),
                    };

                    if f_align > 0 {
                        current_offset_bits = current_offset_bits.div_ceil(f_align) * f_align;
                    }

                    field_offsets_bits.push(current_offset_bits);

                    current_offset_bits = current_offset_bits.saturating_add(f_width);

                    if f_align > max_align_bits {
                        max_align_bits = f_align;
                    }
                }

                let total_width_bits: u32 =
                    current_offset_bits.div_ceil(max_align_bits) * max_align_bits;

                either::Either::Right(StructTypeLayout {
                    width: total_width_bits,
                    align: max_align_bits,
                    sizeof: total_width_bits / self.i8_width,
                    field_offsets: field_offsets_bits,
                })
            }
        };

        self.type_cached.insert(r#type.clone(), layout.clone());

        layout
    }
}
