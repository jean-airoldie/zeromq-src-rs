use std::os::raw::{c_int, c_void};
#[cfg(feature = "libsodium")]
use std::{ffi::CStr, os::raw::c_char};

extern "C" {
    fn zmq_version(major: *mut i32, minor: *mut i32, patch: *mut i32)
        -> c_void;

    #[cfg(feature = "libsodium")]
    fn sodium_version_string() -> *const c_char;
}

pub fn version() -> (i32, i32, i32) {
    let mut major = 0;
    let mut minor = 0;
    let mut patch = 0;
    unsafe {
        zmq_version(
            &mut major as *mut c_int,
            &mut minor as *mut c_int,
            &mut patch as *mut c_int,
        );
    }

    (major, minor, patch)
}

#[cfg(feature = "libsodium")]
pub fn sodium_version() -> &'static CStr {
    unsafe { CStr::from_ptr(sodium_version_string()) }
}
