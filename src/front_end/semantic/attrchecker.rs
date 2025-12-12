use crate::core::compiler::options::CompilationUnit;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::types::ast::Ast;
use crate::front_end::types::semantic::attrchecker::types::AttributeCheckerAttributeApplicant;
use crate::middle_end::mir::attributes::traits::{
    ThrushAttributeComparatorExtensions, ThrushAttributesExtensions,
};
use crate::middle_end::mir::attributes::{
    ThrushAttribute, ThrushAttributeComparator, ThrushAttributes,
};

use ahash::AHashSet;

#[derive(Debug)]
pub struct AttributeChecker<'attr_checker> {
    ast: &'attr_checker [Ast<'attr_checker>],

    errors: Vec<CompilationIssue>,
    warnings: Vec<CompilationIssue>,

    currrent: usize,

    diagnostician: Diagnostician,
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    #[inline]
    pub fn new(
        ast: &'attr_checker [Ast<'attr_checker>],
        file: &'attr_checker CompilationUnit,
    ) -> Self {
        Self {
            ast,
            errors: Vec::with_capacity(100),
            warnings: Vec::with_capacity(100),

            currrent: 0,

            diagnostician: Diagnostician::new(file),
        }
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    pub fn start(&mut self) -> bool {
        while !self.is_eof() {
            let ast: &Ast = self.peek();

            self.analyze_ast(ast);

            self.advance();
        }

        self.check()
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    fn check(&mut self) -> bool {
        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            return true;
        }

        if !self.warnings.is_empty() {
            self.warnings.iter().for_each(|warning| {
                self.diagnostician
                    .dispatch_diagnostic(warning, LoggingType::Warning);
            });
        }

        false
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    fn analyze_ast(&mut self, ast: &'attr_checker Ast) {
        match ast {
            Ast::Function {
                attributes,
                body,
                span,
                ..
            } => {
                if body.is_some() && attributes.has_extern_attribute() {
                    if let Some(span) = attributes.match_attr(ThrushAttributeComparator::Extern) {
                        self.add_error(CompilationIssue::Error(
                            "Attribute error".into(),
                            "External functions cannot have a body. Remove it.".into(),
                            None,
                            span,
                        ));
                    }
                }

                if body.is_none() && !attributes.has_extern_attribute() {
                    self.add_error(CompilationIssue::Error(
                        "Missing error".into(),
                        "A function without body always need the external attribute. Add the '@extern' attribute.".into(),
                        None,
                        *span,
                    ));
                }

                if let Some(body) = body {
                    self.analyze_ast(body);
                }

                self.analyze_attrs(
                    attributes,
                    AttributeCheckerAttributeApplicant::Function,
                    *span,
                );
            }
            Ast::Intrinsic {
                attributes, span, ..
            } => {
                self.analyze_attrs(
                    attributes,
                    AttributeCheckerAttributeApplicant::Intrinsic,
                    *span,
                );
            }
            Ast::AssemblerFunction {
                attributes, span, ..
            } => {
                self.analyze_attrs(
                    attributes,
                    AttributeCheckerAttributeApplicant::AssemblerFunction,
                    *span,
                );
            }
            Ast::Struct {
                attributes, span, ..
            } => {
                self.analyze_attrs(
                    attributes,
                    AttributeCheckerAttributeApplicant::Struct,
                    *span,
                );
            }
            Ast::Enum {
                attributes, span, ..
            } => {
                self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Enum, *span);
            }
            Ast::Const {
                attributes,
                metadata,
                span,
                ..
            } => {
                if !metadata.is_global() && attributes.has_public_attribute() {
                    self.add_error(CompilationIssue::Error(
                        "Attribute error".into(),
                        "Local constant cannot have public visibility.".into(),
                        None,
                        *span,
                    ));
                }

                self.analyze_attrs(
                    attributes,
                    AttributeCheckerAttributeApplicant::Constant,
                    *span,
                );
            }
            Ast::Static {
                attributes,
                metadata,
                span,
                ..
            } => {
                if !metadata.is_global() && attributes.has_public_attribute() {
                    self.add_error(CompilationIssue::Error(
                        "Attribute error".into(),
                        "Local static cannot have public visibility.".into(),
                        None,
                        *span,
                    ));
                }

                self.analyze_attrs(
                    attributes,
                    AttributeCheckerAttributeApplicant::Static,
                    *span,
                );
            }
            Ast::Local {
                attributes, span, ..
            } => {
                self.analyze_attrs(attributes, AttributeCheckerAttributeApplicant::Local, *span);
            }
            Ast::Block { nodes, .. } => {
                nodes.iter().for_each(|node| {
                    self.analyze_ast(node);
                });
            }

            _ => (),
        }
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    fn analyze_attrs(
        &mut self,
        attributes: &'attr_checker ThrushAttributes,
        applicant: AttributeCheckerAttributeApplicant,
        span: Span,
    ) {
        match applicant {
            AttributeCheckerAttributeApplicant::Function => {
                self.check_irrelevant_attributes(attributes, applicant);
                self.check_illogical_attributes(attributes);

                self.get_repeated_attrs(attributes).iter().for_each(|attr| {
                    self.add_error(CompilationIssue::Error(
                        "Repeated attribute".into(),
                        "Repetitive attributes are disallowed.".into(),
                        None,
                        attr.get_span(),
                    ));
                });
            }

            AttributeCheckerAttributeApplicant::Intrinsic => {
                self.check_irrelevant_attributes(attributes, applicant);
                self.check_illogical_attributes(attributes);

                if !attributes.has_public_attribute() {
                    self.add_error(CompilationIssue::Error(
                        "Attribute error".into(),
                        "Intrinsic qualities should always have public visibility.".into(),
                        None,
                        span,
                    ));
                }

                self.get_repeated_attrs(attributes).iter().for_each(|attr| {
                    self.add_error(CompilationIssue::Error(
                        "Repeated attribute".into(),
                        "Repetitive attributes are disallowed.".into(),
                        None,
                        attr.get_span(),
                    ));
                });
            }

            AttributeCheckerAttributeApplicant::Static => {
                self.check_irrelevant_attributes(attributes, applicant);
                self.check_illogical_attributes(attributes);

                self.get_repeated_attrs(attributes).iter().for_each(|attr| {
                    self.add_error(CompilationIssue::Error(
                        "Repeated attribute".into(),
                        "Repetitive attributes are disallowed.".into(),
                        None,
                        attr.get_span(),
                    ));
                });
            }

            AttributeCheckerAttributeApplicant::AssemblerFunction => {
                self.check_irrelevant_attributes(attributes, applicant);
                self.check_illogical_attributes(attributes);

                if !attributes.has_asmsyntax_attribute() {
                    if let Some(span) = attributes.match_attr(ThrushAttributeComparator::Extern) {
                        self.add_error(CompilationIssue::Error(
                            "Missing attribute".into(),
                            "A pure assembler function always have syntax mode. Add the '@asmsyntax' attribute.".into(),
                            None,
                            span,
                        ));
                    }
                }

                if let Some(ThrushAttribute::AsmSyntax(syntax, span)) =
                    attributes.get_attr(ThrushAttributeComparator::AsmSyntax)
                {
                    if !crate::middle_end::mir::attributes::assembler::INLINE_ASSEMBLER_SYNTAXES
                        .contains(&syntax.as_str())
                    {
                        self.add_error(CompilationIssue::Error(
                            "Invalid attribute syntax".into(),
                            format!("Expected a valid assembler syntax, got '{}'.", syntax),
                            None,
                            span,
                        ));
                    }
                }

                if let Some(ThrushAttribute::Convention(convention, span)) =
                    attributes.get_attr(ThrushAttributeComparator::Convention)
                {
                    if !crate::middle_end::mir::attributes::callconventions::CALL_CONVENTIONS
                        .contains_key(convention.as_bytes())
                    {
                        self.add_warning(CompilationIssue::Warning(
                            "Invalid attribute syntax".into(),
                            "Unknown calling convention, setting C by default.".into(),
                            span,
                        ));
                    }
                }

                self.get_repeated_attrs(attributes).iter().for_each(|attr| {
                    self.add_error(CompilationIssue::Error(
                        "Repeated attribute".into(),
                        "Repetitive attributes are disallowed.".into(),
                        None,
                        attr.get_span(),
                    ));
                });
            }

            AttributeCheckerAttributeApplicant::Constant
            | AttributeCheckerAttributeApplicant::Struct
            | AttributeCheckerAttributeApplicant::Enum
            | AttributeCheckerAttributeApplicant::Local => {
                self.check_irrelevant_attributes(attributes, applicant);
                self.check_illogical_attributes(attributes);

                self.get_repeated_attrs(attributes).iter().for_each(|attr| {
                    self.add_error(CompilationIssue::Error(
                        "Repeated attribute".into(),
                        "Repetitive attributes are disallowed.".into(),
                        None,
                        attr.get_span(),
                    ));
                });
            }
        }
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    fn check_irrelevant_attributes(
        &mut self,
        attributes: &ThrushAttributes,
        applicant: AttributeCheckerAttributeApplicant,
    ) {
        const VALID_FUNCTION_ATTRIBUTES: &[ThrushAttributeComparator] = &[
            ThrushAttributeComparator::AlwaysInline,
            ThrushAttributeComparator::InlineHint,
            ThrushAttributeComparator::NoInline,
            ThrushAttributeComparator::Convention,
            ThrushAttributeComparator::Extern,
            ThrushAttributeComparator::Ignore,
            ThrushAttributeComparator::Public,
            ThrushAttributeComparator::Hot,
            ThrushAttributeComparator::NoUnwind,
            ThrushAttributeComparator::OptFuzzing,
            ThrushAttributeComparator::MinSize,
            ThrushAttributeComparator::WeakStack,
            ThrushAttributeComparator::StrongStack,
            ThrushAttributeComparator::PreciseFloats,
            ThrushAttributeComparator::Linkage,
        ];

        const VALID_INTRINSIC_ATTRIBUTES: &[ThrushAttributeComparator] = &[
            ThrushAttributeComparator::AlwaysInline,
            ThrushAttributeComparator::InlineHint,
            ThrushAttributeComparator::NoInline,
            ThrushAttributeComparator::Convention,
            ThrushAttributeComparator::Extern,
            ThrushAttributeComparator::Ignore,
            ThrushAttributeComparator::Public,
            ThrushAttributeComparator::Hot,
            ThrushAttributeComparator::NoUnwind,
            ThrushAttributeComparator::OptFuzzing,
            ThrushAttributeComparator::MinSize,
            ThrushAttributeComparator::WeakStack,
            ThrushAttributeComparator::StrongStack,
            ThrushAttributeComparator::PreciseFloats,
            ThrushAttributeComparator::Linkage,
        ];

        const VALID_ASSEMBLER_FUNCTION_ATTRIBUTES: &[ThrushAttributeComparator] = &[
            ThrushAttributeComparator::AlwaysInline,
            ThrushAttributeComparator::InlineHint,
            ThrushAttributeComparator::NoInline,
            ThrushAttributeComparator::Convention,
            ThrushAttributeComparator::Ignore,
            ThrushAttributeComparator::Public,
            ThrushAttributeComparator::Hot,
            ThrushAttributeComparator::NoUnwind,
            ThrushAttributeComparator::OptFuzzing,
            ThrushAttributeComparator::MinSize,
            ThrushAttributeComparator::WeakStack,
            ThrushAttributeComparator::StrongStack,
            ThrushAttributeComparator::PreciseFloats,
            ThrushAttributeComparator::Linkage,
            ThrushAttributeComparator::AsmAlignStack,
            ThrushAttributeComparator::AsmSyntax,
            ThrushAttributeComparator::AsmSideEffects,
            ThrushAttributeComparator::AsmThrow,
        ];

        const VALID_STATIC_ATTRIBUTES: &[ThrushAttributeComparator] = &[
            ThrushAttributeComparator::Public,
            ThrushAttributeComparator::Extern,
            ThrushAttributeComparator::Linkage,
        ];

        const VALID_CONSTANT_ATTRIBUTES: &[ThrushAttributeComparator] = &[
            ThrushAttributeComparator::Public,
            ThrushAttributeComparator::Extern,
            ThrushAttributeComparator::Linkage,
        ];

        const VALID_ENUM_ATTRIBUTES: &[ThrushAttributeComparator] =
            &[ThrushAttributeComparator::Public];

        const VALID_STRUCTS_ATTRIBUTES: &[ThrushAttributeComparator] = &[
            ThrushAttributeComparator::Public,
            ThrushAttributeComparator::Packed,
        ];

        const VALID_LOCAL_ATTRIBUTES: &[ThrushAttributeComparator] =
            &[ThrushAttributeComparator::Heap];

        match applicant {
            AttributeCheckerAttributeApplicant::Function => {
                attributes.iter().for_each(|attr| {
                    if !VALID_FUNCTION_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            "Irrelevant attribute".into(),
                            "This attribute is not applicable for functions.".into(),
                            attr.get_span(),
                        ));
                    }
                });
            }
            AttributeCheckerAttributeApplicant::Intrinsic => {
                attributes.iter().for_each(|attr| {
                    if !VALID_INTRINSIC_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            "Irrelevant attribute".into(),
                            "This attribute is not applicable for a intrinsic.".into(),
                            attr.get_span(),
                        ));
                    }
                });
            }
            AttributeCheckerAttributeApplicant::Constant => {
                attributes.iter().for_each(|attr| {
                    if !VALID_CONSTANT_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            "Irrelevant attribute".into(),
                            "This attribute is not applicable for constants.".into(),
                            attr.get_span(),
                        ));
                    }
                });
            }
            AttributeCheckerAttributeApplicant::AssemblerFunction => {
                attributes.iter().for_each(|attr| {
                    if !VALID_ASSEMBLER_FUNCTION_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            "Irrelevant attribute".into(),
                            "This attribute is not applicable for assembler functions.".into(),
                            attr.get_span(),
                        ));
                    }
                });
            }
            AttributeCheckerAttributeApplicant::Enum => {
                attributes.iter().for_each(|attr| {
                    if !VALID_ENUM_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            "Irrelevant attribute".into(),
                            "This attribute is not applicable for enumerations.".into(),
                            attr.get_span(),
                        ));
                    }
                });
            }
            AttributeCheckerAttributeApplicant::Static => {
                attributes.iter().for_each(|attr| {
                    if !VALID_STATIC_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            "Irrelevant attribute".into(),
                            "This attribute is not applicable for static symbols.".into(),
                            attr.get_span(),
                        ));
                    }
                });
            }
            AttributeCheckerAttributeApplicant::Struct => {
                attributes.iter().for_each(|attr| {
                    if !VALID_STRUCTS_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            "Irrelevant attribute".into(),
                            "This attribute is not applicable for structures.".into(),
                            attr.get_span(),
                        ));
                    }
                });
            }
            AttributeCheckerAttributeApplicant::Local => {
                attributes.iter().for_each(|attr| {
                    if !VALID_LOCAL_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            "Irrelevant attribute".into(),
                            "This attribute is not applicable for local variable.".into(),
                            attr.get_span(),
                        ));
                    }
                });
            }
        }
    }

    fn check_illogical_attributes(&mut self, attributes: &ThrushAttributes) {
        if attributes.has_extern_attribute() && !attributes.has_public_attribute() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::Extern) {
                self.add_error(CompilationIssue::Error(
                    "Missing attribute".into(),
                    "A external symbol always have public visibility. Add the '@public' attribute."
                        .into(),
                    None,
                    span,
                ));
            }
        }

        if attributes.has_convention_attribute() {
            if let Some(ThrushAttribute::Convention(conv, span)) =
                attributes.get_attr(ThrushAttributeComparator::Convention)
            {
                if !crate::middle_end::mir::attributes::callconventions::CALL_CONVENTIONS
                    .contains_key(&conv.as_bytes())
                {
                    self.add_warning(CompilationIssue::Warning(
                        "Unknown call convention".into(),
                        "Unknown call convention, assuming C standard call convention by default."
                            .into(),
                        span,
                    ));
                }
            }
        }

        if attributes.has_linkage_attribute() {
            if let Some(ThrushAttribute::Linkage(linkage, linkage_raw, span)) =
                attributes.get_attr(ThrushAttributeComparator::Linkage)
            {
                if !crate::middle_end::mir::attributes::linkage::LINKAGES
                    .contains(&linkage_raw.as_str())
                {
                    self.add_warning(CompilationIssue::Warning(
                        "Unknown linkage".into(),
                        "Unknown linking, assuming non-proprietary C (External) standard.".into(),
                        span,
                    ));
                }

                if !attributes.has_public_attribute()
                    && (linkage.is_linker_private() || linkage.is_linker_private_weak())
                {
                    self.add_warning(CompilationIssue::Warning(
                        "Irrelevant attribute".into(),
                        "This attribute is meaningless; The linkage is already private or private weak by default.".into(),
                        span,
                    ));
                }

                if attributes.has_public_attribute() && linkage.is_standard() {
                    self.add_warning(CompilationIssue::Warning(
                        "Irrelevant attribute".into(),
                        "This attribute is meaningless; the linkage is the same as @public.".into(),
                        span,
                    ));
                }

                if attributes.has_public_attribute() && linkage.is_linker_private() {
                    self.add_warning(CompilationIssue::Warning(
                        "Irrelevant attribute".into(),
                        "This will cause a linking failure; the '@public' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_public_attribute() && linkage.is_linker_private_weak() {
                    self.add_warning(CompilationIssue::Warning(
                        "Irrelevant attribute".into(),
                        "This will cause a linking failure; the '@public' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_public_attribute() && linkage.is_internal() {
                    self.add_warning(CompilationIssue::Warning(
                        "Irrelevant attribute".into(),
                        "This will cause a linking failure; the '@public' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_extern_attribute() && linkage.is_linker_private() {
                    self.add_warning(CompilationIssue::Warning(
                        "Irrelevant attribute".into(),
                        "This will cause a linking failure; the '@extern' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_extern_attribute() && linkage.is_linker_private_weak() {
                    self.add_warning(CompilationIssue::Warning(
                        "Irrelevant attribute".into(),
                        "This will cause a linking failure; the '@extern' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_extern_attribute() && linkage.is_internal() {
                    self.add_warning(CompilationIssue::Warning(
                        "Irrelevant attribute".into(),
                        "This will cause a linking failure; the '@extern' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }
            }
        }

        if !attributes.has_extern_attribute() && attributes.has_ignore_attribute() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::Ignore) {
                self.add_error(CompilationIssue::Error(
                    "Attribute error".into(),
                    "The @ignore attribute requires the symbol to be annotated with @extern(\"something\").".into(),
                    None,
                    span,
                ));
            }
        }

        if attributes.has_inlinealways_attr() && attributes.has_inline_attr() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::InlineHint) {
                self.add_error(CompilationIssue::Error(
                    "Illogical attribute".into(),
                    "The attribute is not valid. Use either '@alwaysinline' or '@inline' attribute.".into(),
                    None,
                    span,
                ));
            }
        }

        if attributes.has_inline_attr() && attributes.has_noinline_attr() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::NoInline) {
                self.add_error(CompilationIssue::Error(
                    "Illogical attribute".into(),
                    "The attribute is not valid. Use either '@noinline' or '@inline' attribute."
                        .into(),
                    None,
                    span,
                ));
            }
        }

        if attributes.has_inlinealways_attr() && attributes.has_noinline_attr() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::NoInline) {
                self.add_error(CompilationIssue::Error(
                    "Illogical attribute".into(),
                    "The attribute is not valid. Use either '@alwaysinline' or '@inline' attribute.".into(),
                    None,
                    span,
                ));
            }
        }
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    fn get_repeated_attrs(&self, attributes: &'attr_checker ThrushAttributes) -> ThrushAttributes {
        let mut storage: AHashSet<ThrushAttributeComparator> = AHashSet::with_capacity(20);
        let mut repeated_attrs: ThrushAttributes = Vec::with_capacity(20);

        attributes.iter().for_each(|attr| {
            if !storage.insert(attr.as_attr_cmp()) {
                repeated_attrs.push(attr.clone());
            }
        });

        repeated_attrs
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
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

impl<'attr_checker> AttributeChecker<'attr_checker> {
    #[inline]
    fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }

    #[inline]
    fn add_warning(&mut self, warning: CompilationIssue) {
        self.warnings.push(warning);
    }
}
