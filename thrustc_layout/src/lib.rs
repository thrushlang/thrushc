#![allow(clippy::field_reassign_with_default)]

use thrustc_llvm_target_triple::LLVMTargetTriple;
use thrustc_typesystem::Type;

#[derive(Debug, Clone, Copy, Default)]
pub struct TypeLayout {
    pub width: u32,
    pub align: u32,
    pub sizeof: u32,
}

#[derive(Debug)]
pub struct StructTypeLayout {
    pub width: u32,
    pub align: u32,
    pub field_offsets: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Default)]
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
    pub fn get_type_info(&self, r#type: &Type) -> either::Either<TypeLayout, StructTypeLayout> {
        let mut type_info: TypeLayout = TypeLayout::default();

        match r#type {
            Type::Const(subtype, ..) => self.get_type_info(subtype),

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
                let element_width: u32 = match self.get_type_info(element_type) {
                    either::Either::Left(lft) => lft.width,
                    either::Either::Right(rht) => rht.width,
                };

                let element_align: u32 = match self.get_type_info(element_type) {
                    either::Either::Left(lft) => lft.align,
                    either::Either::Right(rht) => rht.align,
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
                    let element_width: u32 = match self.get_type_info(element_type) {
                        either::Either::Left(lft) => lft.width,
                        either::Either::Right(rht) => rht.width,
                    };

                    let element_align: u32 = match self.get_type_info(element_type) {
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
                let mut current_offset: u32 = 0;
                let mut max_alignment: u32 = 1;
                let mut field_offsets: Vec<u32> = Vec::new();

                for field in types {
                    let field_width: u32 = match self.get_type_info(field) {
                        either::Either::Left(lft) => lft.width,
                        either::Either::Right(rht) => rht.width,
                    };

                    let field_align: u32 = match self.get_type_info(field) {
                        either::Either::Left(lft) => lft.align,
                        either::Either::Right(rht) => rht.align,
                    };

                    let field_size: u32 = field_width / self.i8_align();
                    let field_align: u32 = field_align / self.i8_align();

                    current_offset = align_to(current_offset, field_align);

                    field_offsets.push(current_offset);

                    current_offset = current_offset.saturating_add(field_size);

                    if field_align > max_alignment {
                        max_alignment = field_align;
                    }
                }

                let total_size: u32 = align_to(current_offset, max_alignment);

                either::Either::Right(StructTypeLayout {
                    width: total_size,
                    align: max_alignment,
                    field_offsets,
                })
            }
        }
    }
}

pub fn align_to(value: u32, align: u32) -> u32 {
    if align == 0 {
        return value;
    }

    value.div_ceil(align) * align
}
