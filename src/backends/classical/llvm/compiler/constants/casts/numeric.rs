use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValueEnum, FloatValue, IntValue},
};

pub fn numeric_cast<'ctx>(
    value: BasicValueEnum<'ctx>,
    cast: BasicTypeEnum<'ctx>,
    is_signed: bool,
) -> BasicValueEnum<'ctx> {
    if value.is_int_value() && cast.is_int_type() {
        let integer: IntValue = value.into_int_value();

        return integer.const_cast(cast.into_int_type(), is_signed).into();
    }

    if value.is_float_value() && cast.is_float_type() {
        let float: FloatValue = value.into_float_value();

        return float.const_cast(cast.into_float_type()).into();
    }

    value
}
