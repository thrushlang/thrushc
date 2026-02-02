use thrushc_ast::Ast;
use thrushc_attributes::{
    ThrushAttribute, ThrushAttributeComparator, ThrushAttributes,
    traits::{ThrushAttributeComparatorExtensions, ThrushAttributesExtensions},
};
use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;
use thrushc_typesystem::traits::TypeIsExtensions;

use crate::applicant::AttributeCheckerAttributeApplicant;

use ahash::AHashSet as HashSet;

mod applicant;

#[derive(Debug)]
pub struct AttributeChecker<'attr_checker> {
    ast: &'attr_checker [Ast<'attr_checker>],

    errors: Vec<CompilationIssue>,
    warnings: Vec<CompilationIssue>,

    diagnostician: Diagnostician,
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    #[inline]
    pub fn new(
        ast: &'attr_checker [Ast<'attr_checker>],
        file: &'attr_checker CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        Self {
            ast,
            errors: Vec::with_capacity(100),
            warnings: Vec::with_capacity(100),

            diagnostician: Diagnostician::new(file, options),
        }
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    pub fn start(&mut self) -> bool {
        for node in self.ast.iter() {
            self.analyze_ast(node);
        }

        self.check()
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    fn check(&mut self) -> bool {
        if !self.warnings.is_empty() {
            for warning in self.warnings.iter() {
                self.diagnostician
                    .dispatch_diagnostic(warning, thrushc_logging::LoggingType::Warning);
            }
        }

        if !self.errors.is_empty() {
            for error in self.errors.iter() {
                self.diagnostician
                    .dispatch_diagnostic(error, thrushc_logging::LoggingType::Error);
            }

            return true;
        }

        false
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    fn analyze_ast(&mut self, node: &'attr_checker Ast) {
        match node {
            Ast::Function {
                attributes,
                body,
                return_type,
                span,
                ..
            } => {
                if body.is_some() && attributes.has_extern_attribute() {
                    if let Some(span) = attributes.match_attr(ThrushAttributeComparator::Extern) {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "External functions cannot have a body. Remove it.".into(),
                            None,
                            span,
                        ));
                    }
                }

                if body.is_none() && !attributes.has_extern_attribute() {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0011,
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
                    AttributeCheckerAttributeApplicant::Function { return_type },
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
                        CompilationIssueCode::E0013,
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
                        CompilationIssueCode::E0013,
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
            Ast::Block { nodes, post, .. } => {
                for node in nodes.iter() {
                    self.analyze_ast(node);
                }

                for postnode in post.iter() {
                    self.analyze_ast(postnode);
                }
            }
            Ast::Defer { node, .. } => {
                self.analyze_ast(node);
            }

            Ast::For { local: node, .. } => {
                self.analyze_ast(node);
            }
            Ast::While {
                variable: Some(node),
                ..
            } => {
                self.analyze_ast(node);
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
            AttributeCheckerAttributeApplicant::Function { return_type, .. } => {
                self.check_irrelevant_attributes(attributes, applicant);
                self.check_illogical_attributes(attributes);

                if let Some(attr) = attributes.get_attr(ThrushAttributeComparator::Constructor) {
                    if !return_type.is_void_type() {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "Functions that execute before the entry point should not return anything. Rewrite it to a void type.".into(),
                            None,
                            attr.get_span(),
                        ));
                    }

                    if !attributes.has_public_attribute() {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "Functions that run before the entry point must be public. Add '@public' attribute."
                                .into(),
                            None,
                            attr.get_span(),
                        ));
                    }

                    if let Some(attr) = attributes.get_attr(ThrushAttributeComparator::Extern) {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "Functions that run before the entry point cannot be external. Remove the attribute.".into(),
                            None,
                            attr.get_span(),
                        ));
                    }

                    if let Some(attr) = attributes.get_attr(ThrushAttributeComparator::Linkage) {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "Functions that run before the entrypoint cannot have custom linkage. Remove it.".into(),
                            None,
                            attr.get_span(),
                        ));
                    }
                }

                if let Some(attr) = attributes.get_attr(ThrushAttributeComparator::Destructor) {
                    if !return_type.is_void_type() {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "Functions that execute after the entry point should not return anything. Rewrite them to a void type.".into(),
                            None,
                            attr.get_span(),
                        ));
                    }

                    if !attributes.has_public_attribute() {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "Functions that run after the entry point must be public. Add '@public' attribute."
                                .into(),
                            None,
                            attr.get_span(),
                        ));
                    }

                    if let Some(attr) = attributes.get_attr(ThrushAttributeComparator::Extern) {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "Functions that run after the entry point cannot be external. Remove it.".into(),
                            None,
                            attr.get_span(),
                        ));
                    }

                    if let Some(attr) = attributes.get_attr(ThrushAttributeComparator::Linkage) {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0013,
                            "Functions that run after the entrypoint cannot have custom linkage. Remove it.".into(),
                            None,
                            attr.get_span(),
                        ));
                    }
                }

                self.get_repeated_attrs(attributes).iter().for_each(|attr| {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0033,
                        "Repetitive attributes are disallowed. Remove each one.".into(),
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
                        CompilationIssueCode::E0013,
                        "Intrinsic qualities should always have public visibility.".into(),
                        None,
                        span,
                    ));
                }

                self.get_repeated_attrs(attributes).iter().for_each(|attr| {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0033,
                        "Repetitive attributes are disallowed. Remove each one.".into(),
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
                        CompilationIssueCode::E0033,
                        "Repetitive attributes are disallowed. Remove each one.".into(),
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
                            CompilationIssueCode::E0011,
                            "A pure assembler function always have syntax mode. Add the '@asmsyntax' attribute.".into(),
                            None,
                            span,
                        ));
                    }
                }

                if let Some(ThrushAttribute::AsmSyntax(syntax, span)) =
                    attributes.get_attr(ThrushAttributeComparator::AsmSyntax)
                {
                    const INLINE_ASSEMBLER_SYNTAXES: [&str; 2] = ["Intel", "AT&T"];

                    if !INLINE_ASSEMBLER_SYNTAXES.contains(&syntax.as_str()) {
                        self.add_error(CompilationIssue::Error(
                            CompilationIssueCode::E0012,
                            format!("Expected a valid assembler syntax, got '{}'.", syntax),
                            None,
                            span,
                        ));
                    }
                }

                if let Some(ThrushAttribute::Convention(conv, span)) =
                    attributes.get_attr(ThrushAttributeComparator::Convention)
                {
                    if !thrushc_attributes::callconventions::CALL_CONVENTIONS_AVAILABLE
                        .contains(&conv.as_str())
                    {
                        self.add_warning(CompilationIssue::Warning(
                            CompilationIssueCode::W0002,
                            "Unknown calling convention, setting C by default.".into(),
                            span,
                        ));
                    }
                }

                self.get_repeated_attrs(attributes).iter().for_each(|attr| {
                    self.add_error(CompilationIssue::Error(
                        CompilationIssueCode::E0033,
                        "Repetitive attributes are disallowed. Remove each one.".into(),
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
                        CompilationIssueCode::E0033,
                        "Repetitive attributes are disallowed. Remove each one.".into(),
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
            ThrushAttributeComparator::Constructor,
            ThrushAttributeComparator::Destructor,
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
            AttributeCheckerAttributeApplicant::Function { .. } => {
                attributes.iter().for_each(|attr| {
                    if !VALID_FUNCTION_ATTRIBUTES.contains(&attr.as_attr_cmp()) {
                        self.add_warning(CompilationIssue::Warning(
                            CompilationIssueCode::W0001,
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
                            CompilationIssueCode::W0001,
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
                            CompilationIssueCode::W0001,
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
                            CompilationIssueCode::W0001,
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
                            CompilationIssueCode::W0001,
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
                            CompilationIssueCode::W0001,
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
                            CompilationIssueCode::W0001,
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
                            CompilationIssueCode::W0001,
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
                    CompilationIssueCode::E0013,
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
                if !thrushc_attributes::callconventions::CALL_CONVENTIONS_AVAILABLE
                    .contains(&conv.as_str())
                {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0002,
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
                if !thrushc_attributes::linkage::LINKAGES_AVAILABLE.contains(&linkage_raw.as_str())
                {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0003,
                        "Unknown linking, assuming non-proprietary C standard.".into(),
                        span,
                    ));
                }

                if !attributes.has_public_attribute()
                    && (linkage.is_linker_private() || linkage.is_linker_private_weak())
                {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0004,
                        "This attribute is meaningless; The linkage is already private or private weak by default.".into(),
                        span,
                    ));
                }

                if attributes.has_public_attribute() && linkage.is_standard() {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0004,
                        "This attribute is meaningless; the linkage is the same as @public.".into(),
                        span,
                    ));
                }

                if attributes.has_public_attribute() && linkage.is_linker_private() {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0004,
                        "This will cause a linking failure; the '@public' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_public_attribute() && linkage.is_linker_private_weak() {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0004,
                        "This will cause a linking failure; the '@public' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_public_attribute() && linkage.is_internal() {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0004,
                        "This will cause a linking failure; the '@public' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_extern_attribute() && linkage.is_linker_private() {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0004,
                        "This will cause a linking failure; the '@extern' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_extern_attribute() && linkage.is_linker_private_weak() {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0004,
                        "This will cause a linking failure; the '@extern' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }

                if attributes.has_extern_attribute() && linkage.is_internal() {
                    self.add_warning(CompilationIssue::Warning(
                        CompilationIssueCode::W0004,
                        "This will cause a linking failure; the '@extern' attribute requires non-proprietary linking.".into(),
                        span,
                    ));
                }
            }
        }

        if attributes.has_constructor_attribute() && attributes.has_destructor_attribute() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::Destructor) {
                self.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0012,
                    "A symbol cannot be both a constructor and a destructor at the same time. Remove an attribute.".into(),
                    None,
                    span,
                ));
            }
        }

        if !attributes.has_extern_attribute() && attributes.has_ignore_attribute() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::Ignore) {
                self.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0013,
                    "The @arbitraryArgs attribute requires the symbol to be annotated with @extern(\"something\").".into(),
                    None,
                    span,
                ));
            }
        }

        if attributes.has_inlinealways_attr() && attributes.has_inline_attr() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::InlineHint) {
                self.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0033,
                    "The attribute is not valid. Use either '@alwaysInline' or '@inline' attribute.".into(),
                    None,
                    span,
                ));
            }
        }

        if attributes.has_inline_attr() && attributes.has_noinline_attr() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::NoInline) {
                self.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0033,
                    "The attribute is not valid. Use either '@noInline' or '@inline' attribute."
                        .into(),
                    None,
                    span,
                ));
            }
        }

        if attributes.has_inlinealways_attr() && attributes.has_noinline_attr() {
            if let Some(span) = attributes.match_attr(ThrushAttributeComparator::NoInline) {
                self.add_error(CompilationIssue::Error(
                    CompilationIssueCode::E0033,
                    "The attribute is not valid. Use either '@alwaysInline' or '@inline' attribute.".into(),
                    None,
                    span,
                ));
            }
        }
    }
}

impl<'attr_checker> AttributeChecker<'attr_checker> {
    fn get_repeated_attrs(&self, attributes: &'attr_checker ThrushAttributes) -> ThrushAttributes {
        let mut storage: HashSet<ThrushAttributeComparator> = HashSet::with_capacity(20);
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
    fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }

    #[inline]
    fn add_warning(&mut self, warning: CompilationIssue) {
        self.warnings.push(warning);
    }
}
