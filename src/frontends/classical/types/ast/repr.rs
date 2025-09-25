use crate::{
    core::console::logging,
    frontends::classical::types::{
        ast::Ast,
        parser::repr::{
            GlobalAssemblerFunction, GlobalConstant, GlobalFunction, GlobalStatic, Local,
            LocalConstant, LocalStatic,
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
            );
        }

        logging::print_backend_bug(
            logging::LoggingType::BackendBug,
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
            ..
        } = self
        {
            return (name, ascii_name, kind, &**value, attributes, *metadata);
        }

        logging::print_backend_bug(
            logging::LoggingType::BackendBug,
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
            ..
        } = self
        {
            return (name, ascii_name, kind, &**value, attributes, *metadata);
        }

        logging::print_backend_bug(
            logging::LoggingType::BackendBug,
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
            ..
        } = self
        {
            return (
                name,
                ascii_name,
                return_type,
                parameters,
                parameter_types,
                body,
                attributes,
            );
        }

        logging::print_backend_bug(
            logging::LoggingType::BackendBug,
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
            ..
        } = self
        {
            return (name, ascii_name, kind, &**value, attributes, *metadata);
        }

        logging::print_backend_bug(
            logging::LoggingType::BackendBug,
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
            ..
        } = self
        {
            return (name, ascii_name, kind, &**value, *metadata);
        }

        logging::print_backend_bug(
            logging::LoggingType::BackendBug,
            "Expected constant for transformation to LocalConstant.",
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
            ..
        } = self
        {
            return (name, ascii_name, kind, &**value, *metadata);
        }

        logging::print_backend_bug(
            logging::LoggingType::BackendBug,
            "Expected static for transformation to LocalStatic.",
        );
    }
}
