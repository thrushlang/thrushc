/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

use thrustc_span::Span;

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

            refcounter = refcounter.saturating_add(1);

            *lhs_infered_type = Some((
                Type::FixedArray((*base_type).clone(), *size, span).into(),
                refcounter,
            ));
        }
    }

    #[inline(always)]
    fn has_inferer_inner_type(&self) -> bool {
        matches!(
            self,
            Type::Array {
                infered_type: Some(_),
                ..
            }
        )
    }

    #[inline(always)]
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

    #[inline(always)]
    fn is_inferer_inner_type_is_not_array_decay(&self) -> bool {
        if let Type::Array {
            infered_type: Some((_, 0..=1)),
            ..
        } = self
        {
            return true;
        }

        false
    }

    #[inline(always)]
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
