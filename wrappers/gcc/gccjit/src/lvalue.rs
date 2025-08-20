use std::{ffi::CString, marker::PhantomData};
use std::fmt;
use std::ptr;

use context::Context;
use rvalue::{RValue, ToRValue};
use rvalue;
use object::{ToObject, Object};
use object;
use field::Field;
use field;
use location::Location;
use location;

#[cfg(feature="master")]
#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Default,
    Hidden,
    Internal,
    Protected,
}

#[cfg(feature="master")]
impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Visibility::Default => "default",
            Visibility::Hidden => "hidden",
            Visibility::Internal => "internal",
            Visibility::Protected => "protected",
        }
    }
}

#[cfg(feature="master")]
pub enum AttributeValue<'a> {
    #[allow(dead_code)]
    Int(i32),
    None,
    String(&'a str),
    IntArray(&'a [std::ffi::c_int]),
}

#[cfg(feature="master")]
#[derive(Clone, Copy, Debug)]
pub enum VarAttribute {
    Visibility(Visibility),
    Weak,
}

#[cfg(feature="master")]
impl VarAttribute {
    fn get_value(&self) -> AttributeValue<'_> {
        match *self {
            Self::Visibility(visibility) => AttributeValue::String(visibility.as_str()),
            Self::Weak => AttributeValue::None,
        }
    }

    fn to_sys(self) -> gccjit_sys::gcc_jit_variable_attribute {
        match self {
            VarAttribute::Visibility(_) => gccjit_sys::gcc_jit_variable_attribute::GCC_JIT_VARIABLE_ATTRIBUTE_VISIBILITY,
            VarAttribute::Weak => gccjit_sys::gcc_jit_variable_attribute::GCC_JIT_VARIABLE_ATTRIBUTE_WEAK,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TlsModel {
    GlobalDynamic,
    LocalDynamic,
    InitialExec,
    LocalExec,
    None,
}

impl TlsModel {
    fn to_sys(self) -> gccjit_sys::gcc_jit_tls_model {
        use gccjit_sys::gcc_jit_tls_model::*;

        match self {
            TlsModel::GlobalDynamic => GCC_JIT_TLS_MODEL_GLOBAL_DYNAMIC,
            TlsModel::LocalDynamic => GCC_JIT_TLS_MODEL_LOCAL_DYNAMIC,
            TlsModel::InitialExec => GCC_JIT_TLS_MODEL_INITIAL_EXEC,
            TlsModel::LocalExec => GCC_JIT_TLS_MODEL_LOCAL_EXEC,
            TlsModel::None => GCC_JIT_TLS_MODEL_NONE,
        }
    }
}

/// An LValue in gccjit represents a value that has a concrete
/// location in memory. A LValue can be converted into an RValue
/// through the ToRValue trait.
/// It is also possible to get the address of an LValue.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct LValue<'ctx> {
    marker: PhantomData<&'ctx Context<'ctx>>,
    ptr: *mut gccjit_sys::gcc_jit_lvalue
}

/// ToLValue is a trait implemented by types that can be converted (or treated
/// as) LValues.
pub trait ToLValue<'ctx> {
    fn to_lvalue(&self) -> LValue<'ctx>;
}

impl<'ctx> ToObject<'ctx> for LValue<'ctx> {
    fn to_object(&self) -> Object<'ctx> {
        unsafe {
            object::from_ptr(gccjit_sys::gcc_jit_lvalue_as_object(self.ptr))
        }
    }
}

impl<'ctx> fmt::Debug for LValue<'ctx> {
    fn fmt<'a>(&self, fmt: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        let obj = self.to_object();
        obj.fmt(fmt)
    }
}

impl<'ctx> ToLValue<'ctx> for LValue<'ctx> {
    fn to_lvalue(&self) -> LValue<'ctx> {
        unsafe { from_ptr(self.ptr) }
    }
}

impl<'ctx> ToRValue<'ctx> for LValue<'ctx> {
    fn to_rvalue(&self) -> RValue<'ctx> {
        unsafe {
            let ptr = gccjit_sys::gcc_jit_lvalue_as_rvalue(self.ptr);
            rvalue::from_ptr(ptr)
        }
    }
}

impl<'ctx> LValue<'ctx> {
    /// Given an LValue x and a Field f, gets an LValue for the field
    /// access x.f.
    pub fn access_field(&self,
                        loc: Option<Location<'ctx>>,
                        field: Field<'ctx>) -> LValue<'ctx> {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut()
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_lvalue_access_field(self.ptr,
                                                              loc_ptr,
                                                              field::get_ptr(&field));
            from_ptr(ptr)
        }
    }

    /// Given an LValue x, returns the RValue address of x, akin to C's &x.
    pub fn get_address(&self,
                       loc: Option<Location<'ctx>>) -> RValue<'ctx> {
        let loc_ptr = match loc {
            Some(loc) => unsafe { location::get_ptr(&loc) },
            None => ptr::null_mut()
        };
        unsafe {
            let ptr = gccjit_sys::gcc_jit_lvalue_get_address(self.ptr,
                                                             loc_ptr);
            rvalue::from_ptr(ptr)
        }
    }

    /// Set the initialization value for a global variable.
    pub fn global_set_initializer(&self, blob: &[u8]) {
        unsafe {
            gccjit_sys::gcc_jit_global_set_initializer(self.ptr, blob.as_ptr() as _, blob.len() as _);
        }
    }

    /// Set the initialization value for a global variable.
    pub fn global_set_initializer_rvalue(&self, value: RValue<'ctx>) -> LValue<'ctx> {
        unsafe {
            from_ptr(gccjit_sys::gcc_jit_global_set_initializer_rvalue(self.ptr, rvalue::get_ptr(&value)))
        }
    }

    pub fn set_tls_model(&self, model: TlsModel) {
        unsafe {
            gccjit_sys::gcc_jit_lvalue_set_tls_model(self.ptr, model.to_sys());
        }
    }

    pub fn set_link_section(&self, name: &str) {
        let name = CString::new(name).unwrap();
        unsafe {
            gccjit_sys::gcc_jit_lvalue_set_link_section(self.ptr, name.as_ptr());
        }
    }

    #[cfg(feature="master")]
    pub fn global_set_readonly(&self) {
        unsafe {
            gccjit_sys::gcc_jit_global_set_readonly(self.ptr);
        }
    }

    pub fn set_register_name(&self, reg_name: &str) {
        let name = CString::new(reg_name).unwrap();
        unsafe {
            gccjit_sys::gcc_jit_lvalue_set_register_name(self.ptr, name.as_ptr());
        }
    }

    pub fn set_alignment(&self, alignment: i32) {
        unsafe {
            gccjit_sys::gcc_jit_lvalue_set_alignment(self.ptr, alignment);
        }
    }

    pub fn get_alignment(&self) -> i32 {
        unsafe {
            gccjit_sys::gcc_jit_lvalue_get_alignment(self.ptr)
        }
    }

    #[cfg(feature="master")]
    pub fn add_attribute(&self, attribute: VarAttribute) {
        let value = attribute.get_value();
        match value {
            AttributeValue::Int(_) => unimplemented!(),
            AttributeValue::IntArray(_) => unimplemented!(),
            AttributeValue::None => {
                unsafe {
                    gccjit_sys::gcc_jit_lvalue_add_attribute(self.ptr, attribute.to_sys());
                }
            },
            AttributeValue::String(string) => {
                let cstr = CString::new(string).unwrap();
                unsafe {
                    gccjit_sys::gcc_jit_lvalue_add_string_attribute(self.ptr, attribute.to_sys(), cstr.as_ptr());
                }
            },
        }
    }

    #[cfg(feature = "master")]
    pub fn get_name(&self) -> Option<&'ctx str> {
        unsafe {
            let str = gccjit_sys::gcc_jit_lvalue_get_name(self.ptr);
            if str.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(str).to_str().expect("invalid lvalue name"))
            }
        }
    }

    #[cfg(feature = "master")]
    pub fn set_name(&self, new_name: &str) {
        let new_name = CString::new(new_name).unwrap();
        unsafe {
            gccjit_sys::gcc_jit_lvalue_set_name(self.ptr, new_name.as_ptr());
        }
    }
}

pub unsafe fn from_ptr<'ctx>(ptr: *mut gccjit_sys::gcc_jit_lvalue) -> LValue<'ctx> {
    LValue {
        marker: PhantomData,
        ptr
    }
}

pub unsafe fn get_ptr<'ctx>(lvalue: &LValue<'ctx>) -> *mut gccjit_sys::gcc_jit_lvalue {
    lvalue.ptr
}
