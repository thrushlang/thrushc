/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

use thrustc_errors::CompilationIssue;
use thrustc_span::Span;
use thrustc_token_type::TokenType;
use thrustc_typesystem::{
    Type,
    modificators::StructureTypeModificator,
    traits::{TypeIsExtensions, TypeStructExtensions},
};

use crate::{
    Ast,
    builitins::ThrustBuiltin,
    data::{
        ConstructorData, EnumData, EnumDataField, PropertyData, PropertyDataField, StructureData,
    },
    traits::{
        AstCodeBlockEntensions, AstConstantExtensions, AstConstructorDataExtensions,
        AstDeclarationExtensions, AstEnumFieldsDataExtensions, AstExpressionExtensions,
        AstExpressionOperationExtensions, AstGetType, AstMemoryExtensions,
        AstPropertyDataExtensions, AstPropertyDataFieldExtensions, AstScopeExtensions,
        AstStandardExtensions, AstStatementExtensions, AstStructFieldsDataExtensions,
        AstStructureDataExtensions,
    },
};

impl AstStandardExtensions for Ast<'_> {
    #[inline]
    fn is_literal_value(&self) -> bool {
        match self {
            Ast::Integer { .. }
            | Ast::Float { .. }
            | Ast::Boolean { .. }
            | Ast::Char { .. }
            | Ast::CString { .. }
            | Ast::CNString { .. }
            | Ast::NullPtr { .. } => true,

            Ast::FixedArray { items, .. } => items.iter().all(|item| item.is_literal_value()),
            Ast::Array { items, .. } => items.iter().all(|item| item.is_literal_value()),

            Ast::EnumValue { value, .. } => value.is_literal_value(),

            Ast::Group { node, .. } => node.is_literal_value(),
            Ast::BinaryOp { left, right, .. } => {
                left.is_literal_value() && right.is_literal_value()
            }
            Ast::UnaryOp { node, .. } => node.is_literal_value(),

            _ => false,
        }
    }

    #[inline]
    fn is_reference(&self) -> bool {
        matches!(self, Ast::Reference { .. })
    }

    #[inline]
    fn is_function(&self) -> bool {
        matches!(self, Ast::Function { .. })
    }

    #[inline]
    fn is_intrinsic(&self) -> bool {
        matches!(self, Ast::Intrinsic { .. })
    }

    #[inline]
    fn is_asm_function(&self) -> bool {
        matches!(self, Ast::AssemblerFunction { .. })
    }

    #[inline]
    fn is_struct(&self) -> bool {
        matches!(self, Ast::Struct { .. })
    }

    #[inline]
    fn is_enum(&self) -> bool {
        matches!(self, Ast::Enum { .. })
    }

    #[inline]
    fn is_cstring(&self) -> bool {
        matches!(self, Ast::CString { .. })
    }

    #[inline]
    fn is_cnstring(&self) -> bool {
        matches!(self, Ast::CNString { .. })
    }

    #[inline]
    fn is_constant(&self) -> bool {
        matches!(self, Ast::Const { .. })
    }

    #[inline]
    fn is_static(&self) -> bool {
        matches!(self, Ast::Static { .. })
    }

    #[inline]
    fn is_integer(&self) -> bool {
        matches!(self, Ast::Integer { .. })
    }

    #[inline]
    fn is_terminator(&self) -> bool {
        matches!(self, Ast::Return { .. })
    }

    #[inline]
    fn is_unreacheable(&self) -> bool {
        matches!(self, Ast::Unreachable { .. })
    }

    #[inline]
    fn is_break(&self) -> bool {
        matches!(self, Ast::Break { .. })
    }

    #[inline]
    fn is_breakall(&self) -> bool {
        matches!(self, Ast::BreakAll { .. })
    }

    #[inline]
    fn is_continue(&self) -> bool {
        matches!(self, Ast::Continue { .. })
    }

    #[inline]
    fn is_continueall(&self) -> bool {
        matches!(self, Ast::ContinueAll { .. })
    }

    #[inline]
    fn is_custom_type(&self) -> bool {
        matches!(self, Ast::CustomType { .. })
    }

    #[inline]
    fn is_global_asm(&self) -> bool {
        matches!(self, Ast::GlobalAssembler { .. })
    }

    #[inline]
    fn is_import(&self) -> bool {
        matches!(self, Ast::Import { .. })
    }

    #[inline]
    fn is_conditional(&self) -> bool {
        matches!(self, Ast::If { .. } | Ast::Elif { .. } | Ast::Else { .. })
    }

    #[inline]
    fn is_post_execution_at_scope(&self) -> bool {
        matches!(self, Ast::Defer { .. })
    }
}

impl AstStatementExtensions for Ast<'_> {
    fn is_statement(&self) -> bool {
        matches!(
            self,
            Ast::Block { .. }
                | Ast::If { .. }
                | Ast::Else { .. }
                | Ast::Elif { .. }
                | Ast::While { .. }
                | Ast::For { .. }
                | Ast::Loop { .. }
                | Ast::Return { .. }
                | Ast::Break { .. }
                | Ast::BreakAll { .. }
                | Ast::Continue { .. }
                | Ast::ContinueAll { .. }
                | Ast::Local { .. }
                | Ast::Struct { .. }
                | Ast::Const { .. }
                | Ast::Static { .. }
                | Ast::Defer { .. }
        )
    }
}

impl AstDeclarationExtensions for Ast<'_> {
    fn is_declaration(&self) -> bool {
        matches!(
            self,
            Ast::CustomType { .. }
                | Ast::Struct { .. }
                | Ast::Const { .. }
                | Ast::Static { .. }
                | Ast::Enum { .. }
                | Ast::Function { .. }
                | Ast::Intrinsic { .. }
                | Ast::AssemblerFunction { .. }
                | Ast::GlobalAssembler { .. }
                | Ast::Import { .. }
                | Ast::Embedded { .. }
        )
    }
}

impl AstExpressionExtensions for Ast<'_> {
    fn is_expression(&self) -> bool {
        !self.is_declaration() && !self.is_statement()
    }
}

impl AstCodeBlockEntensions for Ast<'_> {
    #[inline]
    fn is_empty_block(&self) -> bool {
        let Ast::Block { nodes, .. } = self else {
            return false;
        };

        nodes.is_empty()
    }

    #[inline]
    fn has_terminator(&self) -> bool {
        let Ast::Block { nodes, .. } = self else {
            return false;
        };

        {
            for node in nodes.iter() {
                if node.is_terminator() {
                    return true;
                }

                if let Ast::If {
                    block,
                    elseif,
                    anyway,
                    ..
                } = node
                {
                    let if_branch_returns: bool = block.has_terminator();

                    let all_elif_return: bool = elseif.iter().all(|elif_node| {
                        if let Ast::Elif { block, .. } = elif_node {
                            block.has_terminator()
                        } else {
                            false
                        }
                    });

                    let else_branch_returns: bool = anyway.as_ref().is_some_and(|otherwise| {
                        if let Ast::Else { block, .. } = &**otherwise {
                            block.has_terminator()
                        } else {
                            false
                        }
                    });

                    let if_else_returns: bool =
                        if_branch_returns && else_branch_returns && elseif.is_empty();
                    let full_returns: bool =
                        if_branch_returns && all_elif_return && else_branch_returns;

                    if if_else_returns || full_returns {
                        return true;
                    }
                }
            }
        }

        false
    }
}

impl AstMemoryExtensions for Ast<'_> {
    #[inline]
    fn is_allocated(&self) -> bool {
        match self {
            Ast::Reference { metadata, .. } => metadata.is_allocated(),
            Ast::Property { metadata, .. } => metadata.is_allocated(),

            _ => false,
        }
    }

    #[inline]
    fn is_allocated_value(&self) -> Result<bool, CompilationIssue> {
        match self {
            Ast::Reference { metadata, .. } => Ok(metadata.is_allocated()),
            Ast::Property { metadata, .. } => Ok(metadata.is_allocated()),

            _ => Ok(self.get_value_type()?.is_ptr_like_type()),
        }
    }
}

impl AstConstantExtensions for Ast<'_> {
    fn is_constant_value(&self) -> bool {
        match self {
            Ast::Integer { .. }
            | Ast::Float { .. }
            | Ast::Boolean { .. }
            | Ast::Char { .. }
            | Ast::CNString { .. }
            | Ast::CString { .. }
            | Ast::NullPtr { .. }
            | Self::Builtin {
                builtin:
                    ThrustBuiltin::AlignOf { .. }
                    | ThrustBuiltin::SizeOf { .. }
                    | ThrustBuiltin::AbiSizeOf { .. }
                    | ThrustBuiltin::AbiAlignOf { .. }
                    | ThrustBuiltin::BitSizeOf { .. },
                ..
            } => true,
            Ast::EnumValue { value, .. } => value.is_constant_value(),
            Ast::DirectRef { expr, .. } => expr.is_constant_value(),
            Ast::Group { node, .. } => node.is_constant_value(),
            Ast::BinaryOp { left, right, .. } => {
                left.is_constant_value() && right.is_constant_value()
            }
            Ast::UnaryOp { node, .. } => node.is_constant_value(),
            Ast::Reference { metadata, .. } => metadata.is_constant_ref(),
            Ast::As { metadata, .. } => metadata.is_constant(),
            Ast::FixedArray { items, .. } => items.iter().all(|item| item.is_constant_value()),
            Ast::Constructor { data, .. } => data.iter().all(|arg| arg.1.is_constant_value()),

            _ => false,
        }
    }
}

impl AstScopeExtensions for Ast<'_> {
    #[inline]
    fn is_compatible_with_main_scope(&self) -> bool {
        matches!(
            self,
            Ast::CustomType { .. }
                | Ast::Struct { .. }
                | Ast::Enum { .. }
                | Ast::Intrinsic { .. }
                | Ast::Function { .. }
                | Ast::AssemblerFunction { .. }
                | Ast::GlobalAssembler { .. }
                | Ast::Const { .. }
                | Ast::Static { .. }
                | Ast::Import { .. }
        )
    }
}

impl AstExpressionOperationExtensions for Ast<'_> {
    #[inline]
    fn is_binary_operation(&self) -> bool {
        matches!(self, Ast::BinaryOp { .. })
    }

    #[inline]
    fn get_binary_operator(&self) -> Option<TokenType> {
        if let Ast::BinaryOp { operator, .. } = self {
            return Some(*operator);
        }

        None
    }

    #[inline]
    fn is_unary_operation(&self) -> bool {
        matches!(self, Ast::UnaryOp { .. })
    }

    #[inline]
    fn is_unary_preeval_operation(&self) -> bool {
        matches!(self, Ast::UnaryOp { is_pre: true, .. })
    }
}

impl AstPropertyDataExtensions for PropertyData {
    #[inline]
    fn get_first_property(&self) -> Option<&crate::data::PropertyDataField> {
        self.first()
    }
}

impl AstPropertyDataFieldExtensions for PropertyDataField {
    #[inline]
    fn get_base_type(&self) -> thrustc_typesystem::Type {
        self.0.clone()
    }

    #[inline]
    fn get_property_type(&self) -> thrustc_typesystem::Type {
        self.1.0.clone()
    }

    #[inline]
    fn get_index(&self) -> u32 {
        self.1.1
    }
}

impl AstConstructorDataExtensions for ConstructorData<'_> {
    #[inline]
    fn get_type(&self, name: &str, modificator: StructureTypeModificator, span: Span) -> Type {
        let types: Vec<Type> = self.iter().map(|field| field.2.clone()).collect();
        Type::create_struct_type(name.to_string(), types.as_slice(), modificator, span)
    }
}

impl<'a> AstStructureDataExtensions<'a> for StructureData<'a> {
    fn new(
        name: &'a str,
        modificator: thrustc_typesystem::modificators::StructureTypeModificator,
        span: thrustc_span::Span,
    ) -> Self {
        (
            name,
            Vec::with_capacity(u8::MAX as usize),
            modificator,
            span,
        )
    }

    fn get_fields(&self) -> &crate::data::StructureDataFields<'_> {
        &self.1
    }
}

impl AstStructFieldsDataExtensions for StructureData<'_> {
    #[inline]
    fn get_type(&self) -> Type {
        let types: Vec<Type> = self.1.iter().map(|field| field.1.clone()).collect();

        let name: String = self.0.to_string();
        let span: Span = self.3;

        Type::create_struct_type(name, types.as_slice(), self.get_modificator(), span)
    }

    #[inline]
    fn get_modificator(&self) -> StructureTypeModificator {
        self.2
    }
}

impl<'a> AstEnumFieldsDataExtensions<'a> for EnumData<'a> {
    fn get_field(&self, name: &str) -> Option<EnumDataField<'a>> {
        self.iter().find(|enum_field| enum_field.0 == name).cloned()
    }
}

impl std::cmp::PartialEq for Ast<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ast::Integer { .. }, Ast::Integer { .. })
            | (Ast::Float { .. }, Ast::Float { .. })
            | (Ast::CString { .. }, Ast::CString { .. })
            | (Ast::CNString { .. }, Ast::CNString { .. }) => true,
            (left, right) => std::mem::discriminant(left) == std::mem::discriminant(right),
        }
    }
}

impl std::fmt::Display for Ast<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // --- TOP-LEVEL DEFS ---
            Ast::AssemblerFunction {
                name,
                parameters,
                assembler,
                constraints,
                return_type,
                ..
            } => {
                writeln!(f, "\nTop-Level - Assembler Function asmfn {}(", name)?;
                for (i, param) in parameters.iter().enumerate() {
                    writeln!(f, "{}", param)?;
                    if i != parameters.len().saturating_sub(1) {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") {} ", return_type)?;
                write!(f, "Assembler {{ {} }} ", assembler)?;
                write!(f, "Assembler Constraints {{ {} }}", constraints)?;
                Ok(())
            }
            Ast::Const {
                name, kind, value, ..
            } => {
                write!(f, "Top-Level - Const {}: {} = {}", name, kind, value)
            }
            Ast::CustomType { kind, .. } => {
                writeln!(f, "\nTop-Level - Type type {}", kind)
            }
            Ast::Embedded { name, literal, .. } => {
                writeln!(f, "\nTop-Level - Embedded {} {:?};", name, literal)
            }
            Ast::Enum { name, .. } => {
                writeln!(f, "\nTop-Level - Enum {} {{ ... }}", name)
            }
            Ast::Function {
                name,
                parameters,
                body,
                return_type,
                attributes,
                ..
            } => {
                writeln!(f, "\nTop Level - Function fn {}", name)?;
                write!(f, "(")?;
                for (i, param) in parameters.iter().enumerate() {
                    write!(f, "{}", param)?;
                    if i != parameters.len().saturating_sub(1) {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") {} ", return_type)?;
                attributes
                    .iter()
                    .try_for_each(|attr| write!(f, "{}", attr))?;
                if let Some(body) = body {
                    writeln!(f, "{}", body)?;
                }
                Ok(())
            }
            Ast::Import { .. } => {
                writeln!(f, "\nTop-Level - Module Importation import")
            }
            Ast::ImportC { .. } => {
                writeln!(f, "\nTop-Level - Invoke C Preprocessador importC")
            }
            Ast::Intrinsic {
                name,
                parameters,
                return_type,
                ..
            } => {
                writeln!(f, "\nTop-Level - Compiler Intrinsic intrinsic {}(", name)?;
                for (i, param) in parameters.iter().enumerate() {
                    writeln!(f, "{}", param)?;
                    if i != parameters.len().saturating_sub(1) {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") {} ", return_type)
            }
            Ast::Static {
                name, kind, value, ..
            } => {
                if let Some(value) = value {
                    writeln!(f, "\nTop-Level - Static {}: {} = {}", name, kind, value)
                } else {
                    writeln!(f, "\nTop-Level - Static {}: {}", name, kind)
                }
            }
            Ast::Struct { name, .. } => {
                writeln!(f, "\nTop-Level - Struct {} {{ ... }}", name)
            }

            // --- STATEMENTS ---
            Ast::AssemblerFunctionParameter { name, kind, .. } => {
                write!(f, "Stmt - Parameter {}: {}", name, kind)
            }
            Ast::Block { nodes, post, .. } => {
                writeln!(f, "\nStmt - Code Block - Start {{")?;
                for node in nodes {
                    write!(f, "{}", node)?;
                }
                for node in post {
                    write!(f, "{}", node)?;
                }
                write!(f, "}} Code Block - End")
            }
            Ast::Break { .. } => writeln!(f, "Stmt - Break"),
            Ast::BreakAll { .. } => writeln!(f, "Stmt - Breakall"),
            Ast::Continue { .. } => writeln!(f, "Stmt - Continue"),
            Ast::ContinueAll { .. } => writeln!(f, "Stmt - Continueall"),
            Ast::Defer { node, .. } => writeln!(f, "\nStmt - Defer {}", node),
            Ast::Elif {
                condition, block, ..
            } => {
                writeln!(f, "Stmt - Else If/Elif {}{}", condition, block)
            }
            Ast::Else { block, .. } => writeln!(f, "Stmt - Else {}", block),
            Ast::For {
                local,
                condition,
                actions,
                block,
                ..
            } => {
                writeln!(f, "Stmt - For")?;

                write!(f, "{}", local)?;
                writeln!(f, "{}", condition)?;
                write!(f, "{}", actions)?;
                write!(f, "{}", block)?;

                Ok(())
            }
            Ast::FunctionParameter { name, kind, .. } => {
                write!(f, "Stmt - Parameter {}: {}", name, kind)
            }
            Ast::If {
                condition,
                block,
                elseif,
                anyway,
                ..
            } => {
                writeln!(f, "Stmt - If {}{}", condition, block)?;
                for elif in elseif {
                    writeln!(f, "Stmt - Else If/Elif {}", elif)?;
                }
                if let Some(anyway) = anyway {
                    writeln!(f, "Stmt - Else {}", anyway)?;
                }
                Ok(())
            }
            Ast::IntrinsicParameter { kind, .. } => {
                write!(f, "Stmt - Parameter {}", kind)
            }
            Ast::Local {
                name, kind, value, ..
            } => {
                if let Some(value) = value {
                    writeln!(f, "Stmt - Local {}: {} = {};", name, kind, value)
                } else {
                    writeln!(f, "Stmt - Local {}: {};", name, kind)
                }
            }
            Ast::Loop { block, .. } => writeln!(f, "Stmt - Loop {}", block),
            Ast::Mut { source, value, .. } => {
                writeln!(f, "Stmt - Mutation {} = {}", source, value)
            }
            Ast::Return { expression, .. } => {
                if let Some(expr) = expression {
                    writeln!(f, "Stmt - Return {}", expr)
                } else {
                    writeln!(f, "Stmt - Return")
                }
            }
            Ast::While {
                condition, block, ..
            } => {
                writeln!(f, "Stmt - While {} {}", condition, block)
            }

            // --- EXPRESSIONS ---
            Ast::Address {
                source, indexes, ..
            } => {
                write!(f, "address {}[", source)?;
                for (i, idx) in indexes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", idx)?;
                }
                write!(f, "]")
            }
            Ast::Array { items, .. } => {
                write!(f, "Expression - Array [")?;
                for (i, item) in items.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i != items.len().saturating_sub(1) {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Ast::As { from, cast, .. } => write!(f, "Expression - Type Cast {} as {}", from, cast),
            Ast::AsmValue { assembler, .. } => {
                write!(f, "Expression - Assembler Value asm(\"{}\")", assembler)
            }
            Ast::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                write!(
                    f,
                    "Expression - BinaryOperation {} {} {}",
                    left, operator, right
                )
            }
            Ast::Boolean { value, .. } => write!(f, "Expression - Boolean {}", value),
            Ast::Builtin { builtin, .. } => {
                write!(f, "Expression - Compiler Built-in builtin({:?})", builtin)
            }
            Ast::Call { name, args, .. } => {
                write!(f, "Expression - Function Call {}(", name)?;
                for (index, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;
                    if index != args.len().saturating_sub(1) {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
            Ast::Char { byte, .. } => write!(f, "Expression - Char {}", byte),
            Ast::CNString { bytes, .. } => {
                write!(
                    f,
                    "Expression - CNString {:?}",
                    String::from_utf8_lossy(bytes)
                )
            }
            Ast::Constructor { name, data, .. } => {
                write!(f, "Expression - Constructor {}{{ ", name)?;
                for (i, (field_name, ..)) in data.iter().enumerate() {
                    write!(f, "{}", field_name)?;
                    if i != data.len().saturating_sub(1) {
                        write!(f, ", ")?;
                    }
                }
                write!(f, " }}")
            }
            Ast::CString { bytes, .. } => {
                write!(
                    f,
                    "Expression - CString {:?}",
                    String::from_utf8_lossy(bytes)
                )
            }
            Ast::Deref { value, .. } => write!(f, "Expression - Dereferentation defer {}", value),
            Ast::DirectRef { expr, .. } => {
                write!(f, "Expression - Direct Referentation ref {}", expr)
            }
            Ast::EnumValue { name, value, .. } => {
                write!(f, "Expression - Enum Value {}->{}", name, value)
            }
            Ast::FixedArray { items, .. } => {
                write!(f, "Expression - FixedArray fixed[")?;
                for (i, item) in items.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i != items.len().saturating_sub(1) {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Ast::Float { value, .. } => write!(f, "Expression - Float {}", value),
            Ast::GlobalAssembler { asm, .. } => {
                write!(f, "Expression - Global Assembler global_asm(\"{}\")", asm)
            }
            Ast::Group { node, .. } => write!(f, "Expression - Group ({})", node),
            Ast::Index { source, index, .. } => {
                write!(f, "Expression - Indexation {}[{}]", source, index)
            }
            Ast::IndirectCall { function, args, .. } => {
                write!(f, "Expression - Anonymous Function Call {}(", function)?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;
                    if i != args.len().saturating_sub(1) {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
            Ast::Integer { value, .. } => write!(f, "Expression - Integer {}", value),
            Ast::Load { source, .. } => write!(f, "load {}", source),
            Ast::NullPtr { .. } => write!(f, "Expression - nullptr"),
            Ast::Property { source, .. } => write!(f, "Expresssion - Property {}.property", source),
            Ast::Reference { name, .. } => write!(f, "Expression - {}", name),
            Ast::UnaryOp {
                operator,
                node,
                is_pre,
                ..
            } => {
                if *is_pre {
                    write!(f, "Expression - UnaryOperation {}{}", operator, node)
                } else {
                    write!(f, "Expression - UnaryOperation {}{}", node, operator)
                }
            }
            Ast::Unreachable { .. } => write!(f, "Expression  - Unreachable"),
            Ast::Write {
                source,
                write_value,
                ..
            } => {
                write!(f, "write {} = {}", source, write_value)
            }

            // --- OTROS ---
            Ast::Invalid { .. } => write!(f, "invalid ast"),
        }
    }
}
