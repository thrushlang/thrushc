use thrustc_typesystem::Type;

use crate::{Position, SynchronizationPosition};

pub trait ControlContextExtensions {
    fn set_position(&mut self, position: Position);
    fn reset_position(&mut self);
    fn get_position(&self) -> Position;

    fn add_sync_position(&mut self, other: SynchronizationPosition);
    fn pop_sync_position(&mut self);
    fn reset_sync_position(&mut self);
    fn get_sync_position(&self) -> Option<&SynchronizationPosition>;

    fn increase_expression_depth(&mut self);
    fn decrease_expression_depth(&mut self);
    fn get_expression_depth(&self) -> u32;
}

pub trait TypeContextExtensions {
    fn get_infered_type(&self) -> Option<Type>;
    fn add_infered_type(&mut self, t: Type);
    fn pop_infered_type(&mut self);
    fn reset_infered_types(&mut self);
}

pub trait PositionExtensions {
    fn is_constant_position(&self) -> bool;
    fn is_static_position(&self) -> bool;
    fn is_variable_position(&self) -> bool;
    fn is_expression_position(&self) -> bool;
    fn is_irrelevant_position(&self) -> bool;
}
