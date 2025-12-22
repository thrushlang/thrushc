use crate::core::console::logging;

use crate::front_end::types::ast::Ast;

use crate::front_end::types::parser::repr::AssemblerFunction;
use crate::front_end::types::parser::repr::Function;
use crate::front_end::types::parser::repr::FunctionParameter;
use crate::front_end::types::parser::repr::GlobalConstant;
use crate::front_end::types::parser::repr::GlobalStatic;
use crate::front_end::types::parser::repr::Intrinsic;
use crate::front_end::types::parser::repr::Local;
use crate::front_end::types::parser::repr::LocalConstant;
use crate::front_end::types::parser::repr::LocalStatic;

impl Ast<'_> {
    #[inline]
    pub fn as_asm_function(&self) -> AssemblerFunction<'_> {
        if let Ast::AssemblerFunction {
            name,
            ascii_name,
            assembler,
            constraints,
            parameters_types,
            parameters,
            return_type,
            attributes,
            span,
            ..
        } = self
        {
            return (
                name,
                ascii_name,
                assembler,
                constraints,
                return_type,
                parameters,
                parameters_types,
                attributes,
                *span,
            );
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected assembler function for transformation to AssemblerFunction.",
        );
    }

    #[inline]
    pub fn as_global_static(&self) -> GlobalStatic<'_> {
        if let Ast::Static {
            name,
            ascii_name,
            kind,
            value,
            attributes,
            metadata,
            span,
            ..
        } = self
        {
            return (
                name,
                ascii_name,
                kind,
                value.as_deref(),
                attributes,
                *metadata,
                *span,
            );
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected static for transformation to GlobalStatic.",
        );
    }

    #[inline]
    pub fn as_global_constant(&self) -> GlobalConstant<'_> {
        if let Ast::Const {
            name,
            ascii_name,
            kind,
            value,
            attributes,
            metadata,
            span,
            ..
        } = self
        {
            return (
                name, ascii_name, kind, &**value, attributes, *metadata, *span,
            );
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected constant for transformation to GlobalConstant.",
        );
    }

    #[inline]
    pub fn as_function(&self) -> Function<'_> {
        if let Ast::Function {
            name,
            ascii_name,
            parameters,
            parameter_types,
            body,
            return_type,
            attributes,
            span,
            ..
        } = self
        {
            return (
                name,
                ascii_name,
                return_type,
                parameters,
                parameter_types,
                body.as_deref(),
                attributes,
                *span,
            );
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected function for transformation to Function.",
        );
    }

    #[inline]
    pub fn as_intrinsic(&self) -> Intrinsic<'_> {
        if let Ast::Intrinsic {
            name,
            external_name,
            parameters,
            parameters_types,
            return_type,
            attributes,
            span,
            ..
        } = self
        {
            return (
                name,
                external_name,
                return_type,
                parameters,
                parameters_types,
                attributes,
                *span,
            );
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected intrinsic for transformation to Intrinsic.",
        );
    }
}

impl Ast<'_> {
    pub fn as_local(&self) -> Local<'_> {
        if let Ast::Local {
            name,
            ascii_name,
            kind,
            value,
            attributes,
            metadata,
            span,
            ..
        } = self
        {
            return (
                name,
                ascii_name,
                kind,
                value.as_deref(),
                attributes,
                *metadata,
                *span,
            );
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected local for transformation to Local.",
        );
    }

    #[inline]
    pub fn as_local_constant(&self) -> LocalConstant<'_> {
        if let Ast::Const {
            name,
            ascii_name,
            kind,
            value,
            metadata,
            span,
            ..
        } = self
        {
            return (name, ascii_name, kind, &**value, *metadata, *span);
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected constant for transformation to LocalConstant.",
        );
    }

    #[inline]
    pub fn as_function_parameter(&self) -> FunctionParameter<'_> {
        if let Ast::FunctionParameter {
            name,
            ascii_name,
            kind,
            position,
            metadata,
            span,
        } = self
        {
            return (name, ascii_name, kind, *position, *span, *metadata);
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected function parameter for transformation to FunctionParameter.",
        );
    }

    #[inline]
    pub fn as_local_static(&self) -> LocalStatic<'_> {
        if let Ast::Static {
            name,
            ascii_name,
            kind,
            value,
            metadata,
            span,
            ..
        } = self
        {
            return (name, ascii_name, kind, value.as_deref(), *metadata, *span);
        }

        logging::print_frontend_panic(
            logging::LoggingType::FrontEndPanic,
            "Expected static for transformation to LocalStatic.",
        );
    }
}
