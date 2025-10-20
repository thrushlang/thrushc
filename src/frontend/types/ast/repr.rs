use crate::{
    core::console::logging,
    frontend::types::{
        ast::Ast,
        parser::repr::{
            FunctionParameter, GlobalAssemblerFunction, GlobalConstant, GlobalFunction,
            GlobalStatic, Local, LocalConstant, LocalStatic,
        },
    },
};

impl Ast<'_> {
    #[inline]
    pub fn as_global_asm_function(&self) -> GlobalAssemblerFunction<'_> {
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

        logging::print_bug(
            logging::LoggingType::Bug,
            "Expected assembler function for transformation to GlobalAssemblerFunction.",
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

        logging::print_bug(
            logging::LoggingType::Bug,
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

        logging::print_bug(
            logging::LoggingType::Bug,
            "Expected constant for transformation to GlobalConstant.",
        );
    }

    #[inline]
    pub fn as_global_function(&self) -> GlobalFunction<'_> {
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

        logging::print_bug(
            logging::LoggingType::Bug,
            "Expected function for transformation to GlobalFunction.",
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

        logging::print_bug(
            logging::LoggingType::Bug,
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

        logging::print_bug(
            logging::LoggingType::Bug,
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

        logging::print_bug(
            logging::LoggingType::Bug,
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

        logging::print_bug(
            logging::LoggingType::Bug,
            "Expected static for transformation to LocalStatic.",
        );
    }
}
