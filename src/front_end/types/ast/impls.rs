use crate::front_end::types::ast::{Ast, metadata::local::LocalMetadata};

impl std::cmp::PartialEq for Ast<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ast::Integer { .. }, Ast::Integer { .. })
            | (Ast::Float { .. }, Ast::Float { .. })
            | (Ast::Str { .. }, Ast::Str { .. }) => true,
            (left, right) => std::mem::discriminant(left) == std::mem::discriminant(right),
        }
    }
}

impl std::fmt::Display for Ast<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Invalid { kind, .. } => write!(f, "invalid ast"),
            Ast::Char { byte, .. } => write!(f, "{}", byte),
            Ast::Integer { value, .. } => write!(f, "{}", value),
            Ast::Float { value, .. } => write!(f, "{}", value),
            Ast::Boolean { value, .. } => write!(f, "{}", value),
            Ast::Str { bytes, .. } => {
                write!(f, "\"{}\"", String::from_utf8_lossy(bytes))
            }
            Ast::Function {
                name,
                parameters,
                parameter_types,
                body,
                return_type,
                attributes,
                ..
            } => {
                write!(f, "fn {}(", name)?;

                for (i, (param, param_type)) in
                    parameters.iter().zip(parameter_types.iter()).enumerate()
                {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}: {}", param, param_type)?;
                }

                write!(f, ") {} ", return_type)?;

                attributes
                    .iter()
                    .try_for_each(|attr| write!(f, "{}", attr))?;

                if let Some(body) = body {
                    write!(f, "{}", body)?;
                }

                Ok(())
            }
            Ast::Block { nodes, .. } => {
                let _ = write!(f, "{{ ");

                for node in nodes {
                    let _ = write!(f, "{}", node);
                }

                let _ = write!(f, " }}");

                Ok(())
            }
            Ast::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                write!(f, "{} {} {}", left, operator, right)
            }
            Ast::UnaryOp {
                operator,
                expression,
                is_pre,
                ..
            } => {
                if *is_pre {
                    write!(f, "{}{}", operator, expression)
                } else {
                    write!(f, "{}{}", expression, operator)
                }
            }
            Ast::Break { .. } => {
                write!(f, "break")
            }
            Ast::Continue { .. } => {
                write!(f, "continue")
            }
            Ast::For {
                local,
                condition,
                actions,
                block,
                ..
            } => {
                write!(f, "for {} {} {} {}", local, condition, actions, block)
            }
            Ast::Call { name, args, .. } => {
                write!(f, "{}(", name)?;

                for (index, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;

                    if index > 0 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }

            Ast::If {
                condition,
                block,
                elseif,
                anyway,
                ..
            } => {
                write!(f, "if {} {}", condition, block)?;

                for elif in elseif {
                    write!(f, " elif {}", elif)?;
                }

                if let Some(anyway) = anyway {
                    write!(f, " else {}", anyway)?;
                }

                Ok(())
            }

            Ast::Return { expression, .. } => {
                if let Some(expr) = expression {
                    write!(f, "return {}", expr)?;
                }

                write!(f, "return")
            }

            Ast::Local {
                name,
                kind,
                value,
                metadata,
                ..
            } => {
                let local_metadata: &LocalMetadata = metadata;

                if let Some(value) = value {
                    if local_metadata.is_mutable() {
                        write!(f, "let mut {} : {} = {}", name, kind, value)?;
                    } else {
                        write!(f, "let {} : {} = {}", name, kind, value)?;
                    }
                } else if local_metadata.is_mutable() {
                    write!(f, "let mut {} : {};", name, kind)?;
                } else {
                    write!(f, "let {} : {};", name, kind)?;
                }

                Ok(())
            }

            Ast::Mut { source, value, .. } => {
                write!(f, "{} = {}", source, value)?;

                Ok(())
            }

            Ast::Reference { name, .. } => {
                write!(f, "{}", name)
            }

            Ast::Loop { block, .. } => {
                write!(f, "loop {}", block)
            }

            Ast::While {
                condition, block, ..
            } => {
                write!(f, "while {} {}", condition, block)
            }

            Ast::NullPtr { .. } => {
                write!(f, "nullptr")
            }

            Ast::Group { expression, .. } => {
                write!(f, "({})", expression)
            }

            Ast::Elif {
                condition, block, ..
            } => {
                write!(f, "elif {} {}", condition, block)
            }

            Ast::Else { block, .. } => {
                write!(f, "else {}", block)
            }

            Ast::Unreachable { .. } => {
                write!(f, "unreachable")
            }

            Ast::Const {
                name, kind, value, ..
            } => {
                write!(f, "const {}: {} = {}", name, kind, value)
            }

            Ast::Static {
                name, kind, value, ..
            } => {
                if let Some(value) = value {
                    write!(f, "static {}: {} = {}", name, kind, value)
                } else {
                    write!(f, "static {}: {}", name, kind)
                }
            }

            Ast::Struct { name, .. } => {
                write!(f, "struct {} {{ ... }}", name)
            }

            Ast::Enum { name, .. } => {
                write!(f, "enum {} {{ ... }}", name)
            }

            Ast::EnumValue { name, value, .. } => {
                write!(f, "{}::{}", name, value)
            }

            Ast::Constructor { name, args, .. } => {
                write!(f, "{}{{ ", name)?;
                for (i, (field_name, ..)) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", field_name)?;
                }
                write!(f, " }}")
            }

            Ast::Property { source, .. } => {
                write!(f, "{}.property", source)
            }

            Ast::Index { source, index, .. } => {
                write!(f, "{}[{}]", source, index)
            }

            Ast::Array { items, .. } => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }

            Ast::FixedArray { items, .. } => {
                write!(f, "fixed[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }

            Ast::Deref { value, .. } => {
                write!(f, "*{}", value)
            }

            Ast::DirectRef { expr, .. } => {
                write!(f, "&{}", expr)
            }

            Ast::As { from, cast, .. } => {
                write!(f, "{} as {}", from, cast)
            }

            Ast::LLI { name, expr, .. } => {
                write!(f, "lli {} = {}", name, expr)
            }

            Ast::Alloc { alloc, .. } => {
                write!(f, "alloc {}", alloc)
            }

            Ast::Load { source, .. } => {
                write!(f, "load {}", source)
            }

            Ast::Write {
                source,
                write_value,
                ..
            } => {
                write!(f, "write {} = {}", source, write_value)
            }

            Ast::Address {
                source, indexes, ..
            } => {
                write!(f, "address {}[", source)?;
                for (i, idx) in indexes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", idx)?;
                }
                write!(f, "]")
            }

            Ast::Indirect { function, args, .. } => {
                write!(f, "{}(", function)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }

            Ast::Intrinsic { name, .. } => {
                write!(f, "intrinsic {}", name)
            }

            Ast::AssemblerFunction { name, .. } => {
                write!(f, "asmfn {}", name)
            }

            Ast::AsmValue { assembler, .. } => {
                write!(f, "asm(\"{}\")", assembler)
            }

            Ast::GlobalAssembler { asm, .. } => {
                write!(f, "global_asm(\"{}\")", asm)
            }

            Ast::Builtin { builtin, .. } => {
                write!(f, "builtin({:?})", builtin)
            }

            Ast::FunctionParameter { name, kind, .. } => {
                write!(f, "{}: {}", name, kind)
            }

            Ast::IntrinsicParameter { kind, .. } => {
                write!(f, "param: {}", kind)
            }

            Ast::AssemblerFunctionParameter { name, kind, .. } => {
                write!(f, "{}: {}", name, kind)
            }

            Ast::CustomType { kind, .. } => {
                write!(f, "type {}", kind)
            }

            Ast::Import { .. } => {
                write!(f, "import")
            }

            Ast::ImportC { .. } => {
                write!(f, "importC")
            }
        }
    }
}
