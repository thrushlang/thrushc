use thrushc_span::Span;

use crate::{
    Type,
    traits::{InfererTypeExtensions, TypeCodeLocation, TypeIsExtensions},
};

impl InfererTypeExtensions for Type {
    fn inferer_inner_type_from_type(&mut self, other: &Type) {
        let span: Span = self.get_span();

        if let (
            Type::Array {
                base_type,
                infered_type: lhs_infered_type,
                ..
            },
            Type::Array {
                infered_type: Some(rhs_infered_type),
                ..
            },
        ) = (self, other)
        {
            let (Type::FixedArray(_, size, ..), mut refcounter) =
                (&*rhs_infered_type.0, rhs_infered_type.1)
            else {
                return;
            };

            refcounter += 1;

            *lhs_infered_type = Some((
                Type::FixedArray((*base_type).clone(), *size, span).into(),
                refcounter,
            ));
        }
    }

    fn has_inferer_inner_type(&self) -> bool {
        matches!(
            self,
            Type::Array {
                infered_type: Some(_),
                ..
            }
        )
    }

    fn is_inferer_inner_type_valid(&self) -> bool {
        if let Type::Array {
            infered_type: Some((infered_type, 0 | 1)),
            ..
        } = self
        {
            return infered_type.is_fixed_array_type();
        }

        false
    }

    fn get_inferer_inner_type(&self) -> Type {
        match self {
            Type::Array {
                infered_type: Some((infered_type, 0 | 1)),
                ..
            } => (**infered_type).clone(),

            _ => self.clone(),
        }
    }
}
