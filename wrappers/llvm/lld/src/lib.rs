use std::ffi::CStr;

use std::{
    ffi::CString,
    os::raw::{c_char, c_int},
};

#[repr(C)]
struct LldInvokeResult {
    success: bool,
    messages: *const c_char,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum LldFlavor {
    Elf = 0,
    Wasm = 1,
    MachO = 2,
    Coff = 3,
}

unsafe extern "C" {
    unsafe fn lld_link(
        flavor: LldFlavor,
        argc: c_int,
        argv: *const *const c_char,
    ) -> LldInvokeResult;
    unsafe fn link_free_result(result: *mut LldInvokeResult);
}

#[derive(Debug)]
pub struct LLVMLinkerResult {
    success: bool,
    messages: String,
}

impl LLVMLinkerResult {
    pub fn new(success: bool, messages: String) -> Self {
        Self { success, messages }
    }

    pub fn get_state(&self) -> bool {
        self.success
    }

    pub fn get_messages(&self) -> &str {
        &self.messages
    }
}

pub fn link_all(flavor: LldFlavor, args: Vec<&str>) -> LLVMLinkerResult {
    let c_args: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(arg.as_bytes()).unwrap())
        .collect::<Vec<CString>>();

    let args: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();

    let mut lld_result: LldInvokeResult =
        unsafe { lld_link(flavor, args.len() as c_int, args.as_ptr()) };

    let messages: String = if !lld_result.messages.is_null() {
        unsafe {
            CStr::from_ptr(lld_result.messages)
                .to_string_lossy()
                .to_string()
        }
    } else {
        String::new()
    };

    let result: LLVMLinkerResult = LLVMLinkerResult::new(lld_result.success, messages);

    unsafe { link_free_result(&mut lld_result as *mut LldInvokeResult) };

    drop(lld_result);

    result
}
