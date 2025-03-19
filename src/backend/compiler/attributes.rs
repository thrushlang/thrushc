use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    values::FunctionValue,
};

pub enum AttributeApplicant<'ctx> {
    Function(FunctionValue<'ctx>),
}

pub struct AttributesBuilder<'ctx> {
    context: &'ctx Context,
    attribute_applicant: AttributeApplicant<'ctx>,
}

impl<'ctx> AttributesBuilder<'ctx> {
    pub fn new(context: &'ctx Context, attribute_applicant: AttributeApplicant<'ctx>) -> Self {
        Self {
            context,
            attribute_applicant,
        }
    }

    pub fn set_speedup_mode(&self) {
        match self.attribute_applicant {
            AttributeApplicant::Function(function) => {
                self.generate_speedup_attributes()
                    .iter()
                    .for_each(|attribute| {
                        function.add_attribute(AttributeLoc::Function, *attribute);
                    });
            }
        }
    }

    fn generate_speedup_attributes(&self) -> [Attribute; 6] {
        let always_inline: Attribute = self
            .context
            .create_enum_attribute(Attribute::get_named_enum_kind_id("alwaysinline"), 0);

        let hot_zone: Attribute = self
            .context
            .create_enum_attribute(Attribute::get_named_enum_kind_id("hot"), 0);

        let norecurse: Attribute = self
            .context
            .create_enum_attribute(Attribute::get_named_enum_kind_id("norecurse"), 0);

        let nounwind: Attribute = self
            .context
            .create_enum_attribute(Attribute::get_named_enum_kind_id("nounwind"), 0);

        let optsize: Attribute = self
            .context
            .create_enum_attribute(Attribute::get_named_enum_kind_id("optsize"), 0);

        let safe_stack: Attribute = self
            .context
            .create_enum_attribute(Attribute::get_named_enum_kind_id("safestack"), 0);

        [
            always_inline,
            hot_zone,
            norecurse,
            nounwind,
            optsize,
            safe_stack,
        ]
    }
}
