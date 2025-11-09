use crate::front_end::typesystem::types::Type;

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
            Type::U128 => write!(f, "u128"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::F128 => write!(f, "f128"),
            Type::FX8680 => write!(f, "fx86_80"),
            Type::FPPC128 => write!(f, "fppc_128"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::NullPtr => write!(f, "nullptr"),
            Type::Fn(params, kind, modificator) => {
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
            Type::Const(inner_type) => write!(f, "const {}", inner_type),
            Type::FixedArray(kind, size) => {
                write!(f, "array[{}; {}]", kind, size)
            }
            Type::Array(kind) => {
                write!(f, "array[{}]", kind)
            }
            Type::Struct(name, fields, modificator) => {
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
