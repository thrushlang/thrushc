use std::fmt::Display;

use crate::frontend::{
    lexer::span::Span,
    types::{
        lexer::ThrushType,
        linter::{traits::LLVMAttributeComparatorExtensions, types::LLVMAttributeComparator},
    },
};

use super::{
    stmt::ThrushStatement,
    traits::{
        ConstructorExtensions, CustomTypeFieldsExtensions, StructFieldsExtensions,
        ThrushAttributesExtensions,
    },
    types::{Constructor, CustomTypeFields, StructFields, ThrushAttributes},
};

impl ThrushAttributesExtensions for ThrushAttributes<'_> {
    fn has_extern_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_extern_attribute())
    }

    fn has_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }

    fn has_heap_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_heap_attribute())
    }

    fn has_stack_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_stack_attribute())
    }

    fn has_public_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_public_attribute())
    }

    fn has_hot_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_hot_attribute())
    }

    fn has_inline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_inline_attribute())
    }

    fn has_minsize_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_minsize_attribute())
    }

    fn has_inlinealways_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_alwaysinline_attribute())
    }

    fn has_noinline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_noinline_attribute())
    }

    fn has_asmalignstack_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmalingstack_attribute())
    }

    fn has_asmsideffects_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsideeffects_attribute())
    }

    fn has_asmthrow_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmthrow_attribute())
    }

    fn match_attr(&self, cmp: LLVMAttributeComparator) -> Option<Span> {
        if let Some(attr_found) = self.iter().find(|attr| attr.into_llvm_attr_cmp() == cmp) {
            return Some(attr_found.get_span());
        }

        None
    }
}

impl StructFieldsExtensions for StructFields<'_> {
    fn get_type(&self) -> ThrushType {
        let types: Vec<ThrushType> = self.1.iter().map(|field| field.1.clone()).collect();
        ThrushType::create_structure_type(self.0.to_string(), types.as_slice())
    }
}

impl ConstructorExtensions for Constructor<'_> {
    fn get_type(&self) -> ThrushType {
        let types: Vec<ThrushType> = self.1.iter().map(|field| field.2.clone()).collect();
        ThrushType::create_structure_type(self.0.to_string(), types.as_slice())
    }
}

impl CustomTypeFieldsExtensions for CustomTypeFields<'_> {
    fn get_type(&self) -> ThrushType {
        ThrushType::create_structure_type(String::new(), self)
    }
}

impl PartialEq for ThrushStatement<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ThrushStatement::Integer { .. }, ThrushStatement::Integer { .. })
            | (ThrushStatement::Float { .. }, ThrushStatement::Float { .. })
            | (ThrushStatement::Str { .. }, ThrushStatement::Str { .. }) => true,
            (left, right) => std::mem::discriminant(left) == std::mem::discriminant(right),
        }
    }
}

impl Display for ThrushStatement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThrushStatement::Null { .. } => write!(f, "null"),
            ThrushStatement::Pass { .. } => write!(f, "pass"),
            ThrushStatement::Char { byte, .. } => write!(f, "{}", byte),
            ThrushStatement::Integer { value, .. } => write!(f, "{}", value),
            ThrushStatement::Float { value, .. } => write!(f, "{}", value),
            ThrushStatement::Boolean { value, .. } => write!(f, "{}", value),
            ThrushStatement::Str { bytes, .. } => {
                write!(f, "\"{}\"", String::from_utf8_lossy(bytes))
            }
            ThrushStatement::Function {
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

                if body.is_block() {
                    write!(f, "{}", body)?;
                }

                Ok(())
            }
            ThrushStatement::Block { stmts, .. } => {
                let _ = write!(f, "{{ ");

                for stmt in stmts {
                    let _ = write!(f, "{}", stmt);
                }

                let _ = write!(f, " }}");

                Ok(())
            }
            ThrushStatement::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                write!(f, "{} {} {}", left, operator, right)
            }
            ThrushStatement::UnaryOp {
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
            ThrushStatement::Break { .. } => {
                write!(f, "break")
            }
            ThrushStatement::Continue { .. } => {
                write!(f, "continue")
            }
            ThrushStatement::For {
                local,
                cond,
                actions,
                block,
                ..
            } => {
                write!(f, "for {} {} {} {}", local, cond, actions, block)
            }
            ThrushStatement::Call { name, args, .. } => {
                write!(f, "{}(", name)?;

                for (index, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;

                    if index > 0 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }

            ThrushStatement::If {
                cond,
                block,
                elfs,
                otherwise,
                ..
            } => {
                write!(f, "if {} {}", cond, block)?;

                for elif in elfs {
                    write!(f, " elif {}", elif)?;
                }

                if let Some(otherwise) = otherwise {
                    write!(f, " else {}", otherwise)?;
                }

                Ok(())
            }

            ThrushStatement::Return { expression, .. } => {
                if let Some(expr) = expression {
                    write!(f, "return {}", expr)?;
                }

                write!(f, "return")
            }

            ThrushStatement::Local {
                name,
                kind,
                value,
                is_mutable,
                ..
            } => {
                if *is_mutable {
                    write!(f, "let mut {} : {} = {}", name, kind, value)
                } else {
                    write!(f, "let {} : {} = {}", name, kind, value)
                }
            }

            ThrushStatement::Mut { source, value, .. } => {
                if let (Some(reference), _) = source {
                    write!(
                        f,
                        "{} = {}",
                        reference.get_unwrapped_reference_name(),
                        value
                    )?;
                }

                if let (_, Some(expr)) = source {
                    write!(f, "{} = {}", expr, value)?;
                }

                Ok(())
            }

            ThrushStatement::Reference { name, .. } => {
                write!(f, "{}", name)
            }

            ThrushStatement::Loop { block, .. } => {
                write!(f, "loop {}", block)
            }

            ThrushStatement::While { cond, block, .. } => {
                write!(f, "while {} {}", cond, block)
            }

            ThrushStatement::Method {
                name,
                parameters,
                body,
                return_type,
                ..
            } => {
                write!(f, "def {}(", name)?;

                for (index, param) in parameters.iter().enumerate() {
                    write!(f, "{}", param)?;

                    if index > 0 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ") ")?;
                write!(f, "{} ", return_type)?;
                write!(f, "{}", body)?;

                Ok(())
            }

            ThrushStatement::EntryPoint { body, .. } => {
                write!(f, "fn main() {}", body)
            }

            ThrushStatement::NullPtr { .. } => {
                write!(f, "null")
            }

            ThrushStatement::Group { expression, .. } => {
                write!(f, "({})", expression)
            }

            _ => Ok(()),
        }
    }
}
