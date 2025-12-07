#![allow(unused)]

use std::ffi::{CStr, CString};
use crate::si2;

pub unsafe fn open_c_file(path: &str, mode: &str) -> (*mut libc::FILE, *mut si2::FILE) {
    let c_path = CString::new(path).unwrap();
    let c_mode = CString::new(mode).unwrap();
    
    unsafe {
        let fp = libc::fopen(c_path.as_ptr(), c_mode.as_ptr());
        if fp.is_null() {
            panic!("failed to open file");
        }

        let fp_for_lefr = fp as *mut si2::FILE;
        return (fp, fp_for_lefr)
    }
}

pub unsafe fn from_mut_c_char_ptr(raw: *mut ::std::os::raw::c_char) -> String {
    unsafe { CStr::from_ptr(raw).to_string_lossy().into_owned() }
}

pub unsafe fn from_const_c_char_ptr(raw: *const ::std::os::raw::c_char) -> String {
    unsafe { CStr::from_ptr(raw).to_string_lossy().into_owned() }
}