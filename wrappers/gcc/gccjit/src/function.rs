use std::marker::PhantomData;
use std::fmt;
use std::ptr;

use block::Block;
use block;
use context::Context;
use location::Location;
use location;
#[cfg(feature="master")]
use lvalue::{AttributeValue, Visibility};
use lvalue::LValue;
use lvalue;
use object::{ToObject, Object};
use object;
use parameter::Parameter;
use parameter;
use rvalue::{self, RValue};
use std::ffi::CString;
use types::Type;
use types;

/// FunctionType informs gccjit what sort of function a new function will be.
/// An exported function is a function that will be exported using the CompileResult
/// interface, able to be called outside of the jit. An internal function is
/// a function that cannot be called outside of jitted code. An extern function
/// is a function with external linkage, and always inline is a function that is
/// always inlined wherever it is called and cannot be accessed outside of the jit.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FunctionType {
    /// Defines a function that is "exported" by the JIT and can be called from
    /// Rust.
    Exported,
    /// Defines a function that is internal to the JIT and cannot be called
    /// from Rust, but can be called from jitted code.
    Internal,
    /// Defines a function with external linkage.
    Extern,
    /// Defines a function that should always be inlined whenever it is called.
    /// A function with this type cannot be called from Rust. If the optimization
    /// level is None, this function will not actually be inlined, but it still
    /// can only be called from within jitted code.
    AlwaysInline
}

#[cfg(feature="master")]
#[derive(Clone, Debug)]
pub enum FnAttribute<'a> {
    Alias(&'a str),
    AlwaysInline,
    Inline,
    NoInline,
    Target(&'a str),
    Used,
    Visibility(Visibility),
    Cold,
    ReturnsTwice,
    Pure,
    Const,
    Weak,
    NonNull(Vec<std::ffi::c_int>),
    ArmCmseNonsecureCall,
    ArmCmseNonsecureEntry,
    ArmPcs(&'a str),
    AvrInterrupt,
    AvrNoblock,
    AvrSignal,
    GcnAmdGpuHsaKernel,
    Msp430Interrupt,
    NvptxKernel,
    RiscvInterrupt(&'a str),
    X86FastCall,
    X86Interrupt,
    X86MsAbi,
    X86Stdcall,
    X86SysvAbi,
    X86ThisCall,
}

#[cfg(feature="master")]
impl<'a> FnAttribute<'a> {
    fn get_value(&self) -> AttributeValue<'_> {
        match *self {
            FnAttribute::Alias(value) | FnAttribute::ArmPcs(value)| FnAttribute::RiscvInterrupt(value)
                | FnAttribute::Target(value) => AttributeValue::String(value),
            FnAttribute::Visibility(visibility) => AttributeValue::String(visibility.as_str()),
            FnAttribute::AlwaysInline
            | FnAttribute::Inline
            | FnAttribute::NoInline
            | FnAttribute::Used
            | FnAttribute::Cold
            | FnAttribute::ReturnsTwice
            | FnAttribute::Pure
            | FnAttribute::Const
            | FnAttribute::Weak
            | FnAttribute::ArmCmseNonsecureCall
            | FnAttribute::ArmCmseNonsecureEntry
            | FnAttribute::AvrInterrupt
            | FnAttribute::AvrNoblock
            | FnAttribute::AvrSignal
            | FnAttribute::GcnAmdGpuHsaKernel
            | FnAttribute::Msp430Interrupt
            | FnAttribute::NvptxKernel
            | FnAttribute::X86FastCall
            | FnAttribute::X86Interrupt
            | FnAttribute::X86MsAbi
            | FnAttribute::X86Stdcall
            | FnAttribute::X86SysvAbi
            | FnAttribute::X86ThisCall => AttributeValue::None,
            FnAttribute::NonNull(ref value) => {
                debug_assert!(
                    value.iter().all(|attr| *attr > 0),
                    "all values must be > 0 for non-null attribute",
                );
                AttributeValue::IntArray(value)
            }
        }
    }

    fn as_sys(&self) -> gccjit_sys::gcc_jit_fn_attribute {
        match *self {
            FnAttribute::Alias(_) => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_ALIAS,
            FnAttribute::AlwaysInline => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_ALWAYS_INLINE,
            FnAttribute::Inline => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_INLINE,
            FnAttribute::NoInline => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_NOINLINE,
            FnAttribute::Target(_) => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_TARGET,
            FnAttribute::Used => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_USED,
            FnAttribute::Visibility(_) => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_VISIBILITY,
            FnAttribute::Cold => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_COLD,
            FnAttribute::ReturnsTwice => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_RETURNS_TWICE,
            FnAttribute::Pure => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_PURE,
            FnAttribute::Const => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_CONST,
            FnAttribute::Weak => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_WEAK,
            FnAttribute::NonNull(_) => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_NONNULL,
            FnAttribute::ArmCmseNonsecureCall => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_ARM_CMSE_NONSECURE_CALL,
            FnAttribute::ArmCmseNonsecureEntry => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_ARM_CMSE_NONSECURE_ENTRY,
            FnAttribute::ArmPcs(_) => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_ARM_PCS,
            FnAttribute::AvrInterrupt => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_AVR_INTERRUPT,
            FnAttribute::AvrNoblock => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_AVR_NOBLOCK,
            FnAttribute::AvrSignal => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_AVR_SIGNAL,
            FnAttribute::GcnAmdGpuHsaKernel => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_GCN_AMDGPU_HSA_KERNEL,
            FnAttribute::Msp430Interrupt => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_MSP430_INTERRUPT,
            FnAttribute::NvptxKernel => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_NVPTX_KERNEL,
            FnAttribute::RiscvInterrupt(_) => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_RISCV_INTERRUPT,
            FnAttribute::X86FastCall => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_X86_FAST_CALL,
            FnAttribute::X86Interrupt => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_X86_INTERRUPT,
            FnAttribute::X86MsAbi => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_X86_MS_ABI,
            FnAttribute::X86Stdcall => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_X86_STDCALL,
            FnAttribute::X86SysvAbi => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_X86_SYSV_ABI,
            FnAttribute::X86ThisCall => gccjit_sys::gcc_jit_fn_attribute::GCC_JIT_FN_ATTRIBUTE_X86_THIS_CALL,
        }
    }
}

/// Function is gccjit's representation of a function. Functions are constructed
/// by constructing basic blocks and connecting them together. Locals are declared
/// at the function level.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct Function<'ctx> {
    marker: PhantomData<&'ctx Context<'ctx>>,
    ptr: *mut gccjit_sys::gcc_jit_function
}

impl<'ctx> ToObject<'ctx> for Function<'ctx> {
    fn to_object(&self) -> Object<'ctx> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_function_as_object(self.ptr);
            object::from_ptr(ptr)
        }
    }
}

impl<'ctx> fmt::Debug for Function<'ctx> {
    fn fmt<'a>(&self, fmt: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

impl<'ctx> Function<'ctx> {
    pub fn get_param(&self, idx: i32) -> Parameter<'ctx> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_function_get_param(self.ptr, idx);
            #[cfg(debug_assertions)]
            if let Ok(Some(error)) = self.to_object().get_context().get_last_error() {
                panic!("{} ({:?})", error, self);
            }
            parameter::from_ptr(ptr)
        }
    }

    pub fn get_param_count(&self) -> usize {
        unsafe {
            gccjit_sys::gcc_jit_function_get_param_count(self.ptr) as usize
        }
    }

    pub fn get_return_type(&self) -> Type<'ctx> {
        unsafe {
            types::from_ptr(gccjit_sys::gcc_jit_function_get_return_type(self.ptr))
        }
    }

    pub fn get_address(&self, loc: Option<Location<'ctx>>) -> RValue<'ctx> {
        unsafe {
            let loc_ptr = match loc {
                Some(loc) => location::get_ptr(&loc),
                None => ptr::null_mut()
            };
            let ptr = gccjit_sys::gcc_jit_function_get_address(self.ptr, loc_ptr);
            rvalue::from_ptr(ptr)
        }
    }

    pub fn dump_to_dot<S: AsRef<str>>(&self, path: S) {
        unsafe {
            let cstr = CString::new(path.as_ref()).unwrap();
            gccjit_sys::gcc_jit_function_dump_to_dot(self.ptr, cstr.as_ptr());
        }
    }

    pub fn new_block<S: AsRef<str>>(&self, name: S) -> Block<'ctx> {
        unsafe {
            let cstr = CString::new(name.as_ref()).unwrap();
            let ptr = gccjit_sys::gcc_jit_function_new_block(self.ptr,
                                                             cstr.as_ptr());
            #[cfg(debug_assertions)]
            if let Ok(Some(error)) = self.to_object().get_context().get_last_error() {
                panic!("{} ({:?})", error, self);
            }
            block::from_ptr(ptr)
        }
    }

    #[cfg(feature="master")]
    pub fn set_personality_function(&self, personality_func: Function<'ctx>) {
        unsafe {
            gccjit_sys::gcc_jit_function_set_personality_function(self.ptr, personality_func.ptr);
        }
    }

    pub fn new_local<S: AsRef<str>>(&self,
                     loc: Option<Location<'ctx>>,
                     ty: Type<'ctx>,
                     name: S) -> LValue<'ctx> {
        unsafe {
            let loc_ptr = match loc {
                Some(loc) => location::get_ptr(&loc),
                None => ptr::null_mut()
            };
            let cstr = CString::new(name.as_ref()).unwrap();
            let ptr = gccjit_sys::gcc_jit_function_new_local(self.ptr,
                                                             loc_ptr,
                                                             types::get_ptr(&ty),
                                                             cstr.as_ptr());
            #[cfg(debug_assertions)]
            if let Ok(Some(error)) = self.to_object().get_context().get_last_error() {
                panic!("{} ({:?})", error, self);
            }
            lvalue::from_ptr(ptr)
        }
    }

    #[cfg(feature="master")]
    pub fn new_temp(&self, loc: Option<Location<'ctx>>, ty: Type<'ctx>) -> LValue<'ctx> {
        unsafe {
            let loc_ptr = match loc {
                Some(loc) => location::get_ptr(&loc),
                None => ptr::null_mut()
            };
            let ptr = gccjit_sys::gcc_jit_function_new_temp(self.ptr, loc_ptr, types::get_ptr(&ty));
            #[cfg(debug_assertions)]
            if let Ok(Some(error)) = self.to_object().get_context().get_last_error() {
                panic!("{} ({:?})", error, self);
            }
            lvalue::from_ptr(ptr)
        }
    }

    #[cfg(feature="master")]
    pub fn add_attribute<'a>(&self, attribute: FnAttribute<'a>) {
        let value = attribute.get_value();
        match value {
            AttributeValue::Int(value) => {
                // Basically the same as `IntArray` but for only one element.
                let value = &[value];
                unsafe {
                    gccjit_sys::gcc_jit_function_add_integer_array_attribute(
                        self.ptr,
                        attribute.as_sys(),
                        value.as_ptr(),
                        value.len() as _,
                    );
                }

            }
            AttributeValue::IntArray(value) => {
                unsafe {
                    gccjit_sys::gcc_jit_function_add_integer_array_attribute(
                        self.ptr,
                        attribute.as_sys(),
                        value.as_ptr(),
                        value.len() as _,
                    );
                }
            }
            AttributeValue::None => {
                unsafe {
                    gccjit_sys::gcc_jit_function_add_attribute(self.ptr, attribute.as_sys());
                }
            },
            AttributeValue::String(string) => {
                let cstr = CString::new(string).unwrap();
                unsafe {
                    gccjit_sys::gcc_jit_function_add_string_attribute(self.ptr, attribute.as_sys(), cstr.as_ptr());
                }
            },
        }
    }
}

pub unsafe fn from_ptr<'ctx>(ptr: *mut gccjit_sys::gcc_jit_function) -> Function<'ctx> {
    Function {
        marker: PhantomData,
        ptr
    }
}

pub unsafe fn get_ptr<'ctx>(loc: &Function<'ctx>) -> *mut gccjit_sys::gcc_jit_function {
    loc.ptr
}
