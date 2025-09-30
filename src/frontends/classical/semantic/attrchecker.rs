use ahash::AHashSet;

use crate::{
    core::{
        compiler::options::CompilationUnit,
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
        errors::standard::ThrushCompilerIssue,
    },
    frontends::classical::types::{
        ast::Ast,
        parser::stmts::{traits::ThrushAttributesExtensions, types::ThrushAttributes},
        semantic::{
            attrchecker::types::AttributeCheckerAttributeApplicant,
            linter::{traits::LLVMAttributeComparatorExtensions, types::LLVMAttributeComparator},
        },
    },
};

#[derive(Debug)]
pub struct AttributeChecker<'attr_checker> {
    ast: &'attr_checker [Ast<'attr_checker>],
    errors: Vec<ThrushCompilerIssue>,
    currrent: usize,
    dignostician: Diagnostician,
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    pub fn new(
        ast: &'attr_checker [Ast<'attr_checker>],
        file: &'attr_checker CompilationUnit,
    ) -> Self {
        Self {
            ast,
            errors: Vec::with_capacity(100),
            currrent: 0,
            dignostician: Diagnostician::new(file),
        }
    }

    pub fn check(&mut self) -> bool {
        while !self.is_eof() {
            let ast: &Ast = self.peek();

            self.analyze_ast(ast);

            self.advance();
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.dignostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            return true;
        }

        false
    }

    fn analyze_ast(&mut self, ast: &'attr_checker Ast) {
        /* ######################################################################


            TYPE CHECKER DECLARATIONS - START


        ########################################################################*/

        if let Ast::EntryPoint { body, .. } = ast {
            self.analyze_ast(body);
        }

        if let Ast::Function {
            attributes,
            body,
            span,
            ..
        } = ast
        {
            if body.is_some() && attributes.has_extern_attribute() {
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

            if let Some(body) = body {
                self.analyze_ast(body);
            }

            self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Function);
        }

        if let Ast::Struct { attributes, .. } = ast {
            self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Struct);
        }

        if let Ast::Enum { attributes, .. } = ast {
            self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Enum);
        }

        /* ######################################################################


            TYPE CHECKER DECLARATIONS - END


        ########################################################################*/

        /* ######################################################################


            TYPE CHECKER STATEMENTS - START


        ########################################################################*/

        if let Ast::Block { stmts, .. } = ast {
            stmts.iter().for_each(|stmt| {
                self.analyze_ast(stmt);
            });
        }

        if let Ast::Const {
            attributes,
            metadata,
            span,
            ..
        } = ast
        {
            if !metadata.is_global() && attributes.has_public_attribute() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Attribute error".into(),
                    "Local constant cannot have public visibility.".into(),
                    None,
                    *span,
                ));
            }

            self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Constant);
        }

        if let Ast::Static {
            attributes,
            metadata,
            span,
            ..
        } = ast
        {
            if !metadata.is_global() && attributes.has_public_attribute() {
                self.add_error(ThrushCompilerIssue::Error(
                    "Attribute error".into(),
                    "Local static cannot have public visibility.".into(),
                    None,
                    *span,
                ));
            }

            self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Static);
        }

        /* ######################################################################


            TYPE CHECKER STATEMENTS - END


        ########################################################################*/
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
                            "Attribute error".into(),
                            "The @ignore attribute requires the function to be annotated with @extern(\"something\").".into(),
                            None,
                            span,
                        ));
                    }
                }

                if attributes.has_inlinealways_attr() && attributes.has_inline_attr() {
                    if let Some(span) = attributes.match_attr(LLVMAttributeComparator::InlineHint) {
                        self.add_error(ThrushCompilerIssue::Error(
                            "Illogical attribute".into(),
                            "The attribute is not valid. Use either '@alwaysinline' or '@inline' attribute.".into(),
                            None,
                            span,
                        ));
                    }
                }

                if attributes.has_inline_attr() && attributes.has_noinline_attr() {
                    if let Some(span) = attributes.match_attr(LLVMAttributeComparator::NoInline) {
                        self.add_error(ThrushCompilerIssue::Error(
                            "Illogical attribute".into(),
                            "The attribute is not valid. Use either '@noinline' or '@inline' attribute.".into(),
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
            | AttributeCheckerAttributeApplicant::Static
            | AttributeCheckerAttributeApplicant::Struct
            | AttributeCheckerAttributeApplicant::Enum => {
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
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    #[inline]
    fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }

    #[inline]
    fn advance(&mut self) {
        if !self.is_eof() {
            self.currrent += 1;
        }
    }

    #[inline]
    fn peek(&self) -> &'attr_checker Ast<'attr_checker> {
        self.ast.get(self.currrent).unwrap_or_else(|| {
            logging::print_frontend_panic(
                LoggingType::FrontEndPanic,
                "Attemping to get a statement in invalid position at attribute checker.",
            );
        })
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.currrent >= self.ast.len()
    }
}
