use std::ffi::CString;

use ahash::AHashSet as HashSet;
use inkwell::values::{AsValueRef, CallSiteValue, FunctionValue};

pub fn clean_llvm_name(name: &std::ffi::CStr) -> std::borrow::Cow<'_, str> {
    let s: std::borrow::Cow<'_, str> = name.to_string_lossy();

    if let Some(dot_pos) = s.rfind('.') {
        let suffix = &s[dot_pos + 1..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return std::borrow::Cow::Owned(s[..dot_pos].to_string());
        }
    }

    s
}

pub fn get_functions_by_ordered_calls(functions: Vec<FunctionValue>) -> Vec<FunctionValue> {
    let mut ordered: Vec<FunctionValue<'_>> = Vec::with_capacity(u8::MAX as usize);
    let mut visited: HashSet<CString> = HashSet::with_capacity(u8::MAX as usize);

    {
        for function in &functions {
            sort_functions_by_call_recursive(function, &mut visited, &mut ordered);
        }
    }

    ordered
}

fn sort_functions_by_call_recursive<'ctx>(
    function: &FunctionValue<'ctx>,
    visited: &mut HashSet<CString>,
    ordered: &mut Vec<FunctionValue<'ctx>>,
) {
    let func_name: CString = function.get_name().into();

    if visited.contains(&func_name) {
        return;
    }

    if function.get_first_basic_block().is_none() {
        visited.insert(func_name);
        ordered.push(*function);
        return;
    }

    {
        for basic_block in function.get_basic_blocks() {
            let mut instr: Option<inkwell::values::InstructionValue<'_>> =
                basic_block.get_first_instruction();

            while let Some(current_instr) = instr {
                if current_instr.get_opcode() == inkwell::values::InstructionOpcode::Call {
                    let callsite: CallSiteValue<'_> =
                        unsafe { CallSiteValue::new(current_instr.as_value_ref()) };

                    let called_fn: FunctionValue<'_> = callsite.get_called_fn_value();

                    sort_functions_by_call_recursive(&called_fn, visited, ordered);
                }

                instr = current_instr.get_next_instruction();
            }
        }
    }

    if !visited.contains(&func_name) {
        visited.insert(func_name);
        ordered.push(*function);
    }
}
