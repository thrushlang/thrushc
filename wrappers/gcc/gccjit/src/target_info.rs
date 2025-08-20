use context::CType;
use std::{ffi::{CStr, CString}, fmt};

pub struct TargetInfo {
    ptr: *mut gccjit_sys::gcc_jit_target_info,
}

unsafe impl Send for TargetInfo {}
unsafe impl Sync for TargetInfo {}

impl fmt::Debug for TargetInfo {
    fn fmt<'a>(&self, fmt: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        "TargetInfo".fmt(fmt)
    }
}

impl TargetInfo {
    pub fn cpu_supports(&self, feature: &str) -> bool {
        let feature =
            match CString::new(feature) {
                Ok(feature) => feature,
                Err(_) => return false,
            };
        unsafe {
            gccjit_sys::gcc_jit_target_info_cpu_supports(self.ptr, feature.as_ptr()) != 0
        }
    }

    pub fn arch(&self) -> Option<&'static CStr> {
        unsafe {
            let arch = gccjit_sys::gcc_jit_target_info_arch(self.ptr);
            if arch.is_null() {
                return None;
            }
            Some(CStr::from_ptr(arch))
        }
    }

    #[cfg(feature="master")]
    pub fn supports_target_dependent_type(&self, c_type: CType) -> bool {
        unsafe {
            gccjit_sys::gcc_jit_target_info_supports_target_dependent_type(self.ptr, c_type.to_sys()) != 0
        }
    }
}

impl Drop for TargetInfo {
    fn drop(&mut self) {
        unsafe {
            gccjit_sys::gcc_jit_target_info_release(self.ptr);
        }
    }
}

pub unsafe fn from_ptr(ptr: *mut gccjit_sys::gcc_jit_target_info) -> TargetInfo {
    TargetInfo {
        ptr,
    }
}
