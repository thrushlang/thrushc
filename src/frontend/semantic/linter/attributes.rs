use crate::{
    core::{
        compiler::options::CompilationUnit,
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
        errors::standard::ThrushCompilerIssue,
    },
    frontend::types::{
        ast::Ast,
        parser::stmts::{traits::ThrushAttributesExtensions, types::ThrushAttributes},
        semantic::linter::{
            traits::LLVMAttributeComparatorExtensions,
            types::{LLVMAttributeComparator, LinterAttributeApplicant},
        },
    },
};

#[derive(Debug)]
pub struct AttributesLinter<'attr_linter> {
    ast: &'attr_linter [Ast<'attr_linter>],
    warnings: Vec<ThrushCompilerIssue>,
    current: usize,
    dignostician: Diagnostician,
}

impl<'attr_linter> AttributesLinter<'attr_linter> {
    pub fn new(
        ast: &'attr_linter [Ast<'attr_linter>],
        file: &'attr_linter CompilationUnit,
    ) -> Self {
        Self {
            ast,
            warnings: Vec::with_capacity(100),
            current: 0,
            dignostician: Diagnostician::new(file),
        }
    }

    pub fn check(&mut self) {
        while !self.is_eof() {
            let current_stmt: &Ast = self.peek();

            self.analyze_stmt(current_stmt);

            self.advance();
        }

        self.warnings.iter().for_each(|warn: &ThrushCompilerIssue| {
            self.dignostician
                .dispatch_diagnostic(warn, LoggingType::Warning);
        });
    }

    fn analyze_stmt(&mut self, stmt: &'attr_linter Ast) {
        if let Ast::Function { attributes, .. } = stmt {
            self.analyze_attributes(attributes, LinterAttributeApplicant::Function);
        }

        if let Ast::Struct { attributes, .. } = stmt {
            self.analyze_attributes(attributes, LinterAttributeApplicant::Struct);
        }

        if let Ast::Const { attributes, .. } = stmt {
            self.analyze_attributes(attributes, LinterAttributeApplicant::Constant);
        }
    }

    fn analyze_attributes(
        &mut self,
        attributes: &ThrushAttributes,
        applicant: LinterAttributeApplicant,
    ) {
        match applicant {
            LinterAttributeApplicant::Function => {
                if attributes.has_inlinealways_attr()
                    && attributes.has_hot_attr()
                    && attributes.has_minsize_attr()
                {
                    if let Some(attr_span) = attributes.match_attr(LLVMAttributeComparator::Hot) {
                        self.add_warning(ThrushCompilerIssue::Warning(
                            String::from("Possible undefined behavior"),
                            String::from(
                                "Excessive optimization of a function or method can result in undefined behavior in specific scenarios or make it unsuitable for intended use. Exercise caution to ensure compatibility and stability.",
                            ),
                            attr_span,
                        ));
                    }
                }

                if attributes.has_inline_attr() && attributes.has_noinline_attr() {}
            }
            LinterAttributeApplicant::Constant | LinterAttributeApplicant::Struct => {
                if attributes.has_public_attribute() && attributes.len() > 1 || attributes.len() > 1
                {
                    let organized_contrary_attrs: ThrushAttributes =
                        self.get_contrary_attrs(attributes, LLVMAttributeComparator::Public);

                    organized_contrary_attrs.iter().for_each(|attr| {
                        self.add_warning(ThrushCompilerIssue::Warning(
                            String::from("Not applicable attribute"),
                            String::from("This attribute isn't applicable."),
                            attr.get_span(),
                        ));
                    });
                }
            }
        }
    }

    fn get_contrary_attrs(
        &self,
        attributes: &'attr_linter ThrushAttributes,
        point_attr: LLVMAttributeComparator,
    ) -> ThrushAttributes<'attr_linter> {
        attributes
            .iter()
            .filter_map(|attr| match attr {
                attr if attr.into_llvm_attr_cmp() != point_attr => Some(*attr),
                _ => None,
            })
            .collect()
    }
}

impl<'attr_linter> AttributesLinter<'attr_linter> {
    #[inline]
    fn peek(&self) -> &'attr_linter Ast<'attr_linter> {
        self.ast.get(self.current).unwrap_or_else(|| {
            logging::print_frontend_panic(
                LoggingType::FrontEndPanic,
                "Attemping to get a statement in invalid position at attributes linter.",
            );
        })
    }

    #[inline]
    fn add_warning(&mut self, warn: ThrushCompilerIssue) {
        self.warnings.push(warn);
    }

    #[inline]
    fn advance(&mut self) {
        if !self.is_eof() {
            self.current += 1;
        }
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.current >= self.ast.len()
    }
}
