use std::fmt::Display;

use crate::back_end::llvm::compiler::attributes::LLVMAttribute;
use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::{Ast, metadata::local::LocalMetadata};
use crate::front_end::types::semantic::linter::traits::LLVMAttributeComparatorExtensions;
use crate::front_end::types::semantic::linter::types::LLVMAttributeComparator;

use crate::front_end::typesystem::modificators::StructureTypeModificator;
use crate::front_end::typesystem::traits::TypeStructExtensions;
use crate::front_end::typesystem::types::Type;

use crate::front_end::types::parser::stmts::traits::{
    ConstructorExtensions, StructFieldsExtensions, ThrushAttributesExtensions,
};

use crate::front_end::types::parser::stmts::types::{Constructor, StructFields, ThrushAttributes};

impl ThrushAttributesExtensions for ThrushAttributes<'_> {
    #[inline]
    fn has_extern_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_extern_attribute())
    }

    #[inline]
    fn has_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }

    #[inline]
    fn has_heap_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_heap_attribute())
    }

    #[inline]
    fn has_public_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_public_attribute())
    }

    #[inline]
    fn has_hot_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_hot_attribute())
    }

    #[inline]
    fn has_inline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_inline_attribute())
    }

    #[inline]
    fn has_minsize_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_minsize_attribute())
    }

    #[inline]
    fn has_inlinealways_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_alwaysinline_attribute())
    }

    #[inline]
    fn has_noinline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_noinline_attribute())
    }

    #[inline]
    fn has_asmalignstack_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmalingstack_attribute())
    }

    #[inline]
    fn has_asmsideffects_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsideeffects_attribute())
    }

    #[inline]
    fn has_asmthrow_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmthrow_attribute())
    }

    #[inline]
    fn has_asmsyntax_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsyntax_attribute())
    }

    #[inline]
    fn match_attr(&self, cmp: LLVMAttributeComparator) -> Option<Span> {
        if let Some(attr_found) = self.iter().find(|attr| attr.into_llvm_attr_cmp() == cmp) {
            return Some(attr_found.get_span());
        }

        None
    }

    #[inline]
    fn get_attr(&self, cmp: LLVMAttributeComparator) -> Option<LLVMAttribute<'_>> {
        if let Some(attr_found) = self.iter().find(|attr| attr.into_llvm_attr_cmp() == cmp) {
            return Some(*attr_found);
        }

        None
    }
}

impl StructFieldsExtensions for StructFields<'_> {
    #[inline]
    fn get_type(&self) -> Type {
        let types: Vec<Type> = self.1.iter().map(|field| field.1.clone()).collect();
        Type::create_struct_type(self.0.to_string(), types.as_slice(), self.get_modificator())
    }

    #[inline]
    fn get_modificator(&self) -> StructureTypeModificator {
        self.2
    }
}

impl ConstructorExtensions for Constructor<'_> {
    #[inline]
    fn get_type(&self, name: &str, modificator: StructureTypeModificator) -> Type {
        let types: Vec<Type> = self.iter().map(|field| field.2.clone()).collect();
        Type::create_struct_type(name.to_string(), types.as_slice(), modificator)
    }
}

impl PartialEq for Ast<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ast::Integer { .. }, Ast::Integer { .. })
            | (Ast::Float { .. }, Ast::Float { .. })
            | (Ast::Str { .. }, Ast::Str { .. }) => true,
            (left, right) => std::mem::discriminant(left) == std::mem::discriminant(right),
        }
    }
}

impl Display for Ast<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Pass { .. } => write!(f, "pass"),
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
            Ast::Block { stmts, .. } => {
                let _ = write!(f, "{{ ");

                for stmt in stmts {
                    let _ = write!(f, "{}", stmt);
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
                cond,
                actions,
                block,
                ..
            } => {
                write!(f, "for {} {} {} {}", local, cond, actions, block)
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

            Ast::While { cond, block, .. } => {
                write!(f, "while {} {}", cond, block)
            }

            Ast::EntryPoint { body, .. } => {
                write!(f, "fn main() {}", body)
            }

            Ast::NullPtr { .. } => {
                write!(f, "nullptr")
            }

            Ast::Group { expression, .. } => {
                write!(f, "({})", expression)
            }

            _ => Ok(()),
        }
    }
}
