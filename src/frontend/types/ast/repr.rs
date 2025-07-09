use crate::frontend::types::{
    ast::Ast,
    parser::repr::{
        AssemblerFunctionRepresentation, ConstantRepresentation, FunctionRepresentation,
        StaticRepresentation,
    },
};

impl Ast<'_> {
    pub fn as_asm_function_representation(&self) -> AssemblerFunctionRepresentation {
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

        unreachable!()
    }

    pub fn as_static_representation(&self) -> StaticRepresentation {
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
            return (name, ascii_name, kind, &**value, *metadata, attributes);
        }

        unreachable!()
    }

    pub fn as_constant_representation(&self) -> ConstantRepresentation {
        if let Ast::Const {
            name,
            ascii_name,
            kind,
            value,
            attributes,
            ..
        } = self
        {
            return (name, ascii_name, kind, &**value, attributes);
        }

        unreachable!()
    }

    pub fn as_function_representation(&self) -> FunctionRepresentation {
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

        unreachable!()
    }
}
