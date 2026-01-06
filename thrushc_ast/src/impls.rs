use thrushc_errors::CompilationIssue;
use thrushc_typesystem::traits::TypeIsExtensions;

use crate::{
    Ast,
    builitins::ThrushBuiltin,
    traits::{
        AstCodeBlockEntensions, AstConstantExtensions, AstGetType, AstMemoryExtensions,
        AstMutabilityExtensions, AstScopeExtensions, AstStandardExtensions, AstStatementExtentions,
    },
};

impl AstStandardExtensions for Ast<'_> {
    #[inline]
    fn is_literal_value(&self) -> bool {
        match self {
            Ast::Integer { .. }
            | Ast::Float { .. }
            | Ast::Boolean { .. }
            | Ast::Char { .. }
            | Ast::Str { .. }
            | Ast::NullPtr { .. } => true,

            Ast::FixedArray { items, .. } => items.iter().all(|item| item.is_literal_value()),
            Ast::Array { items, .. } => items.iter().all(|item| item.is_literal_value()),

            Ast::EnumValue { value, .. } => value.is_literal_value(),

            Ast::Group { expression, .. } => expression.is_literal_value(),
            Ast::BinaryOp { left, right, .. } => {
                left.is_literal_value() && right.is_literal_value()
            }
            Ast::UnaryOp { expression, .. } => expression.is_literal_value(),

            _ => false,
        }
    }

    #[inline]
    fn is_reference(&self) -> bool {
        matches!(self, Ast::Reference { .. })
    }

    #[inline]
    fn is_before_unary(&self) -> bool {
        matches!(self, Ast::UnaryOp { is_pre: true, .. })
    }

    #[inline]
    fn is_function(&self) -> bool {
        matches!(self, Ast::Function { .. })
    }

    #[inline]
    fn is_intrinsic(&self) -> bool {
        matches!(self, Ast::Intrinsic { .. })
    }

    #[inline]
    fn is_asm_function(&self) -> bool {
        matches!(self, Ast::AssemblerFunction { .. })
    }

    #[inline]
    fn is_struct(&self) -> bool {
        matches!(self, Ast::Struct { .. })
    }

    #[inline]
    fn is_enum(&self) -> bool {
        matches!(self, Ast::Enum { .. })
    }

    #[inline]
    fn is_str(&self) -> bool {
        matches!(self, Ast::Str { .. })
    }

    #[inline]
    fn is_constant(&self) -> bool {
        matches!(self, Ast::Const { .. })
    }

    #[inline]
    fn is_static(&self) -> bool {
        matches!(self, Ast::Static { .. })
    }

    #[inline]
    fn is_integer(&self) -> bool {
        matches!(self, Ast::Integer { .. })
    }

    #[inline]
    fn is_terminator(&self) -> bool {
        matches!(self, Ast::Return { .. })
    }

    #[inline]
    fn is_unreacheable(&self) -> bool {
        matches!(self, Ast::Unreachable { .. })
    }

    #[inline]
    fn is_break(&self) -> bool {
        matches!(self, Ast::Break { .. })
    }

    #[inline]
    fn is_continue(&self) -> bool {
        matches!(self, Ast::Continue { .. })
    }

    #[inline]
    fn is_custom_type(&self) -> bool {
        matches!(self, Ast::CustomType { .. })
    }

    #[inline]
    fn is_global_asm(&self) -> bool {
        matches!(self, Ast::GlobalAssembler { .. })
    }

    #[inline]
    fn is_import(&self) -> bool {
        matches!(self, Ast::Import { .. })
    }

    #[inline]
    fn is_conditional(&self) -> bool {
        matches!(self, Ast::If { .. } | Ast::Elif { .. } | Ast::Else { .. })
    }
}

impl AstStatementExtentions for Ast<'_> {
    fn is_statement(&self) -> bool {
        matches!(
            self,
            Ast::Block { .. }
                | Ast::If { .. }
                | Ast::Else { .. }
                | Ast::Elif { .. }
                | Ast::While { .. }
                | Ast::For { .. }
                | Ast::Loop { .. }
                | Ast::Return { .. }
                | Ast::Break { .. }
                | Ast::Continue { .. }
                | Ast::Local { .. }
                | Ast::Struct { .. }
                | Ast::Const { .. }
                | Ast::Static { .. }
        )
    }
}

impl AstCodeBlockEntensions for Ast<'_> {
    #[inline]
    fn is_empty_block(&self) -> bool {
        let Ast::Block { nodes, .. } = self else {
            return false;
        };

        nodes.is_empty()
    }

    #[inline]
    fn has_terminator(&self) -> bool {
        let Ast::Block { nodes, .. } = self else {
            return false;
        };

        {
            for node in nodes.iter() {
                if node.is_terminator() {
                    return true;
                }

                if let Ast::If {
                    block,
                    elseif,
                    anyway,
                    ..
                } = node
                {
                    let if_branch_returns: bool = block.has_terminator();

                    let all_elif_return: bool = elseif.iter().all(|elif_node| {
                        if let Ast::Elif { block, .. } = elif_node {
                            block.has_terminator()
                        } else {
                            false
                        }
                    });

                    let else_branch_returns: bool = anyway.as_ref().is_some_and(|otherwise| {
                        if let Ast::Else { block, .. } = &**otherwise {
                            block.has_terminator()
                        } else {
                            false
                        }
                    });

                    let if_else_returns: bool =
                        if_branch_returns && else_branch_returns && elseif.is_empty();
                    let full_returns: bool =
                        if_branch_returns && all_elif_return && else_branch_returns;

                    if if_else_returns || full_returns {
                        return true;
                    }
                }
            }
        }

        false
    }
}

impl AstMutabilityExtensions for Ast<'_> {
    #[inline]
    fn is_mutable(&self) -> bool {
        match self {
            Ast::Local { metadata, .. } => metadata.is_mutable(),
            Ast::FunctionParameter { metadata, .. } => metadata.is_mutable(),
            Ast::Index { metadata, .. } => metadata.is_mutable(),
            Ast::Reference { metadata, .. } => metadata.is_mutable(),
            Ast::Property { source, .. } => source.is_reference(),

            _ => false,
        }
    }
}

impl AstMemoryExtensions for Ast<'_> {
    #[inline]
    fn is_allocated(&self) -> bool {
        match self {
            Ast::Reference { metadata, .. } => metadata.is_allocated(),
            Ast::Property { metadata, .. } => metadata.is_allocated(),

            _ => false,
        }
    }

    #[inline]
    fn is_allocated_value(&self) -> Result<bool, CompilationIssue> {
        match self {
            Ast::Reference { metadata, .. } => Ok(metadata.is_allocated()),
            Ast::Property { metadata, .. } => Ok(metadata.is_allocated()),

            _ => Ok(self.get_value_type()?.is_ptr_like_type()),
        }
    }
}

impl AstConstantExtensions for Ast<'_> {
    fn is_constant_value(&self) -> bool {
        match self {
            Ast::Integer { .. }
            | Ast::Float { .. }
            | Ast::Boolean { .. }
            | Ast::Char { .. }
            | Ast::Str { .. }
            | Ast::NullPtr { .. }
            | Self::Builtin {
                builtin:
                    ThrushBuiltin::AlignOf { .. }
                    | ThrushBuiltin::SizeOf { .. }
                    | ThrushBuiltin::AbiSizeOf { .. }
                    | ThrushBuiltin::AbiAlignOf { .. }
                    | ThrushBuiltin::BitSizeOf { .. },
                ..
            } => true,
            Ast::EnumValue { value, .. } => value.is_constant_value(),
            Ast::DirectRef { expr, .. } => expr.is_constant_value(),
            Ast::Group { expression, .. } => expression.is_constant_value(),
            Ast::BinaryOp { left, right, .. } => {
                left.is_constant_value() && right.is_constant_value()
            }
            Ast::UnaryOp { expression, .. } => expression.is_constant_value(),
            Ast::Reference { metadata, .. } => metadata.is_constant(),
            Ast::As { metadata, .. } => metadata.is_constant(),
            Ast::FixedArray { items, .. } => items.iter().all(|item| item.is_constant_value()),
            Ast::Constructor { args, .. } => args.iter().all(|arg| arg.1.is_constant_value()),

            _ => false,
        }
    }
}

impl AstScopeExtensions for Ast<'_> {
    #[inline]
    fn is_compatible_with_main_scope(&self) -> bool {
        matches!(
            self,
            Ast::CustomType { .. }
                | Ast::Struct { .. }
                | Ast::Enum { .. }
                | Ast::Intrinsic { .. }
                | Ast::Function { .. }
                | Ast::AssemblerFunction { .. }
                | Ast::GlobalAssembler { .. }
                | Ast::Const { .. }
                | Ast::Static { .. }
                | Ast::Import { .. }
        )
    }
}

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
            Ast::Invalid { .. } => write!(f, "invalid ast"),
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
                if let Some(value) = value {
                    if metadata.is_mutable() {
                        write!(f, "let mut {} : {} = {}", name, kind, value)?;
                    } else {
                        write!(f, "let {} : {} = {}", name, kind, value)?;
                    }
                } else if metadata.is_mutable() {
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
