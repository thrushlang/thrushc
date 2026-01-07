use thrushc_span::Span;
use thrushc_typesystem::Type;

#[derive(Debug)]
pub struct TypeCheckerTypeContext<'type_checker> {
    current_function_type: Option<(&'type_checker Type, Span)>,
}

impl<'type_checker> TypeCheckerTypeContext<'type_checker> {
    #[inline]
    pub fn new() -> Self {
        Self {
            current_function_type: None,
        }
    }
}

impl<'type_checker> TypeCheckerTypeContext<'type_checker> {
    #[inline]
    pub fn set_current_function_type(&mut self, function_type: (&'type_checker Type, Span)) {
        self.current_function_type = Some(function_type);
    }

    #[inline]
    pub fn unset_current_function_type(&mut self) {
        self.current_function_type = None;
    }
}

impl<'type_checker> TypeCheckerTypeContext<'type_checker> {
    pub fn get_current_function_type(&self) -> Option<(&'type_checker Type, Span)> {
        self.current_function_type
    }
}
