use std::fmt::Display;

use crate::backend::llvm::compiler::attributes::{AttributeBuilder, LLVMAttributeApplicant};
use crate::backend::llvm::compiler::codegen::LLVMCodegen;
use crate::backend::llvm::compiler::typegen;
use crate::backend::types::repr::LLVMFunction;
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::parser::repr::{FunctionParameter, GlobalFunction};
use crate::{
    backend::llvm::compiler::{
        attributes::LLVMAttribute,
        context::LLVMCodeGenContext,
        conventions::CallConvention,
        utils::{self, LONG_RANGE_OBFUSCATION},
    },
    frontend::{
        types::{
            ast::Ast,
            parser::stmts::{traits::ThrushAttributesExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

use inkwell::builder::Builder;
use inkwell::{
    context::Context,
    module::{Linkage, Module},
    types::FunctionType,
    values::FunctionValue,
};

pub fn compile_decl<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    global_fn: GlobalFunction<'ctx>,
) {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let funcion_name: &str = global_fn.0;
    let function_ascii_name: &str = global_fn.1;
    let function_type: &Type = global_fn.2;
    let function_parameters: &[Ast<'ctx>] = global_fn.3;
    let function_parameters_types: &[Type] = global_fn.4;
    let attributes: &ThrushAttributes = global_fn.6;

    let ignore_args: bool = attributes.has_ignore_attribute();
    let is_public: bool = attributes.has_public_attribute();

    let mut extern_name: Option<&str> = None;
    let mut convention: u32 = CallConvention::Standard as u32;

    attributes.iter().for_each(|attribute| match attribute {
        LLVMAttribute::Extern(name, ..) => {
            extern_name = Some(name);
        }

        LLVMAttribute::Convention(conv, _) => {
            convention = (*conv) as u32;
        }
        _ => (),
    });

    let llvm_name: &str = if let Some(ffi_name) = extern_name {
        ffi_name
    } else if is_public {
        function_ascii_name
    } else {
        &format!(
            "__fn_{}_{}",
            utils::generate_random_string(LONG_RANGE_OBFUSCATION),
            function_ascii_name
        )
    };

    let function_type: FunctionType =
        typegen::function_type(context, function_type, function_parameters, ignore_args);

    let llvm_function: FunctionValue = llvm_module.add_function(llvm_name, function_type, None);

    if !is_public && extern_name.is_none() {
        llvm_function.set_linkage(Linkage::LinkerPrivate);
    }

    let mut attribute_builder: AttributeBuilder = AttributeBuilder::new(
        llvm_context,
        attributes,
        LLVMAttributeApplicant::Function(llvm_function),
    );

    attribute_builder.add_function_attributes(&mut convention);

    context.set_current_fn(llvm_function);

    context.new_function(
        funcion_name,
        (llvm_function, function_parameters_types, convention),
    );
}

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, global_fn: GlobalFunction<'ctx>) {
    let llvm_context: &Context = codegen.get_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

    let function_name: &str = global_fn.0;
    let function_type: &Type = global_fn.2;
    let function_parameters: &[Ast<'ctx>] = global_fn.3;
    let funcion_body: &Ast = global_fn.5;

    let represented_llvm_function: LLVMFunction = codegen
        .get_context()
        .get_table()
        .get_function(function_name);

    let llvm_function: FunctionValue = represented_llvm_function.0;

    llvm_builder.position_at_end(llvm_context.append_basic_block(llvm_function, ""));

    codegen.get_mut_context().set_current_fn(llvm_function);

    function_parameters.iter().for_each(|parameter| {
        if let Ast::FunctionParameter {
            name,
            ascii_name,
            kind,
            position,
            ..
        } = parameter
        {
            self::compile_parameter(codegen, llvm_function, (name, ascii_name, kind, *position));
        }
    });

    codegen.codegen_block(funcion_body);

    if function_type.is_void_type() && !funcion_body.has_return_for_function() {
        let _ = llvm_builder.build_return(None).is_err();
    }

    codegen.get_mut_context().unset_current_function();
}

fn compile_parameter<'ctx>(
    codegen: &mut LLVMCodegen<'_, 'ctx>,
    llvm_fn: FunctionValue<'ctx>,
    parameter: FunctionParameter<'ctx>,
) {
    let name: &str = parameter.0;
    let ascii_name: &str = parameter.1;

    let kind: &Type = parameter.2;
    let position: u32 = parameter.3;

    if let Some(value) = llvm_fn.get_nth_param(position) {
        codegen
            .get_mut_context()
            .new_parameter(name, ascii_name, kind, value);
    } else {
        self::codegen_abort(
            "The value of a parameter of an LLVM function could not be obtained at code generation time.",
        );
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
