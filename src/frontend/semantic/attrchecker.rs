use ahash::AHashSet;

use crate::{
    core::{
        compiler::options::CompilerFile,
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
        errors::standard::ThrushCompilerIssue,
    },
    frontend::types::{
        parser::stmts::{
            stmt::ThrushStatement, traits::ThrushAttributesExtensions, types::ThrushAttributes,
        },
        semantic::{
            attrchecker::types::AttributeCheckerAttributeApplicant,
            linter::{traits::LLVMAttributeComparatorExtensions, types::LLVMAttributeComparator},
        },
    },
};

#[derive(Debug)]
pub struct AttributeChecker<'attr_checker> {
    stmts: &'attr_checker [ThrushStatement<'attr_checker>],
    errors: Vec<ThrushCompilerIssue>,
    currrent: usize,
    dignostician: Diagnostician,
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    pub fn new(
        stmts: &'attr_checker [ThrushStatement<'attr_checker>],
        file: &'attr_checker CompilerFile,
    ) -> Self {
        Self {
            stmts,
            errors: Vec::with_capacity(100),
            currrent: 0,
            dignostician: Diagnostician::new(file),
        }
    }

    pub fn check(&mut self) -> bool {
        while !self.is_eof() {
            let current_stmt: &ThrushStatement = self.peek();

            self.analyze_stmt(current_stmt);

            self.advance();
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.dignostician
                    .build_diagnostic(error, LoggingType::Error);
            });

            return true;
        }

        false
    }

    fn analyze_stmt(&mut self, stmt: &'attr_checker ThrushStatement) {
        if let ThrushStatement::Function {
            attributes,
            body,
            span,
            ..
        } = stmt
        {
            if !body.is_null() && attributes.has_extern_attribute() {
                if let Some(span) = attributes.match_attr(LLVMAttributeComparator::Extern) {
                    self.add_error(ThrushCompilerIssue::Error(
                        "Attribute error".into(),
                        "External functions cannot have a body.".into(),
                        None,
                        span,
                    ));
                } else {
                    self.add_error(ThrushCompilerIssue::Error(
                        "Attribute error".into(),
                        "External functions cannot have a body.".into(),
                        None,
                        *span,
                    ));
                }
            }

            self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Function);
        }

        if let ThrushStatement::Struct { attributes, .. } = stmt {
            self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Struct);
        }

        if let ThrushStatement::Const { attributes, .. } = stmt {
            self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Constant);
        }
    }

    fn analyze_attrs(
        &mut self,
        attributes: &'attr_checker ThrushAttributes,
        applicant: AttributeCheckerAttributeApplicant,
    ) {
        match applicant {
            AttributeCheckerAttributeApplicant::Function => {
                let repeated_attrs: ThrushAttributes = self.get_repeated_attrs(attributes);

                repeated_attrs.iter().for_each(|attr| {
                    self.add_error(ThrushCompilerIssue::Error(
                        "Repeated attribute".into(),
                        "Repetitive attributes are disallowed.".into(),
                        None,
                        attr.get_span(),
                    ));
                });

                if attributes.has_extern_attribute() && !attributes.has_public_attribute() {
                    if let Some(span) = attributes.match_attr(LLVMAttributeComparator::Extern) {
                        self.add_error(ThrushCompilerIssue::Error(
                            "Missing attribute".into(),
                            "External functions always have public visibility. Add the '@public' attribute.".into(),
                            None,
                            span,
                        ));
                    }
                }

                if !attributes.has_extern_attribute() && attributes.has_ignore_attribute() {
                    if let Some(span) = attributes.match_attr(LLVMAttributeComparator::Ignore) {
                        self.add_error(ThrushCompilerIssue::Error(
                            String::from("Attribute error"),
                            String::from("The @ignore attribute requires the function to be annotated with @extern(\"something\")."),
                            None,
                            span,
                        ));
                    }
                }

                if attributes.has_inlinealways_attr() && attributes.has_inline_attr() {
                    if let Some(span) = attributes.match_attr(LLVMAttributeComparator::InlineHint) {
                        self.add_error(ThrushCompilerIssue::Error(
                            String::from("Illogical attribute"),
                            String::from(
                                "The attribute is not valid. Use either '@alwaysinline' or '@inline' attribute.",
                            ),
                            None,
                            span,
                        ));
                    }
                }

                if attributes.has_inline_attr() && attributes.has_noinline_attr() {
                    if let Some(span) = attributes.match_attr(LLVMAttributeComparator::NoInline) {
                        self.add_error(ThrushCompilerIssue::Error(
                            String::from("Illogical attribute"),
                            String::from(
                                "The attribute is not valid. Use either '@noinline' or '@inline' attribute.",
                            ),
                            None,
                            span,
                        ));
                    }
                }

                if attributes.has_inlinealways_attr() && attributes.has_noinline_attr() {
                    if let Some(span) = attributes.match_attr(LLVMAttributeComparator::NoInline) {
                        self.add_error(ThrushCompilerIssue::Error(
                            String::from("Illogical attribute"),
                            String::from(
                                "The attribute is not valid. Use either '@alwaysinline' or '@inline' attribute.",
                            ),
                            None,
                            span,
                        ));
                    }
                }
            }

            AttributeCheckerAttributeApplicant::Constant
            | AttributeCheckerAttributeApplicant::Struct => {
                let repeated_attrs: ThrushAttributes = self.get_repeated_attrs(attributes);

                repeated_attrs.iter().for_each(|attr| {
                    self.add_error(ThrushCompilerIssue::Error(
                        String::from("Repeated attribute"),
                        String::from("Repetitive attributes are disallowed."),
                        None,
                        attr.get_span(),
                    ));
                });
            }
        }
    }

    fn get_repeated_attrs(
        &self,
        attributes: &'attr_checker ThrushAttributes,
    ) -> ThrushAttributes<'attr_checker> {
        let mut storage: AHashSet<LLVMAttributeComparator> = AHashSet::with_capacity(20);
        let mut repeated_attrs: ThrushAttributes = Vec::with_capacity(20);

        attributes.iter().for_each(|attr| {
            if !storage.insert(attr.into_llvm_attr_cmp()) {
                repeated_attrs.push(*attr);
            }
        });

        repeated_attrs
    }

    fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }

    fn advance(&mut self) {
        if !self.is_eof() {
            self.currrent += 1;
        }
    }

    fn peek(&self) -> &'attr_checker ThrushStatement<'attr_checker> {
        self.stmts.get(self.currrent).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                "Attemping to get a statement in invalid position at Attribute Checker.",
            );

            unreachable!()
        })
    }

    fn is_eof(&self) -> bool {
        self.currrent >= self.stmts.len()
    }
}
