use {
    super::super::super::frontend::lexer::DataTypes,
    inkwell::{context::Context, types::BasicTypeEnum},
};

pub trait BasicValueEnumExt<'ctx> {
    fn get_data_type(&self, context: &'ctx Context) -> DataTypes;
}

impl<'ctx> BasicValueEnumExt<'ctx> for inkwell::values::BasicValueEnum<'ctx> {
    fn get_data_type(&self, context: &'ctx Context) -> DataTypes {
        match self.get_type() {
            BasicTypeEnum::FloatType(kind) => {
                if kind == context.f32_type() {
                    DataTypes::F32
                } else {
                    DataTypes::F64
                }
            }

            BasicTypeEnum::IntType(kind) => {
                if kind == context.i8_type() {
                    DataTypes::I8
                } else if kind == context.i16_type() {
                    DataTypes::I16
                } else if kind == context.i32_type() {
                    DataTypes::I32
                } else if kind == context.i64_type() {
                    DataTypes::I64
                } else {
                    unreachable!()
                }
            }

            BasicTypeEnum::PointerType(_) => {
                // Reemplazar con generic después.

                DataTypes::Void
            }

            _ => unreachable!(),
        }
    }
}
